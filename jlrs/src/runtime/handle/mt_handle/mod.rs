//! A handle that lets you call directly into Julia from arbitrary threads.

#[cfg(feature = "async")]
use std::num::NonZeroUsize;
use std::{cell::Cell, marker::PhantomData, path::Path, pin::Pin, sync::atomic::AtomicUsize};

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
    convert::into_jlrs_result::IntoJlrsResult,
    data::managed::module::{JlrsCore, Main},
    error::{IOError, CANNOT_DISPLAY_VALUE},
    memory::{gc::gc_unsafe, get_tls, scope::LocalReturning},
    prelude::{JlrsResult, JuliaString, LocalScope, Managed, Value},
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
/// An initial `MtHandle` can be created by calling [`Builder::start_mt`] or
/// [`Builder::spawn_mt`]. Julia exits when the all handles have been dropped.
///
/// [`Builder::start_mt`]: crate::runtime::builder::Builder::start_mt
/// [`Builder::spawn_mt`]: crate::runtime::builder::Builder::spawn_mt
pub struct MtHandle {
    _marker: PhantomData<*mut ()>,
}

impl MtHandle {
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

    pub(crate) unsafe fn new() -> Self {
        N_HANDLES.fetch_add(1, Ordering::Relaxed);
        MtHandle {
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "async")]
impl MtHandle {
    /// Returns a builder for a new thread pool.
    pub fn pool_builder<'a, E: Executor<N>, const N: usize>(
        &'a self,
        executor_opts: E,
    ) -> PoolBuilder<'a, E, N> {
        let _: () = E::VALID;
        PoolBuilder::new(self, executor_opts)
    }
}

unsafe impl Send for MtHandle {}

impl Clone for MtHandle {
    fn clone(&self) -> Self {
        N_HANDLES.fetch_add(1, Ordering::Relaxed);
        Self {
            _marker: PhantomData,
        }
    }
}

impl Drop for MtHandle {
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

    /// Calls `include` in the `Main` module in Julia, which executes the file's contents in that
    /// module. This has the same effect as calling `include` in the Julia REPL.
    ///
    /// This is unsafe because the content of the file is evaluated.
    ///
    /// Example:
    ///
    /// ```no_run
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let (mut julia, th_handle) = Builder::new().spawn_mt().unwrap();
    /// julia.with(|handle| unsafe {
    ///     handle.include("Path/To/MyJuliaCode.jl").unwrap();
    /// });
    /// # std::mem::drop(julia);
    /// # th_handle.join().unwrap();
    /// # }
    /// ```
    pub unsafe fn include<P: AsRef<Path>>(&self, path: P) -> JlrsResult<()> {
        if path.as_ref().exists() {
            return self.local_scope::<_, 2>(|mut frame| {
                let path_jl_str = JuliaString::new(&mut frame, path.as_ref().to_string_lossy());
                Main::include(&frame)
                    .call1(&mut frame, path_jl_str.as_value())
                    .into_jlrs_result()
                    .map(|_| ())
            });
        }

        Err(IOError::NotFound {
            path: path.as_ref().to_string_lossy().into(),
        })?
    }

    /// Evaluate `using {module_name}`.
    ///
    /// Safety: `module_name` must be a valid module or package name.
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let (mut julia, th_handle) = Builder::new().spawn_mt().unwrap();
    /// julia.with(|handle| unsafe {
    ///     handle.using("LinearAlgebra").unwrap();
    /// });
    /// # std::mem::drop(julia);
    /// # th_handle.join().unwrap();
    /// # }
    /// ```
    pub unsafe fn using<S: AsRef<str>>(&self, module_name: S) -> JlrsResult<()> {
        return self.local_scope::<_, 1>(|mut frame| {
            let cmd = format!("using {}", module_name.as_ref());
            Value::eval_string(&mut frame, cmd)
                .map(|_| ())
                .into_jlrs_result()
        });
    }
}

impl IsActive for ActiveHandle<'_> {}

impl<'ctx> LocalReturning<'ctx> for ActiveHandle<'ctx> {
    fn returning<T>(&mut self) -> &mut impl LocalScope<'ctx, T> {
        self
    }
}

/// Thread pool builder
#[cfg(feature = "async-rt")]
pub struct PoolBuilder<'a, E: Executor<N>, const N: usize> {
    _handle: PhantomData<&'a MtHandle>,
    executor_opts: E,
    channel_capacity: usize,
    n_workers: NonZeroUsize,
    prefix: Option<String>,
}

#[cfg(feature = "async-rt")]
impl<'a, E: Executor<N>, const N: usize> PoolBuilder<'a, E, N> {
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
        if let Err(err) = wait_main.call0(&weak_handle) {
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

            if let Err(err) = notify_main.call0(&weak_handle) {
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
