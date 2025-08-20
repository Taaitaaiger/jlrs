//! A handle that lets you call directly into Julia from arbitrary threads.

#[cfg(feature = "async")]
use std::num::NonZeroUsize;
use std::{
    cell::Cell,
    marker::PhantomData,
    pin::Pin,
    sync::atomic::AtomicUsize,
    thread::{Scope, ScopedJoinHandle},
};

use atomic::Ordering;
use jl_sys::{jl_adopt_thread, jl_atexit_hook, jlrs_gc_safe_enter, jlrs_ptls_from_gcstack};
use parking_lot::{Condvar, Mutex};

#[cfg(feature = "async")]
use self::manager::get_manager;
#[cfg(feature = "async")]
use super::async_handle::AsyncHandle;
use super::{notify, weak_handle::WeakHandle, IsActive};
#[cfg(feature = "async")]
use crate::runtime::executor::Executor;
use crate::{
    call::Call,
    data::managed::module::JlrsCore,
    error::CANNOT_DISPLAY_VALUE,
    memory::{gc::gc_unsafe, get_tls},
    prelude::{LocalScope, Managed},
    runtime::state::{set_exit, set_pending_exit},
    weak_handle_unchecked,
};

#[cfg(feature = "async")]
pub(super) mod manager;

thread_local! {
    static ADOPTED: Cell<bool> = Cell::new(false);
}

pub(super) static N_HANDLES: AtomicUsize = AtomicUsize::new(0);
pub(crate) static EXIT_LOCK: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());

/// A handle that lets you call into Julia from arbitrary threads.
///
/// An initial `MtHandle` can be created by calling [`Builder::start_mt`]. Julia exits when the
/// all handles have been dropped.
///
/// [`Builder::start_mt`]: crate::runtime::builder::Builder::start_mt
pub struct MtHandle<'scope, 'env> {
    _marker: PhantomData<&'scope mut &'scope ()>,
    scope: &'scope Scope<'scope, 'env>,
}

impl<'scope, 'env> MtHandle<'scope, 'env> {
    /// Prepares the environment to enable calling into Julia and calls `func`.
    pub fn with<T, F>(&mut self, func: F) -> T
    where
        for<'ctx> F: FnOnce(ActiveHandle<'ctx>) -> T,
    {
        unsafe {
            if !ADOPTED.get() {
                adopt_thread();
            }

            gc_unsafe(|_| {
                let mut weak = weak_handle_unchecked!();
                func(ActiveHandle::new(&mut weak))
            })
        }
    }

    pub fn spawn<T>(&self, f: impl FnOnce(Self) -> T + Send + 'scope) -> ScopedJoinHandle<'scope, T>
    where
        T: Send + 'scope,
    {
        let s = self.clone();
        self.scope.spawn(|| f(s))
    }

    pub(crate) unsafe fn new(scope: &'scope Scope<'scope, 'env>) -> Self {
        N_HANDLES.fetch_add(1, Ordering::Relaxed);
        MtHandle {
            _marker: PhantomData,
            scope,
        }
    }
}

#[cfg(feature = "async")]
impl<'scope, 'env> MtHandle<'scope, 'env> {
    /// Returns a builder for a new thread pool.
    pub fn pool_builder<'a, E: Executor<N>, const N: usize>(
        &'a self,
        executor_opts: E,
    ) -> PoolBuilder<'a, 'scope, 'env, E, N> {
        let _: () = E::VALID;
        PoolBuilder::new(self, executor_opts)
    }
}

unsafe impl<'scope, 'env> Send for MtHandle<'scope, 'env> {}

impl<'scope, 'env> Clone for MtHandle<'scope, 'env> {
    fn clone(&self) -> Self {
        N_HANDLES.fetch_add(1, Ordering::Relaxed);
        Self {
            _marker: PhantomData,
            scope: self.scope,
        }
    }
}

impl<'scope, 'env> Drop for MtHandle<'scope, 'env> {
    fn drop(&mut self) {
        unsafe { drop_handle() }
    }
}

