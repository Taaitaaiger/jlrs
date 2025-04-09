//! Async runtime configuration.

use std::{path::Path, thread, thread::JoinHandle};

use async_channel::{bounded, unbounded};
use jl_sys::jlrs_gc_safe_enter;

#[cfg(feature = "multi-rt")]
use crate::runtime::handle::mt_handle::MtHandle;
use crate::{
    error::{JlrsError, RuntimeError},
    memory::{get_tls, stack_frame::StackFrame},
    prelude::JlrsResult,
    runtime::{
        builder::{init_runtime, Builder},
        executor::Executor,
        handle::async_handle::{
            cancellation_token::CancellationToken, on_main_thread, AsyncHandle,
        },
        state::{can_init, set_exit},
    },
    InstallJlrsCore,
};

/// A `Builder` for an async runtime.
pub struct AsyncBuilder<E: Executor<N>, const N: usize> {
    builder: Builder,
    channel_capacity: usize,
    executor_opts: E,
}

impl<E: Executor<N>, const N: usize> AsyncBuilder<E, N> {
    #[inline]
    pub(super) fn new(builder: Builder, executor_opts: E) -> Self {
        AsyncBuilder {
            builder,
            channel_capacity: 0,
            executor_opts,
        }
    }

    /// Start the async runtime on a new thread.
    #[inline]
    pub fn spawn(self) -> JlrsResult<(AsyncHandle, JoinHandle<()>)> {
        spawn_main(self.builder, self.executor_opts, self.channel_capacity)
    }

    /// Start the async runtime on the current thread.
    ///
    /// A new thread is spawned which calls `func`.
    #[inline]
    pub fn start<T: 'static + Send>(
        self,
        func: impl 'static + Send + FnOnce(AsyncHandle) -> T,
    ) -> JlrsResult<T> {
        run_main(
            self.builder,
            self.executor_opts,
            self.channel_capacity,
            func,
        )
    }

    /// Start the multithreaded runtime as an async runtime on the current thread.
    ///
    /// Like `AsyncBuilder::start`, a new thread is spawned which calls `func`. The `AsyncHandle`
    /// can be used to execute tasks on this main thread.
    #[inline]
    #[cfg(feature = "multi-rt")]
    pub fn start_mt<'env, T: 'static + Send, F>(self, func: F) -> JlrsResult<T>
    where
        F: 'env + Send + for<'scope> FnOnce(MtHandle<'scope, 'env>, AsyncHandle) -> T,
    {
        mt_impl::run_main_mt(
            self.builder,
            self.executor_opts,
            self.channel_capacity,
            func,
        )
    }

    /// Set the capacity of the channel used to communicate with the async runtime.
    ///
    /// The default value is 0, i.e. unbounded.
    #[inline]
    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        self
    }

    /// Set the number of threads Julia can use.
    ///
    /// If it's set to 0, the default value, the number of threads is the number of CPU
    /// cores.
    ///
    /// NB: When the `nightly` or `beta` feature is enabled, this sets the number of
    /// threads allocated to the `:default` pool.
    #[inline]
    pub const fn n_threads(mut self, n: usize) -> Self {
        self.builder.n_threads = n;
        self
    }

    /// Set the number of threads allocated to the `:interactive` pool.
    ///
    /// If it's set to 0, the default value, no threads are allocated to this pool.
    #[inline]
    pub const fn n_interactive_threads(mut self, n: usize) -> Self {
        self.builder.n_threadsi = n;
        self
    }

    /// Use a custom system image.
    ///
    /// You must provide two arguments to use a custom system image, `julia_bindir` and
    /// `image_path`. The first is the absolute path to a directory that contains a
    /// compatible Julia binary (eg `${JULIA_DIR}/bin`), the second is the path to a
    /// system image.
    ///
    /// A custom system image can be created with [`PackageCompiler`].
    ///
    /// Returns an error if either of the paths does not exist.
    ///
    /// Safety: using a custom system image can cause additional, unchecked code to be executed.
    ///
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl
    #[inline]
    pub unsafe fn image<P, Q>(mut self, julia_bindir: P, image_path: Q) -> Result<Self, Self>
    where
        P: AsRef<Path> + Send + 'static,
        Q: AsRef<Path> + Send + 'static,
    {
        if !julia_bindir.as_ref().exists() {
            return Err(self);
        }

        if !image_path.as_ref().exists() {
            return Err(self);
        }

        self.builder.image = Some((
            julia_bindir.as_ref().to_path_buf(),
            image_path.as_ref().to_path_buf(),
        ));

        Ok(self)
    }

    /// Enable or disable automatically installing JlrsCore.
    ///
    /// jlrs requires that the JlrsCore package is installed. By default, this package is
    /// installed automatically if it is unavailable. The configured behavior can be overridden
    /// with the following environment variables:
    ///
    /// - `JLRS_CORE_VERSION=major.minor.patch`
    /// Installs the set version of JlrsCore before loading it.
    ///
    /// - `JLRS_CORE_REVISION=rev`
    /// Installs the set revision of JlrsCore before loading it.
    ///
    /// - `JLRS_CORE_REPO=repo-url`
    /// Can be used with `JLRS_CORE_REVISION` to set the repository JlrsCore will be downloaded
    /// from.
    ///
    /// - `JLRS_CORE_NO_INSTALL=...`
    /// Don't install JlrsCore, its value is ignored.
    ///
    /// `JLRS_CORE_NO_INSTALL` takes priority over `JLRS_CORE_REVISION`, which takes priority over
    ///  `JLRS_CORE_VERSION`.
    #[inline]
    pub fn install_jlrs(mut self, install: InstallJlrsCore) -> Self {
        self.builder.install_jlrs_core = install;
        self
    }
}

