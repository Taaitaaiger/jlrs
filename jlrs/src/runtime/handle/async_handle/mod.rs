//! A handle to Julia running on a background thread.

use std::{
    cell::RefCell,
    collections::VecDeque,
    ffi::{c_void, CStr},
    path::Path,
    ptr::NonNull,
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use async_channel::{Receiver, Sender, TryRecvError};
use jl_sys::{jl_gcframe_t, jlrs_gc_unsafe_enter, jlrs_gc_unsafe_leave, jlrs_ppgcstack};
use tokio::sync::oneshot::channel as oneshot_channel;

#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
use self::task_complete::{TaskComplete, TaskCompleteState};
use self::{
    cancellation_token::CancellationToken,
    dispatch::Dispatch,
    envelope::{
        BlockingTask, IncludeTask, PendingTask, Persistent, RegisterTask, SetErrorColorTask, Task,
    },
    message::{Message, MessageInner},
    persistent::PersistentHandle,
};
#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
use super::mt_handle::manager::{get_manager, PoolId};
use crate::{
    async_util::{
        future::{wake_task, GcUnsafeFuture},
        task::{sleep, AsyncTask, PersistentTask, Register},
    },
    convert::into_jlrs_result::IntoJlrsResult,
    error::IOError,
    memory::{gc::gc_unsafe_with, get_tls, stack_frame::JlrsStackFrame, target::frame::GcFrame},
    prelude::{JlrsResult, LocalScope, Module, StackFrame, Value},
    runtime::executor::{Executor, IsFinished},
    util::RequireSendSync,
    weak_handle_unchecked,
};

pub(crate) mod cancellation_token;
pub mod channel;
pub mod dispatch;
mod envelope;
pub mod message;
pub mod persistent;
#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
mod task_complete;

/// A handle to the async runtime.
///
/// This handle can be used to include files and send new tasks to the runtime. The runtime shuts
/// down when the last handle is dropped and all active tasks have completed.
#[derive(Clone)]
pub struct AsyncHandle {
    sender: Sender<Message>,
    pool_or_token: PoolIdOrToken,
    n_workers: Arc<AtomicUsize>,
}

impl AsyncHandle {
    /// Prepare to send a new async task.
    pub fn task<A>(&self, task: A) -> Dispatch<Message, A::Output>
    where
        A: AsyncTask,
    {
        let (sender, receiver) = oneshot_channel();
        let pending_task = PendingTask::<_, _, Task>::new(task, sender);
        let boxed = Box::new(pending_task);
        let msg = MessageInner::Task(boxed).wrap();

        Dispatch::new(msg, &self.sender, receiver)
    }

    /// Prepare to register a task.
    pub fn register_task<R>(&self) -> Dispatch<Message, JlrsResult<()>>
    where
        R: Register,
    {
        let (sender, receiver) = oneshot_channel();
        let pending_task = PendingTask::<R, _, RegisterTask>::new(sender);
        let boxed = Box::new(pending_task);
        let msg = MessageInner::Task(boxed).wrap();

        Dispatch::new(msg, &self.sender, receiver)
    }

    /// Prepare to send a new blocking task.
    pub fn blocking_task<T, F>(&self, task: F) -> Dispatch<Message, T>
    where
        for<'base> F: 'static + Send + FnOnce(GcFrame<'base>) -> T,
        T: Send + 'static,
    {
        let (sender, receiver) = oneshot_channel();
        let pending_task = BlockingTask::new(task, sender);
        let boxed = Box::new(pending_task);
        let msg = MessageInner::BlockingTask(boxed).wrap();

        Dispatch::new(msg, &self.sender, receiver)
    }

    /// Prepare to send a new persistent task.
    pub fn persistent<P>(&self, task: P) -> Dispatch<Message, JlrsResult<PersistentHandle<P>>>
    where
        P: PersistentTask,
    {
        let (sender, receiver) = oneshot_channel();
        let pending_task = PendingTask::<_, _, Persistent>::new(task, sender);
        let boxed = Box::new(pending_task);
        let msg = MessageInner::Task(boxed).wrap();

        Dispatch::new(msg, &self.sender, receiver)
    }

    /// Prepare to include a file.
    ///
    /// Returns an error if the file doesn't exist.
    ///
    /// Safety: the content of the file is evaluated if it exists, which can't be checked for
    /// correctness.
    pub unsafe fn include<P>(&self, path: P) -> JlrsResult<Dispatch<Message, JlrsResult<()>>>
    where
        P: AsRef<Path>,
    {
        if !path.as_ref().exists() {
            Err(IOError::NotFound {
                path: path.as_ref().to_string_lossy().into(),
            })?
        }

        let (sender, receiver) = oneshot_channel();
        let pending_task = IncludeTask::new(path.as_ref().into(), sender);
        let msg = MessageInner::Include(Box::new(pending_task)).wrap();

        let dispatch = Dispatch::new(msg, &self.sender, receiver);
        Ok(dispatch)
    }

    /// Evaluate `using {module_name}`.
    ///
    /// Safety: `module_name` must be a valid module or package name.
    pub unsafe fn using(&self, module_name: String) -> Dispatch<Message, JlrsResult<()>> {
        let (sender, receiver) = oneshot_channel();
        let pending_task = BlockingTask::new(
            move |mut frame| unsafe {
                let cmd = format!("using {}", module_name);
                Value::eval_string(&mut frame, cmd)
                    .map(|_| ())
                    .into_jlrs_result()
            },
            sender,
        );

        let msg = MessageInner::BlockingTask(Box::new(pending_task)).wrap();
        Dispatch::new(msg, &self.sender, receiver)
    }

    /// Prepare to enable or disable colored error messages originating from Julia.
    ///
    /// This feature is disabled by default and is a global property.
    pub fn error_color(&self, enable: bool) -> Dispatch<Message, ()> {
        let (sender, receiver) = oneshot_channel();
        let pending_task = SetErrorColorTask::new(enable, sender);
        let msg = MessageInner::ErrorColor(Box::new(pending_task)).wrap();

        Dispatch::new(msg, &self.sender, receiver)
    }

    /// The current number of workers in the thread pool.
    pub fn n_workers(&self) -> usize {
        self.n_workers.load(Ordering::Relaxed)
    }

    /// Returns `true` if the handle has been closed.
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    /// Close the backing channel.
    ///
    /// This will shut down the pool. If `cancel` is true, pending messages in the channel will
    /// not be handled, only running tasks will be run to completion.
    pub fn close(&self, cancel: bool) {
        self.sender.close();
        if cancel {
            match self.pool_or_token {
                #[cfg(feature = "multi-rt")]
                #[cfg(not(any(
                    feature = "julia-1-6",
                    feature = "julia-1-7",
                    feature = "julia-1-8"
                )))]
                PoolIdOrToken::PoolId(pool_id) => get_manager().drop_pool(&pool_id),
                PoolIdOrToken::Token(ref token) => token.cancel(),
            }
        }
    }

    pub(crate) unsafe fn new_main(sender: Sender<Message>, token: CancellationToken) -> Self {
        AsyncHandle {
            sender,
            pool_or_token: PoolIdOrToken::Token(token),
            n_workers: Arc::new(AtomicUsize::new(1)),
        }
    }
}

