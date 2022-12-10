//! Build a runtime.
//!
//! Before Julia can be used it must be initialized. The builders provided by this module must be
//! used to initialize Julia and set custom parameters. The [`RuntimeBuilder`] only lets you
//! provide a custom system image, [`AsyncRuntimeBuilder`] provides additional methods to set the
//! number of threads available to Julia among others.

use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

#[cfg(feature = "sync-rt")]
use super::sync_rt::PendingJulia;
#[cfg(any(feature = "sync-rt", feature = "async-rt"))]
use crate::error::JlrsResult;

/// Build a sync runtime.
///
/// With this builder you can set a custom system image by calling [`RuntimeBuilder::image`],
/// the builder can be upgraded to an [`AsyncRuntimeBuilder`] by calling
/// [`RuntimeBuilder::async_runtime`] and providing a backing runtime. To start the runtime you
/// must call [`RuntimeBuilder::start`].
pub struct RuntimeBuilder {
    pub(crate) image: Option<(PathBuf, PathBuf)>,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "async-rt")] {
        use std::{
            marker::PhantomData,
            time::Duration,
        };
        use super::async_rt::{AsyncRuntime, AsyncJulia};

        /// Build the async runtime backed by some runtime `R`.
        pub struct AsyncRuntimeBuilder<R>
        where
            R: AsyncRuntime,
        {
            pub(crate) builder: RuntimeBuilder,
            pub(crate) n_threads: usize,
            pub(crate) channel_capacity: NonZeroUsize,
            pub(crate) recv_timeout: Duration,
            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            pub(crate) n_threadsi: usize,
            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            pub(crate) n_workers: usize,
            _runtime: PhantomData<R>,
        }

        impl<R> AsyncRuntimeBuilder<R>
        where
            R: AsyncRuntime,
        {
            /// Set the number of threads Julia can use.
            ///
            /// If it's set to 0, the default value, the number of threads is the number of CPU
            /// cores.
            ///
            /// NB: When the `nightly` or `beta` feature is enabled, this sets the number of
            /// threads allocated to the `:default` pool.
            pub fn n_threads(mut self, n: usize) -> Self {
                self.n_threads = n;
                self
            }

            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            /// Set the number of threads allocated to the `:interactive` pool.
            ///
            /// If it's set to 0, the default value, no threads are allocated to this pool.
            pub fn n_interactive_threads(mut self, n: usize) -> Self {
                self.n_threadsi = n;
                self
            }


            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            /// Set the number of worker threads jlrs creates in addition to the runtime thread.
            ///
            /// If it's set to 0, the default value, no worker threads are created.
            pub fn n_worker_threads(mut self, n: usize) -> Self {
                self.n_workers = n;
                self
            }

            /// Set the capacity of the channel used to communicate with the async runtime.
            ///
            /// The default value is 16.
            pub fn channel_capacity(mut self, capacity: NonZeroUsize) -> Self {
                self.channel_capacity = capacity;
                self
            }

            /// Set the receive timeout of the channel used to communicate with the async runtime.
            ///
            /// If no message is received before the timeout occurs, the async runtime yields
            /// control to Julia to ensure the scheduler and garbage collector can run, and events
            /// are processed periodically.
            ///
            /// The default value is 1 millisecond.
            pub fn recv_timeout(mut self, timeout: Duration) -> Self {
                self.recv_timeout = timeout;
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
            /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl
            // TODO: Check if these paths exist.
            pub fn image<P, Q>(mut self, julia_bindir: P, image_path: Q) -> Self
            where
                P: AsRef<Path> + Send + 'static,
                Q: AsRef<Path> + Send + 'static,
            {
                self.builder.image = Some((
                    julia_bindir.as_ref().to_path_buf(),
                    image_path.as_ref().to_path_buf(),
                ));
                self
            }

            /// Initialize Julia on another thread.
            ///
            /// You must set the maximum number of concurrent tasks with the `N` const generic.
            pub unsafe fn start<const N: usize>(self) -> JlrsResult<(AsyncJulia<R>, std::thread::JoinHandle<JlrsResult<()>>)> {
                AsyncJulia::init::<N>(self)
            }

            /// Initialize Julia as a blocking task.
            ///
            /// You must set the maximum number of concurrent tasks with the `N` const generic.
            pub unsafe fn start_async<const N: usize>(self) -> JlrsResult<(AsyncJulia<R>, R::RuntimeHandle)> {
                AsyncJulia::init_async::<N>(self)
            }

            pub(crate) fn has_workers(&self) -> bool {
                #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
                {
                    self.n_workers > 0
                }

                #[cfg(not(any(feature = "julia-1-10", feature = "julia-1-9")))]
                {
                    false
                }
            }
        }
    }
}

impl RuntimeBuilder {
    /// Create a new `RuntimeBuilder`.
    pub fn new() -> Self {
        RuntimeBuilder { image: None }
    }

    #[cfg(feature = "sync-rt")]
    /// initialize Julia on the current thread.
    pub unsafe fn start<'context>(self) -> JlrsResult<PendingJulia> {
        PendingJulia::init(self)
    }

    /// Upgrade this builder to an [`AsyncRuntimeBuilder`].
    ///
    /// You must provide a backing runtime `R`, jlrs supports using tokio and async-std as backing
    /// runtimes if the `tokio-rt` and `async-std-rt` features are enabled.
    ///
    /// For example, if you want to use tokio as the backing runtime:
    ///
    /// ```
    /// use jlrs::prelude::*;
    ///
    /// # fn main() {
    /// let (_julia, _thread_handle) = unsafe {
    ///     RuntimeBuilder::new()
    ///         .async_runtime::<Tokio>()
    ///         .start::<1>()
    ///         .expect("Could not start Julia")
    /// };
    /// # }
    /// ```
    ///
    /// Smilarly for async-std:
    ///
    /// ```
    /// use jlrs::prelude::*;
    ///
    /// # fn main() {
    /// let (_julia, _thread_handle) = unsafe {
    ///     RuntimeBuilder::new()
    ///         .async_runtime::<AsyncStd>()
    ///         .start::<1>()
    ///         .expect("Could not start Julia")
    /// };
    /// # }
    /// ```
    #[cfg(feature = "async-rt")]
    pub fn async_runtime<R>(self) -> AsyncRuntimeBuilder<R>
    where
        R: AsyncRuntime,
    {
        AsyncRuntimeBuilder {
            builder: self,
            n_threads: 0,
            channel_capacity: unsafe { NonZeroUsize::new_unchecked(16) },
            recv_timeout: Duration::from_millis(1),
            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            n_threadsi: 0,
            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            n_workers: 0,
            _runtime: PhantomData,
        }
    }

    /// Use a custom system image.
    ///
    /// You must provide two arguments to use a custom system image, `julia_bindir` and
    /// `image_path`. The first is the absolute path to a directory that contains a compatible
    /// Julia binary (eg `${JULIA_DIR}/bin`), the second is the path to a system image.
    ///
    /// A custom system image can be created with [`PackageCompiler`].
    ///
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl
    // TODO: Check if these paths exist.
    pub fn image<P, Q>(mut self, julia_bindir: P, image_path: Q) -> Self
    where
        P: AsRef<Path> + Send + 'static,
        Q: AsRef<Path> + Send + 'static,
    {
        self.image = Some((
            julia_bindir.as_ref().to_path_buf(),
            image_path.as_ref().to_path_buf(),
        ));
        self
    }
}
