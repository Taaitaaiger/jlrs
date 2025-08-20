//! Runtime configuration.
//!
//! Before Julia can be used it must be initialized. The builders provided by this module must be
//! used to initialize Julia and set custom parameters. The [`Builder`] only lets you
//! provide a custom system image, [`AsyncBuilder`] provides additional methods to set the
//! number of threads available to Julia among others.

#[cfg(feature = "async-rt")]
pub mod async_builder;

use std::{
    ffi::CString,
    path::{Path, PathBuf},
};

#[cfg(feature = "async-rt")]
pub use async_builder::*;
use jl_sys::{
    jl_init, jl_init_with_image, jlrs_set_nthreadpools, jlrs_set_nthreads,
    jlrs_set_nthreads_per_pool,
};

#[cfg(any(feature = "multi-rt", feature = "local-rt"))]
use crate::error::JlrsResult;
#[cfg(feature = "async-rt")]
use crate::runtime::executor::Executor;
#[cfg(feature = "local-rt")]
use crate::runtime::handle::local_handle::LocalHandle;
#[cfg(feature = "multi-rt")]
use crate::runtime::handle::mt_handle::MtHandle;
use crate::{InstallJlrsCore, init_jlrs};

/// Build a runtime.
///
/// With this builder you can set a custom system image by calling [`Builder::image`],
/// the builder can be upgraded to an [`AsyncBuilder`] by calling
/// [`Builder::async_runtime`] and providing a backing runtime.
pub struct Builder {
    pub(crate) image: Option<(PathBuf, PathBuf)>,
    pub(crate) install_jlrs_core: InstallJlrsCore,
    pub(crate) n_threads: usize,
    pub(crate) n_threadsi: usize,
}

impl Builder {
    /// Create a new builder.
    ///
    /// The default options are: no custom system image, install JlrsCore if it is unavailable,
    /// and don't start any additional threads.
    pub const fn new() -> Self {
        Builder {
            image: None,
            install_jlrs_core: InstallJlrsCore::Default,
            n_threads: 0,
            n_threadsi: 0,
        }
    }

    #[cfg(feature = "local-rt")]
    #[inline]
    /// Initialize Julia on the current thread.
    pub fn start_local(self) -> JlrsResult<LocalHandle> {
        use crate::{error::RuntimeError, runtime::state::can_init};

        if !can_init() {
            Err(RuntimeError::AlreadyInitialized)?;
        }

        unsafe {
            init_runtime(&self);
            Ok(LocalHandle::new())
        }
    }

    /// Start the multithreaded runtime from the current thread.
    ///
    /// A new thread is spawned which calls `func`. Julia will remain enabled until `func`
    /// returns.
    #[inline]
    #[cfg(feature = "multi-rt")]
    pub fn start_mt<'env, T: 'static + Send, F>(self, func: F) -> JlrsResult<T>
    where
        F: 'env + for<'scope> FnOnce(MtHandle<'scope, 'env>) -> T + Send,
    {
        mt_impl::sync_impl::start(self, func)
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
        self.n_threads = n;
        self
    }

    /// Set the number of threads allocated to the `:interactive` pool.
    ///
    /// If it's set to 0, the default value, no threads are allocated to this pool.
    #[inline]
    pub const fn n_interactive_threads(mut self, n: usize) -> Self {
        self.n_threadsi = n;
        self
    }

    /// Use a custom system image.
    ///
    /// You must provide two arguments to use a custom system image, `julia_bindir` and
    /// `image_path`. The first is the absolute path to a directory that contains a
    /// compatible Julia binary (eg `${JLRS_JULIA_DIR}/bin`), the second is the path to a
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

        self.image = Some((
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
        self.install_jlrs_core = install;
        self
    }

    /// Upgrade this builder to an [`AsyncBuilder`].
    ///
    /// You must provide an executor, jlrs supports using tokio if the `tokio-rt` feature is
    /// enabled:
    ///
    /// ```
    /// use jlrs::prelude::*;
    ///
    /// # fn main() {
    /// let (_julia, _thread_handle) = unsafe {
    ///     Builder::new()
    ///         .async_runtime(Tokio::<1>::new(false))
    ///         .spawn()
    ///         .expect("Could not start Julia")
    /// };
    /// # }
    /// ```
    #[cfg(feature = "async-rt")]
    #[inline]
    pub fn async_runtime<E: Executor<N>, const N: usize>(
        self,
        executor_opts: E,
    ) -> AsyncBuilder<E, N> {
        let _: () = E::VALID;
        AsyncBuilder::new(self, executor_opts)
    }
}

#[cfg(feature = "multi-rt")]
mod mt_impl {
    pub(super) mod sync_impl {
        use std::thread;

