//! Build a Julia runtime.
//!
//! Before Julia can be used it must be initialized. The builders provided by this module must be
//! used to initialize Julia and set custom parameters. The [`RuntimeBuilder`] only lets you
//! provide a custom system image, [`AsyncRuntimeBuilder`] provides additional methods to set the
//! number of threads available to Julia among others.

#[cfg(any(feature = "sync-rt", feature = "async-rt"))]
use crate::error::JlrsResult;
use std::path::{Path, PathBuf};

#[cfg(feature = "sync-rt")]
use super::sync_rt::Julia;

/// Build a sync runtime.
///
/// With this builder you can set a custom system image by calling [`RuntimeBuilder::image`],
/// the builder can be upgraded to an [`AsyncRuntimeBuilder`] by calling
/// [`RuntimeBuilder::async_runtime`] and providing a backing runtime and channel type. To start
/// the runtime you must call [`RuntimeBuilder::start`]
pub struct RuntimeBuilder {
    pub(crate) image: Option<(PathBuf, PathBuf)>,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "async-rt")] {
        use std::{
            marker::PhantomData,
            time::Duration,
        };
        use crate::async_util::channel::Channel;
        use super::async_rt::{AsyncRuntime, Message, AsyncJulia};

        pub struct AsyncRuntimeBuilder<R, C>
        where
            R: AsyncRuntime,
            C: Channel<Message>,
        {
            pub(crate) builder: RuntimeBuilder,
            #[cfg(not(feature = "lts"))]
            pub(crate) n_threads: usize,
            pub(crate) n_tasks: usize,
            pub(crate) channel_capacity: usize,
            pub(crate) recv_timeout: Duration,
            _runtime: PhantomData<R>,
            _channel: PhantomData<C>,
        }

        impl<R, C> AsyncRuntimeBuilder<R, C>
        where
            R: AsyncRuntime,
            C: Channel<Message>,
        {
            /// Set the number of threads Julia can use.
            ///
            /// If it's set to 0, the default value, the number of threads is the number of CPU cores.
            ///
            /// This method is not available for the LTS version, instead you must set the number
            /// of threads using the `JULIA_NUM_THREADS` environment variable. At least three
            /// threads are required.
            #[cfg(not(feature = "lts"))]
            pub fn n_threads(mut self, n: usize) -> Self {
                self.n_threads = n;
                self
            }

            /// Set the maximum number of concurrently running tasks.
            ///
            /// If it's set to 0, the default value, the number is equal to the number of threads.
            pub fn n_tasks(mut self, n: usize) -> Self {
                self.n_tasks = n;
                self
            }

            /// Set the capacity of the channel used to communicate with the async runtime.
            ///
            /// If it's set to 0, the channel is created by calling `C::channel(None)`, otherwise
            /// `C::channel(Some(capacity))` is called.
            pub fn channel_capacity(mut self, capacity: usize) -> Self {
                self.channel_capacity = capacity;
                self
            }

            /// Set the receive timeout of the channel used to communicate with the async runtime.
            ///
            /// If no message is received before the timeout occurs, the async runtime yields
            /// control to Julia to ensure the scheduler can run and events are processed
            /// periodically. By default it's 1 millisecond.
            pub fn recv_timeout(mut self, timeout: Duration) -> Self {
                self.recv_timeout = timeout;
                self
            }

            /// Set a custom system image.
            ///
            /// You provide two arguments to use a custom system image, `julia_bindir` and
            /// `image_path`. The first is the absolute path to a directory that contains a
            /// compatible Julia binary (eg `${JULIA_DIR}/bin`), the second must be either an
            /// absolute or a relative path to a system image.
            ///
            /// A custom system image can be created with [`PackageCompiler`].
            ///
            /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl
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
            pub unsafe fn start(self) -> JlrsResult<(AsyncJulia<R>, std::thread::JoinHandle<JlrsResult<()>>)> {
                AsyncJulia::init(self)
            }

            /// Initialize Julia as a blocking task.
            pub async unsafe fn start_async(self) -> JlrsResult<(AsyncJulia<R>, R::RuntimeHandle)> {
                AsyncJulia::init_async(self).await
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
    pub unsafe fn start(self) -> JlrsResult<Julia> {
        Julia::init(self)
    }

    /// Upgrade this builder to an [`AsyncRuntimeBuilder`].
    ///
    /// In order to upgrade this builder to an `AsyncRuntimeBuilder`, you must provide a backing
    /// runtime `R` and a backing channel `C`. By default, jlrs supports using tokio and async-std
    /// as backing runtimes if the `tokio-rt` and `async-std-rt` features are enabled.
    ///
    /// For example, if you want to use tokio as the backing runtime and use an unbounded channel:
    ///
    /// ```
    /// use jlrs::prelude::*;
    ///
    /// # fn main() {
    ///     let (_julia, _thread_handle) = unsafe { RuntimeBuilder::new()
    ///         .async_runtime::<Tokio, UnboundedChannel<_>>()
    ///         .start()
    ///         .expect("Could not start Julia") };
    /// # }
    /// ```
    ///
    /// Smilarly for async-std:
    ///
    /// ```
    /// use jlrs::prelude::{AsyncStd, AsyncStdChannel, RuntimeBuilder};
    ///
    /// # fn main() {
    ///     let (_julia, _thread_handle) = unsafe { RuntimeBuilder::new()
    ///         .async_runtime::<AsyncStd, AsyncStdChannel<_>>()
    ///         .start()
    ///         .expect("Could not start Julia") };
    /// # }
    /// ```
    #[cfg(feature = "async-rt")]
    pub fn async_runtime<R, C>(self) -> AsyncRuntimeBuilder<R, C>
    where
        R: AsyncRuntime,
        C: Channel<Message>,
    {
        AsyncRuntimeBuilder {
            builder: self,
            #[cfg(not(feature = "lts"))]
            n_threads: 0,
            n_tasks: 0,
            channel_capacity: 0,
            recv_timeout: Duration::from_millis(1),
            _runtime: PhantomData,
            _channel: PhantomData,
        }
    }

    /// Use a custom system image.
    ///
    /// You provide two arguments to use a custom system image, `julia_bindir` and `image_path`.
    /// The first is the absolute path to a directory that contains a compatible Julia binary (eg
    /// `${JULIA_DIR}/bin`), the second must be either an absolute or a relative path to a system
    /// image.
    ///
    /// A custom system image can be created with [`PackageCompiler`].
    ///
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl
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
