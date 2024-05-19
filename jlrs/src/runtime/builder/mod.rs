//! Build a runtime.
//!
//! Before Julia can be used it must be initialized. The builders provided by this module must be
//! used to initialize Julia and set custom parameters. The [`Builder`] only lets you
//! provide a custom system image, [`AsyncBuilder`] provides additional methods to set the
//! number of threads available to Julia among others.

#[cfg(feature = "async-rt")]
pub mod async_builder;

#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
use std::thread::JoinHandle;
use std::{
    ffi::CString,
    path::{Path, PathBuf},
};

#[cfg(feature = "async-rt")]
pub use async_builder::*;
use jl_sys::{jl_init, jl_init_with_image, jlrs_set_nthreads};
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
use jl_sys::{jlrs_set_nthreadpools, jlrs_set_nthreads_per_pool};

#[cfg(any(feature = "multi-rt", feature = "local-rt"))]
use crate::error::JlrsResult;
#[cfg(feature = "async-rt")]
use crate::runtime::executor::Executor;
#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
use crate::runtime::handle::mt_handle::MtHandle;
#[cfg(feature = "local-rt")]
use crate::runtime::{handle::local_handle::LocalHandle, sync_rt::PendingJulia};
use crate::{init_jlrs, InstallJlrsCore};

/// Build a runtime.
///
/// With this builder you can set a custom system image by calling [`Builder::image`],
/// the builder can be upgraded to an [`AsyncBuilder`] by calling
/// [`Builder::async_runtime`] and providing a backing runtime. To start the runtime you
/// must call [`Builder::start`].
pub struct Builder {
    pub(crate) image: Option<(PathBuf, PathBuf)>,
    pub(crate) install_jlrs_core: InstallJlrsCore,
    pub(crate) n_threads: usize,
    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
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
            #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
            n_threadsi: 0,
        }
    }

    #[cfg(feature = "local-rt")]
    #[inline]
    #[deprecated]
    /// initialize Julia on the current thread.
    ///
    /// Deprecated: use [`Builder::start_local`] instead.
    pub unsafe fn start(self) -> JlrsResult<PendingJulia> {
        PendingJulia::init(self)
    }

    #[cfg(feature = "local-rt")]
    #[inline]
    /// initialize Julia on the current thread.
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

    #[inline]
    #[cfg(feature = "multi-rt")]
    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
    pub fn spawn_mt(self) -> JlrsResult<(MtHandle, JoinHandle<()>)> {
        mt_impl::sync_impl::spawn(self)
    }

    #[inline]
    #[cfg(feature = "multi-rt")]
    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
    pub fn start_mt<T: 'static + Send>(
        self,
        func: impl 'static + Send + FnOnce(MtHandle) -> T,
    ) -> JlrsResult<T> {
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

    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
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

        self.image = Some((
            julia_bindir.as_ref().to_path_buf(),
            image_path.as_ref().to_path_buf(),
        ));

        Ok(self)
    }

    /// Enable or disable automatically installing JlrsCore.
    ///
    /// jlrs requires that the JlrsCore package is installed. By default, this package is
    /// installed automatically if it is unavailable
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
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
mod mt_impl {
    use parking_lot::{Condvar, Mutex};
    static INIT_LOCK: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());

    pub(super) mod sync_impl {
        use std::thread::{self, JoinHandle};

        use jl_sys::jl_atexit_hook;

        use super::INIT_LOCK;
        use crate::{
            error::{JlrsError, RuntimeError},
            memory::gc::gc_safe,
            prelude::JlrsResult,
            runtime::{
                builder::{init_runtime, Builder},
                handle::{
                    mt_handle::{wait_loop, MtHandle, EXIT_LOCK},
                    notify, wait,
                },
                state::{can_init, set_exit},
            },
        };

        pub(crate) fn spawn(options: Builder) -> JlrsResult<(MtHandle, JoinHandle<()>)> {
            if !can_init() {
                Err(RuntimeError::AlreadyInitialized)?;
            }

            let handle = thread::spawn(move || {
                unsafe {
                    init_runtime(&options);
                    notify(&INIT_LOCK);
                    wait_loop();

                    // Returned from wait_main, so we're about to exit Julia because all handles have
                    // been dropped. Next we need to wait until we've returned from `notify_main` too.
                    gc_safe(|| wait(&EXIT_LOCK));
                    set_exit();
                    jl_atexit_hook(0);
                }
            });

            wait(&INIT_LOCK);
            let mt_handle = unsafe { MtHandle::new() };
            Ok((mt_handle, handle))
        }

        pub(crate) fn start<T>(
            options: Builder,
            func: impl 'static + Send + FnOnce(MtHandle) -> T,
        ) -> JlrsResult<T>
        where
            T: Send + 'static,
        {
            if !can_init() {
                Err(RuntimeError::AlreadyInitialized)?;
            }

            unsafe {
                init_runtime(&options);
            }

            let handle = thread::spawn(|| unsafe {
                let handle = MtHandle::new();
                func(handle)
            });

            let ret = unsafe {
                wait_loop();

                // Returned from wait_main, so we're about to exit Julia becuase all handles have
                // been dropped. Next we need to wait until we've returned from `notify_main` too.
                gc_safe(|| wait(&EXIT_LOCK));
                set_exit();
                jl_atexit_hook(0);

                match handle.join() {
                    Ok(ret) => ret,
                    Err(e) => Err(JlrsError::exception(format!("{e:?}")))?,
                }
            };

            Ok(ret)
        }
    }
}

unsafe fn init_runtime(options: &Builder) {
    set_n_threads(options);
    init_julia(options);
    init_jlrs(&options.install_jlrs_core);
}

unsafe fn init_julia(options: &Builder) {
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

unsafe fn set_n_threads(options: &Builder) {
    #[cfg(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8",))]
    {
        if options.n_threads == 0 {
            jlrs_set_nthreads(-1);
        } else {
            jlrs_set_nthreads(options.n_threads as _);
        }
    }

    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8",)))]
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