pub(crate) fn spawn_main<R: Executor<N>, const N: usize>(
    builder: Builder,
    executor_opts: R,
    channel_capacity: usize,
) -> JlrsResult<(AsyncHandle, JoinHandle<()>)> {
    if !can_init() {
        Err(RuntimeError::AlreadyInitialized)?;
    }

    let token = CancellationToken::new();
    let t2 = token.clone();
    let (sender, receiver) = if channel_capacity == 0 {
        unbounded()
    } else {
        bounded(channel_capacity)
    };

    let thread_handle = std::thread::spawn(move || unsafe {
        init_runtime(&builder);

        let ptls = get_tls();
        jlrs_gc_safe_enter(ptls);

        let mut base_frame = StackFrame::<N>::new_n();
        executor_opts.block_on(on_main_thread::<R, N>(receiver, token, &mut base_frame));

        set_exit();
    });

    unsafe {
        let handle = AsyncHandle::new_main(sender, t2);
        Ok((handle, thread_handle))
    }
}

pub(crate) fn run_main<T: 'static + Send, R: Executor<N>, const N: usize>(
    builder: Builder,
    executor_opts: R,
    channel_capacity: usize,
    func: impl 'static + Send + FnOnce(AsyncHandle) -> T,
) -> JlrsResult<T> {
    if !can_init() {
        Err(RuntimeError::AlreadyInitialized)?;
    }

    unsafe {
        init_runtime(&builder);

        let token = CancellationToken::new();
        let t2 = token.clone();
        let (sender, receiver) = if channel_capacity == 0 {
            unbounded()
        } else {
            bounded(channel_capacity)
        };

        let handle = AsyncHandle::new_main(sender, t2);

        let ptls = get_tls();
        jlrs_gc_safe_enter(ptls);

        let handle = thread::spawn(move || func(handle));

        let mut base_frame = StackFrame::<N>::new_n();
        executor_opts.block_on(on_main_thread::<R, N>(receiver, token, &mut base_frame));

        set_exit();

        handle
            .join()
            .map_err(|_| Box::new(JlrsError::exception("thread panicked")))
    }
}

#[cfg(feature = "multi-rt")]
mod mt_impl {
    use std::{
        panic::{catch_unwind, AssertUnwindSafe},
        thread,
    };

    use jl_sys::jl_atexit_hook;

    use super::super::{init_runtime, Builder};
    use crate::{
        error::{JlrsError, RuntimeError},
        memory::{gc::gc_safe, stack_frame::StackFrame},
        prelude::JlrsResult,
        runtime::{
            executor::Executor,
            handle::{
                async_handle::{
                    cancellation_token::CancellationToken, channel::channel, on_main_thread,
                    AsyncHandle,
                },
                mt_handle::{wait_loop, MtHandle, EXIT_LOCK},
                wait,
            },
            state::{can_init, set_exit},
        },
    };

    pub(crate) fn run_main_mt<'env, T, E, F, const N: usize>(
        options: Builder,
        executor_opts: E,
        channel_capacity: usize,
        func: F,
    ) -> JlrsResult<T>
    where
        T: Send + 'static,
        E: Executor<N>,
        F: 'env + Send + for<'scope> FnOnce(MtHandle<'scope, 'env>, AsyncHandle) -> T,
    {
        if !can_init() {
            Err(RuntimeError::AlreadyInitialized)?;
        }

        let token = CancellationToken::new();
        let t2 = token.clone();
        let (sender, receiver) = channel(channel_capacity);

        unsafe {
            init_runtime(&options);
        }

        let async_handle = unsafe { AsyncHandle::new_main(sender, t2) };

        let ret = thread::scope(|scope| {
            let handle = scope.spawn(|| unsafe {
                thread::scope(|scope| {
                    let handle = MtHandle::new(scope);
                    func(handle, async_handle)
                })
            });

            unsafe {
                let mut base_frame = StackFrame::<N>::new_n();
                let res = catch_unwind(AssertUnwindSafe(|| {
                    executor_opts.block_on(on_main_thread::<E, N>(
                        receiver,
                        token,
                        &mut base_frame,
                    ));
                }));

                wait_loop();

                // The spawned thread may have been adopted, so join it before
                // calling jl_atexit_hook
                let th_res = handle.join();

                match res {
                    Ok(_) => {
                        // Returned from wait_main, so we're about to exit Julia becuase all handles have
                        // been dropped. Next we need to wait until we've returned from `notify_main` too.
                        gc_safe(|| wait(&EXIT_LOCK));
                        set_exit();
                        jl_atexit_hook(0);
                    }
                    Err(_) => {
                        set_exit();
                        jl_atexit_hook(1);
                    }
                }

                th_res
            }
        });

        match ret {
            Ok(ret) => Ok(ret),
            Err(e) => Err(JlrsError::exception(format!("{e:?}")))?,
        }
    }
}
