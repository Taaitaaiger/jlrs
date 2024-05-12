use std::{
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{channel as mpsc_channel, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
};

use async_channel::Receiver;
use fnv::FnvHashMap;
use jl_sys::{jl_adopt_thread, jlrs_clear_gc_stack, jlrs_gc_safe_enter, jlrs_ptls_from_gcstack};
use once_cell::sync::OnceCell;

use crate::{
    memory::gc::gc_unsafe_with,
    prelude::StackFrame,
    runtime::{
        executor::Executor,
        handle::{
            async_handle::{
                cancellation_token::CancellationToken, channel::channel, message::Message,
                on_adopted_thread, AsyncHandle,
            },
            mt_handle::drop_handle,
        },
    },
};

static MANAGER: OnceCell<Manager> = OnceCell::new();
static POOL_ID: AtomicUsize = AtomicUsize::new(0);
static WORKER_ID: AtomicUsize = AtomicUsize::new(0);

type Spawner = Box<
    dyn Send + Sync + Fn(PoolId, WorkerId, CancellationToken, Receiver<Message>) -> JoinHandle<()>,
>;

#[derive(Hash, Debug, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub(crate) struct PoolId(usize);

impl PoolId {
    pub(crate) fn inner(self) -> usize {
        self.0
    }

    fn next() -> PoolId {
        PoolId(POOL_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Hash, Debug, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub(crate) struct WorkerId(usize);

impl WorkerId {
    pub(crate) fn inner(self) -> usize {
        self.0
    }

    fn next() -> WorkerId {
        WorkerId(WORKER_ID.fetch_add(1, Ordering::Relaxed))
    }
}

struct WorkerHandle {
    handle: JoinHandle<()>,
    token: CancellationToken,
}

impl WorkerHandle {
    fn join(self) -> thread::Result<()> {
        self.handle.join()
    }

    fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    fn cancel(&self) {
        self.token.cancel()
    }
}

struct Pool {
    pool_id: PoolId,
    n_workers: Arc<AtomicUsize>,
    handles: FnvHashMap<WorkerId, WorkerHandle>,
    spawner: Spawner,
    receiver: Receiver<Message>,
}

impl Pool {
    fn new(
        pool_id: PoolId,
        n_workers: Arc<AtomicUsize>,
        spawner: Spawner,
        receiver: Receiver<Message>,
    ) -> Self {
        let handles = (0..n_workers.load(Ordering::Relaxed))
            .map(|_| {
                let worker_id = WorkerId::next();
                let token = CancellationToken::new();

                let handle = spawner(pool_id, worker_id, token.clone(), receiver.clone());
                (worker_id, WorkerHandle { handle, token })
            })
            .collect();

        Pool {
            pool_id,
            n_workers,
            handles,
            spawner,
            receiver,
        }
    }

    fn add_worker(&mut self) {
        let worker_id = WorkerId::next();
        let token = CancellationToken::new();

        let handle = (self.spawner)(
            self.pool_id,
            worker_id,
            token.clone(),
            self.receiver.clone(),
        );

        self.n_workers.fetch_add(1, Ordering::Relaxed);
        self.handles
            .insert(worker_id, WorkerHandle { handle, token });
    }

    fn restart_worker(&mut self, worker_id: WorkerId) {
        // Reuse same token.
        let token = self
            .handles
            .get(&worker_id)
            .expect("unknown worker")
            .token
            .clone();

        let handle = (self.spawner)(
            self.pool_id,
            worker_id,
            token.clone(),
            self.receiver.clone(),
        );
        if let Some(handle) = self
            .handles
            .insert(worker_id, WorkerHandle { handle, token })
        {
            handle.join().ok();
        }
    }

    fn cancel_worker(&mut self) {
        for (_, handle) in self.handles.iter() {
            if !handle.is_cancelled() {
                handle.cancel();
                break;
            }
        }
    }

    fn cancel_all_workers(&mut self) {
        for (_, handle) in self.handles.iter() {
            handle.cancel();
        }
    }

    fn drop_worker(&mut self, worker_id: WorkerId) -> usize {
        self.handles.remove(&worker_id).map(|handle| {
            handle.join().ok();
            self.n_workers.fetch_sub(1, Ordering::Relaxed);
        });

        self.handles.len()
    }
}

pub(crate) struct Manager {
    sender: Sender<ManagerMsg>,
}

impl Manager {
    pub(crate) fn spawn_pool<E: Executor<N>, const N: usize>(
        &self,
        executor_opts: E,
        channel_capacity: usize,
        n_workers: usize,
        prefix: Option<String>,
    ) -> AsyncHandle {
        let pool_id = PoolId::next();
        let (sender, receiver) = channel(channel_capacity);

        let spawn_worker = spawn_worker::<E, N>;
        let e = Arc::new(executor_opts);
        let spawn_worker = Box::new(
            move |pool_id: PoolId,
                  worker_id: WorkerId,
                  token: CancellationToken,
                  receiver: Receiver<Message>| {
                let e = e.clone();
                spawn_worker(e, token, prefix.clone(), pool_id, worker_id, receiver)
            },
        );

        let n_workers = Arc::new(AtomicUsize::new(n_workers));

        let spawn_pool = ManagerMsg::SpawnPool {
            pool_id,
            n_workers: n_workers.clone(),
            receiver,
            spawner: spawn_worker,
        };

        self.sender.send(spawn_pool).ok();
        unsafe { AsyncHandle::new(sender, pool_id, n_workers) }
    }

    pub(crate) fn drop_pool(&self, pool_id: &PoolId) {
        let drop_pool = ManagerMsg::DropPool {
            pool_id: PoolId(pool_id.0),
        };
        self.sender.send(drop_pool).ok();
    }

    pub(crate) fn add_worker(&self, pool_id: &PoolId) {
        let add_worker = ManagerMsg::AddWorker {
            pool_id: PoolId(pool_id.0),
        };
        self.sender.send(add_worker).ok();
    }

    pub(crate) fn remove_worker(&self, pool_id: &PoolId) {
        let remove_worker = ManagerMsg::RemoveWorker {
            pool_id: PoolId(pool_id.0),
        };
        self.sender.send(remove_worker).ok();
    }

    pub(crate) fn restart_worker(&self, pool_id: PoolId, worker_id: WorkerId) {
        let restart_worker = ManagerMsg::RestartWorker { pool_id, worker_id };
        self.sender.send(restart_worker).ok();
    }

    pub(crate) fn drop_worker(&self, pool_id: PoolId, worker_id: WorkerId) {
        let drop_worker = ManagerMsg::DropWorker { pool_id, worker_id };
        self.sender.send(drop_worker).ok();
    }
}

enum ManagerMsg {
    SpawnPool {
        pool_id: PoolId,
        n_workers: Arc<AtomicUsize>,
        receiver: Receiver<Message>,
        spawner: Spawner,
    },
    DropPool {
        pool_id: PoolId,
    },
    AddWorker {
        pool_id: PoolId,
    },
    RemoveWorker {
        pool_id: PoolId,
    },
    DropWorker {
        pool_id: PoolId,
        worker_id: WorkerId,
    },
    RestartWorker {
        pool_id: PoolId,
        worker_id: WorkerId,
    },
}

struct Pools {
    pools: FnvHashMap<PoolId, Pool>,
}

impl Pools {
    fn new() -> Self {
        Pools {
            pools: FnvHashMap::default(),
        }
    }

    fn spawn_pool(
        &mut self,
        pool_id: PoolId,
        n_workers: Arc<AtomicUsize>,
        receiver: Receiver<Message>,
        spawner: Spawner,
    ) {
        let pool = Pool::new(pool_id, n_workers, spawner, receiver);
        self.pools.insert(pool_id, pool);
    }

    fn cancel_all_workers(&mut self, pool_id: PoolId) {
        self.pools
            .get_mut(&pool_id)
            .map(|pool| pool.cancel_all_workers());
    }

    fn add_worker(&mut self, pool_id: PoolId) {
        if let Some(pool) = self.pools.get_mut(&pool_id) {
            pool.add_worker();
        }
    }

    fn cancel_worker(&mut self, pool_id: PoolId) {
        if let Some(pool) = self.pools.get_mut(&pool_id) {
            pool.cancel_worker();
        }
    }

    fn drop_worker(&mut self, pool_id: PoolId, worker_id: WorkerId) {
        let remove_pool = if let Some(pool) = self.pools.get_mut(&pool_id) {
            let n = pool.drop_worker(worker_id);
            n == 0
        } else {
            false
        };

        if remove_pool {
            self.pools
                .remove(&pool_id)
                .map(|pool| pool.receiver.close());
            unsafe { drop_handle() }
        }
    }

    fn restart_worker(&mut self, pool_id: PoolId, worker_id: WorkerId) {
        if let Some(pool) = self.pools.get_mut(&pool_id) {
            pool.restart_worker(worker_id);
        }
    }
}

pub(crate) fn get_manager() -> &'static Manager {
    MANAGER.get_or_init(|| {
        let (sender, receiver) = mpsc_channel();

        let _ = thread::spawn(move || {
            let mut pools = Pools::new();
            loop {
                match receiver.recv() {
                    Ok(ManagerMsg::SpawnPool {
                        pool_id,
                        n_workers,
                        receiver,
                        spawner,
                    }) => pools.spawn_pool(pool_id, n_workers, receiver, spawner),
                    Ok(ManagerMsg::DropPool { pool_id }) => pools.cancel_all_workers(pool_id),
                    Ok(ManagerMsg::AddWorker { pool_id }) => pools.add_worker(pool_id),
                    Ok(ManagerMsg::RemoveWorker { pool_id }) => pools.cancel_worker(pool_id),
                    Ok(ManagerMsg::DropWorker { pool_id, worker_id }) => {
                        pools.drop_worker(pool_id, worker_id)
                    }
                    Ok(ManagerMsg::RestartWorker { pool_id, worker_id }) => {
                        pools.restart_worker(pool_id, worker_id)
                    }
                    Err(_) => break,
                }
            }
        });

        Manager { sender }
    })
}

pub(super) fn spawn_worker<R: Executor<N>, const N: usize>(
    executor_opts: Arc<R>,
    token: CancellationToken,
    prefix: Option<String>,
    pool_id: PoolId,
    worker_id: WorkerId,
    receiver: Receiver<Message>,
) -> JoinHandle<()> {
    let prefix = prefix.unwrap_or_else(|| "jlrs".into()).replace('\0', "");

    let name = format!("{}-{}-{}", prefix, pool_id.inner(), worker_id.inner());
    thread::Builder::new()
        .name(name)
        .spawn(move || unsafe {
            let pgcstack = jl_adopt_thread();
            let ptls = jlrs_ptls_from_gcstack(pgcstack);

            // Tasks are run in GC-unsafe block so we can be GC-safe otherwise.
            jlrs_gc_safe_enter(ptls);

            // Catch unwind to detect that this thread has panicked so we can clean up any
            // lingering state, leave the thread in the GC-safe state, and request this worker to
            // be restarted.
            //
            // If an exception is thrown, we're going to let Julia abort the process.
            let mut base_frame = StackFrame::<N>::new_n();
            let res = catch_unwind(AssertUnwindSafe(|| {
                // Do _not_ use a GcSafeFuture, we've explicitly entered a GC-safe state and
                // require the `run_worker` to enter a GC-unsafe state whenever it needs to call
                // into Julia.
                executor_opts.block_on(on_adopted_thread::<R, N>(receiver, token, &mut base_frame));
            }));

            let manager = get_manager();
            match res {
                Ok(_) => {
                    // Clean exit
                    jlrs_gc_safe_enter(ptls);
                    manager.drop_worker(pool_id, worker_id);
                }
                Err(e) => {
                    // Exit due to panic.
                    // There might be some lingering gc frames on the stack, clear them before
                    // entering the GC-safe state. Don't assume we're in a GC-unsafe state.
                    gc_unsafe_with(ptls, |_| jlrs_clear_gc_stack());
                    jlrs_gc_safe_enter(ptls);
                    manager.restart_worker(pool_id, worker_id);

                    resume_unwind(e)
                }
            };
        })
        .expect("cannot start worker")
}