/// An active handle to the current thread.
///
/// An [`MtHandle`] existing
pub struct ActiveHandle<'ctx> {
    _weak: PhantomData<&'ctx mut Pin<&'ctx mut WeakHandle>>,
}

impl<'ctx> ActiveHandle<'ctx> {
    unsafe fn new(_weak: &'ctx mut Pin<&'ctx mut WeakHandle>) -> Self {
        ActiveHandle { _weak: PhantomData }
    }
}

impl IsActive for ActiveHandle<'_> {}

/// Thread pool builder
#[cfg(feature = "async-rt")]
pub struct PoolBuilder<'a, 'scope, 'env, E: Executor<N>, const N: usize> {
    _handle: PhantomData<&'a MtHandle<'scope, 'env>>,
    executor_opts: E,
    channel_capacity: usize,
    n_workers: NonZeroUsize,
    prefix: Option<String>,
}

#[cfg(feature = "async-rt")]
impl<'a, 'scope, 'env, E: Executor<N>, const N: usize> PoolBuilder<'a, 'scope, 'env, E, N> {
    fn new(_handle: &'a MtHandle, executor_opts: E) -> Self {
        PoolBuilder {
            _handle: PhantomData,
            executor_opts,
            channel_capacity: 0,
            n_workers: unsafe { NonZeroUsize::new_unchecked(1) },
            prefix: None,
        }
    }

    /// Set the capacity of the channel used to communicate with this pool.
    ///
    /// The default value is 0, i.e. unbounded.
    #[inline]
    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        self
    }

    /// Set the number of worker threads in this pool.
    ///
    /// The default value is 1.
    #[inline]
    pub fn n_workers(mut self, n_workers: NonZeroUsize) -> Self {
        self.n_workers = n_workers;
        self
    }

    /// Set the worker name prefix.
    #[inline]
    pub fn prefix(mut self, prefix: String) -> Self {
        self.prefix = Some(prefix);
        self
    }

    /// Spawn the thread pool.
    pub fn spawn(self) -> AsyncHandle {
        N_HANDLES.fetch_add(1, Ordering::Relaxed);
        get_manager().spawn_pool(
            self.executor_opts,
            self.channel_capacity,
            self.n_workers.get(),
            self.prefix,
        )
    }
}

#[inline(never)]
#[cold]
unsafe fn adopt_thread() {
    let mut ptls = get_tls();
    if ptls.is_null() {
        let pgcstack = jl_adopt_thread();
        ptls = jlrs_ptls_from_gcstack(pgcstack);
    }

    jlrs_gc_safe_enter(ptls);
    ADOPTED.set(true);
}

pub(crate) fn wait_loop() {
    unsafe {
        let weak_handle = weak_handle_unchecked!();
        let wait_main = JlrsCore::wait_main(&weak_handle);

        // Start waiting
        if let Err(err) = wait_main.call(&weak_handle, []) {
            let err = weak_handle.local_scope::<_, 1>(|mut frame| {
                err.root(&mut frame).error_string_or(CANNOT_DISPLAY_VALUE)
            });

            set_exit();
            jl_atexit_hook(1);
            panic!("unexpected error in JlrsCore.Threads.wait_main: {}", err);
        }
    }
}

unsafe fn drop_handle() {
    let n_handles = N_HANDLES.fetch_sub(1, Ordering::Relaxed);
    if n_handles == 1 {
        let _ = std::thread::spawn(|| {
            let pgcstack = jl_adopt_thread();
            let ptls = jlrs_ptls_from_gcstack(pgcstack);

            set_pending_exit();

            let weak_handle = weak_handle_unchecked!();
            let notify_main = JlrsCore::notify_main(&weak_handle);

            if let Err(err) = notify_main.call(&weak_handle, []) {
                weak_handle.local_scope::<_, 1>(|mut frame| {
                    panic!(
                        "unexpected error when calling JlrsCore.Threads.notify_main: {:?}",
                        err.root(&mut frame)
                    );
                });
            }

            jlrs_gc_safe_enter(ptls);
            notify(&EXIT_LOCK);
        })
        .join();
    }
}