#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
impl AsyncHandle {
    /// Try to add a worker to the pool.
    ///
    /// Returns `false` if the pool has been closed or the pool is running the main runtime
    /// thread.
    pub fn try_add_worker(&self) -> bool {
        if !self.sender.is_closed() {
            match self.pool_or_token {
                PoolIdOrToken::PoolId(pool_id) => {
                    get_manager().add_worker(&pool_id);
                    true
                }
                PoolIdOrToken::Token(_) => false,
            }
        } else {
            false
        }
    }

    /// Try to remove a worker from the pool.
    ///
    /// Returns `false` if the pool has been closed or the pool is running the main runtime
    /// thread. The pool is closed when all workers have been removed.
    pub fn try_remove_worker(&self) -> bool {
        if !self.sender.is_closed() {
            match self.pool_or_token {
                PoolIdOrToken::PoolId(pool_id) => {
                    get_manager().remove_worker(&pool_id);
                    true
                }
                PoolIdOrToken::Token(_) => false,
            }
        } else {
            false
        }
    }

    pub(super) unsafe fn new(
        sender: Sender<Message>,
        pool_id: PoolId,
        n_workers: Arc<AtomicUsize>,
    ) -> Self {
        AsyncHandle {
            sender,
            pool_or_token: PoolIdOrToken::PoolId(pool_id),
            n_workers,
        }
    }
}

impl RequireSendSync for AsyncHandle {}

#[derive(Clone)]
enum PoolIdOrToken {
    #[cfg(feature = "multi-rt")]
    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
    PoolId(PoolId),
    Token(CancellationToken),
}