        use jl_sys::jl_atexit_hook;

        use crate::{
            error::{JlrsError, RuntimeError},
            memory::gc::gc_safe,
            prelude::JlrsResult,
            runtime::{
                builder::{Builder, init_runtime},
                handle::{
                    mt_handle::{EXIT_LOCK, MtHandle, wait_loop},
                    wait,
                },
                state::{can_init, set_exit},
            },
        };

        pub(crate) fn start<'env, T, F>(options: Builder, func: F) -> JlrsResult<T>
        where
            T: Send + 'static,
            F: 'env + for<'scope> FnOnce(MtHandle<'scope, 'env>) -> T + Send,
        {
            if !can_init() {
                Err(RuntimeError::AlreadyInitialized)?;
            }

            unsafe {
                init_runtime(&options);
            }

            let ret = thread::scope(|scope| {
                let handle = scope.spawn(|| unsafe {
                    thread::scope(|scope| {
                        let handle = MtHandle::new(scope);
                        func(handle)
                    })
                });

                unsafe {
                    wait_loop();

                    let th_res = handle.join();

                    // Returned from wait_main, so we're about to exit Julia becuase all handles have
                    // been dropped. Next we need to wait until we've returned from `notify_main` too.
                    gc_safe(|| wait(&EXIT_LOCK));
                    set_exit();
                    jl_atexit_hook(0);

                    th_res
                }
            });

            match ret {
                Ok(ret) => Ok(ret),
                Err(e) => Err(JlrsError::exception(format!("{e:?}")))?,
            }
        }
    }
}

unsafe fn init_runtime(options: &Builder) {
    unsafe {
        set_n_threads(options);
        init_julia(options);
        init_jlrs(&options.install_jlrs_core, true);
    }
}

unsafe fn init_julia(options: &Builder) {
    unsafe {
        if let Some((bin_dir, image_path)) = options.image.as_ref() {
            let julia_bindir_str = bin_dir.as_os_str().as_encoded_bytes();
            let image_path_str = image_path.as_os_str().as_encoded_bytes();

            let bindir = CString::new(julia_bindir_str).unwrap();
            let im_rel_path = CString::new(image_path_str).unwrap();

            jl_init_with_image(bindir.as_ptr(), im_rel_path.as_ptr())
        } else {
            jl_init();
        }
    }
}

unsafe fn set_n_threads(options: &Builder) {
    unsafe {
        if options.n_threadsi != 0 {
            if options.n_threads == 0 {
                jlrs_set_nthreads(-1);
                jlrs_set_nthreadpools(2);
                let perthread = Box::new([-1i16, options.n_threadsi as _]);
                jlrs_set_nthreads_per_pool(Box::leak(perthread) as *const _);
            } else {
                let nthreads = options.n_threads as i16;
                let nthreadsi = options.n_threadsi as i16;
                jlrs_set_nthreads(nthreads + nthreadsi);
                jlrs_set_nthreadpools(2);
                let perthread = Box::new([nthreads, options.n_threadsi as _]);
                jlrs_set_nthreads_per_pool(Box::leak(perthread) as *const _);
            }
        } else if options.n_threads == 0 {
            jlrs_set_nthreads(-1);
            jlrs_set_nthreadpools(1);
            let perthread = Box::new(-1i16);
            jlrs_set_nthreads_per_pool(Box::leak(perthread) as *const _);
        } else {
            let n_threads = options.n_threads as _;
            jlrs_set_nthreads(n_threads);
            jlrs_set_nthreadpools(1);
            let perthread = Box::new(n_threads);
            jlrs_set_nthreads_per_pool(Box::leak(perthread) as *const _);
        }
    }
}