// Run the async runtime on the main thread, i.e. the thread that initialized Julia.
//
// Because we're running this on the main thread we have to account for other tasks that run on
// this thread. For example, if Julia is started with one thread every task will be spawned on
// this thread. To handle this, we call `Base.sleep` whenever no new tasks can be spawned or the
// task queue is empty.
pub(crate) async unsafe fn on_main_thread<'ctx, R: Executor<N>, const N: usize>(
    receiver: Receiver<Message>,
    token: CancellationToken,
    base_frame: &'ctx mut StackFrame<N>,
) {
    let ptls = get_tls();
    // gc-unsafe: {
    let state = jlrs_gc_unsafe_enter(ptls);
    let base_frame: &'static mut StackFrame<N> = std::mem::transmute(base_frame);
    let mut pinned = base_frame.pin();
    let base_frame = pinned.stack_frame();

    set_custom_fns();
    jlrs_gc_unsafe_leave(ptls, state);
    // }

    let free_stacks = create_free_stacks(N);
    let running_tasks = create_running_tasks::<R, N>();

    let ppgcstack = jlrs_ppgcstack();
    assert!(!ppgcstack.is_null());
    let pgcstack = *ppgcstack;

    loop {
        clear_failed_tasks::<R, N>(&running_tasks, &free_stacks, &base_frame, pgcstack).await;

        if token.is_cancelled() {
            break;
        }

        while free_stacks.borrow().len() == 0 {
            gc_unsafe_with(ptls, |unrooted| sleep(&unrooted, Duration::from_millis(1)));
            R::yield_now().await;
        }

        match receiver.try_recv() {
            Err(TryRecvError::Empty) => {
                gc_unsafe_with(ptls, |unrooted| sleep(&unrooted, Duration::from_millis(1)));
                R::yield_now().await;
            }
            Ok(msg) => match msg.inner {
                MessageInner::Task(task) => {
                    let idx = free_stacks.borrow_mut().pop_front().unwrap();
                    let stack = base_frame.nth_stack(idx);

                    let task = {
                        let free_stacks = free_stacks.clone();
                        let running_tasks = running_tasks.clone();

                        R::spawn_local(GcUnsafeFuture::new(async move {
                            task.call(stack).await;
                            free_stacks.borrow_mut().push_back(idx);
                            running_tasks.borrow_mut()[idx] = None;
                        }))
                    };

                    running_tasks.borrow_mut()[idx] = Some(task);
                }
                MessageInner::BlockingTask(task) => {
                    let stack = base_frame.sync_stack();
                    gc_unsafe_with(ptls, |_| task.call(stack))
                }
                MessageInner::Include(task) => {
                    let stack = base_frame.sync_stack();
                    gc_unsafe_with(ptls, |_| task.call(stack))
                }
                MessageInner::ErrorColor(task) => {
                    let stack = base_frame.sync_stack();
                    gc_unsafe_with(ptls, |_| task.call(stack))
                }
            },
            _ => break,
        }
    }

    for i in 0..N {
        while running_tasks.borrow()[i].is_some() {
            gc_unsafe_with(ptls, |unrooted| sleep(&unrooted, Duration::from_millis(1)));
            R::yield_now().await;
        }
    }

    ::std::mem::drop(pinned);
}

// Run the async runtime on an adopted thread.
//
// Because we're running this on an adopted thread we don't have to account for other tasks that
// run on this thread because Julia doesn't schedule tasks on adopted threads. This means we don't
// have to call `Base.sleep` but can use `async`/`.await` instead.
//
// The thread must be in the GC-safe state when this function is called.
#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
pub(super) async unsafe fn on_adopted_thread<'ctx, R: Executor<N>, const N: usize>(
    receiver: Receiver<Message>,
    token: CancellationToken,
    base_frame: &'ctx mut StackFrame<N>,
) {
    let _: () = R::VALID;
    let ptls = get_tls();
    // gc-unsafe: {
    let state = jlrs_gc_unsafe_enter(ptls);
    let base_frame: &'static mut StackFrame<N> = std::mem::transmute(base_frame);
    let mut pinned = base_frame.pin();
    let base_frame = pinned.stack_frame();

    set_custom_fns();
    jlrs_gc_unsafe_leave(ptls, state);
    // }

    let free_stacks = create_free_stacks(N);
    let running_tasks = create_running_tasks::<R, N>();

    jl_sys::jl_enter_threaded_region();

    let task_complete_state = TaskCompleteState::new();
    let task_complete = TaskComplete::new(&task_complete_state);

    let ppgcstack = jlrs_ppgcstack();
    assert!(!ppgcstack.is_null());
    let pgcstack = *ppgcstack;

    loop {
        // If a task has finished but is still in the list, it panicked or was cancelled.
        clear_failed_tasks::<R, N>(&running_tasks, &free_stacks, &base_frame, pgcstack).await;

        if token.is_cancelled() {
            break;
        }

        if free_stacks.borrow().len() == 0 {
            task_complete.clear().await;
        }

        match R::timeout(Duration::from_millis(1), receiver.recv()).await {
            None => (),
            Some(Ok(msg)) => match msg.inner {
                MessageInner::Task(task) => {
                    let idx = free_stacks.borrow_mut().pop_front().unwrap();
                    let stack = base_frame.nth_stack(idx);

                    let task = {
                        let free_stacks = free_stacks.clone();
                        let running_tasks = running_tasks.clone();
                        let task_complete_state = task_complete_state.clone();

                        R::spawn_local(GcUnsafeFuture::new(async move {
                            task.call(stack).await;
                            free_stacks.borrow_mut().push_back(idx);
                            running_tasks.borrow_mut()[idx] = None;
                            task_complete_state.complete();
                        }))
                    };

                    running_tasks.borrow_mut()[idx] = Some(task);
                }
                MessageInner::BlockingTask(task) => {
                    let stack = base_frame.sync_stack();
                    gc_unsafe_with(ptls, |_| task.call(stack))
                }
                MessageInner::Include(task) => {
                    let stack = base_frame.sync_stack();
                    gc_unsafe_with(ptls, |_| task.call(stack))
                }
                MessageInner::ErrorColor(task) => {
                    let stack = base_frame.sync_stack();
                    gc_unsafe_with(ptls, |_| task.call(stack))
                }
            },
            Some(Err(_)) => break,
        }
    }

    for i in 0..N {
        if let Some(task) = running_tasks.borrow_mut()[i].take() {
            task.await.ok();
        }
    }

    jl_sys::jl_exit_threaded_region();

    ::std::mem::drop(pinned);
}

fn create_free_stacks(n: usize) -> Rc<RefCell<VecDeque<usize>>> {
    let mut free_stacks = VecDeque::with_capacity(n);
    for i in 0..n {
        free_stacks.push_back(i);
    }

    Rc::new(RefCell::new(free_stacks))
}

fn create_running_tasks<R: Executor<N>, const N: usize>(
) -> Rc<RefCell<Box<[Option<R::JoinHandle>]>>> {
    let mut running_tasks = Vec::with_capacity(N);
    for _ in 0..N {
        running_tasks.push(None);
    }

    Rc::new(RefCell::new(running_tasks.into_boxed_slice()))
}

async unsafe fn clear_failed_tasks<R: Executor<N>, const N: usize>(
    running_tasks: &Rc<RefCell<Box<[Option<<R as Executor<N>>::JoinHandle>]>>>,
    free_stacks: &Rc<RefCell<VecDeque<usize>>>,
    stacks: &JlrsStackFrame<'_, '_, N>,
    pgcstack: *mut jl_gcframe_t,
) {
    let mut cleared = false;
    for (idx, handle) in running_tasks
        .borrow_mut()
        .iter_mut()
        .enumerate()
        .filter(|(_, h)| h.is_some())
    {
        if handle.as_ref().unwrap_unchecked().is_finished() {
            if let Err(_e) = handle.take().unwrap().await {
                stacks.nth_stack(idx).pop_roots(0);
                free_stacks.borrow_mut().push_back(idx);
                cleared = true;
                // restore_gc_stack();
            }
        }
    }

    if cleared {
        let ppgcstack = jlrs_ppgcstack();
        let gcstack_ref = NonNull::new_unchecked(ppgcstack).as_mut();
        *gcstack_ref = pgcstack;
    }
}

// TODO: Atomic
unsafe fn set_custom_fns() {
    unsafe {
        let handle = weak_handle_unchecked!();

        let cmd = CStr::from_bytes_with_nul_unchecked(b"const JlrsThreads = JlrsCore.Threads\0");
        handle.local_scope::<_, 2>(|mut frame| {
            Value::eval_cstring(&mut frame, cmd).expect("using JlrsCore threw an exception");

            let wake_rust = Value::new(&mut frame, wake_task as *mut c_void);
            Module::main(&frame)
                .submodule(&frame, "JlrsThreads")
                .unwrap()
                .as_managed()
                .global(&frame, "wakerust")
                .unwrap()
                .as_managed()
                .set_nth_field_unchecked(0, wake_rust);
        })
    }
}
