//! Executor used by background threads for cooperative multitasking.

use std::{future::Future, time::Duration};

use async_trait::async_trait;

use super::handle::async_handle::{channel::RecvError, message::Message};

/// Indicates that a task has finished running.
pub trait IsFinished {
    /// Returns `true` if a task has finished running.
    fn is_finished(&self) -> bool;
}

/// Functionality that is necessary to use jlrs asynchronously.
///
/// Note that executors can be blocked for extended periods of time, so this trait should only be
/// implemented for async runtimes that let you create independent local runtimes.
///
/// The generic `N` is the maximum number of concurrent tasks can be handled by the executor.
///
/// An implementation that uses tokio is avaiable: [`Tokio`].
///
/// [`Tokio`]: crate::runtime::executor::tokio_exec::Tokio
#[async_trait(?Send)]
pub trait Executor<const N: usize>: Send + Sync + 'static {
    /// Error that is returned when a task can't be joined because it has panicked.
    ///
    /// If the runtime doesn't catch panics, use `()`.
    type JoinError;

    /// The handle type of a task spawned by `Executor::spawn_local`.
    type JoinHandle: Future<Output = Result<(), Self::JoinError>> + IsFinished;

    /// An executor that can't handle async tasks won't function correctly.
    ///
    /// Do not override the default implementation.
    const VALID: () = assert!(N > 0, "executor must support at leat 1 task");

    /// Run `loop_fn` to completion.
    ///
    /// Implementations of this method should start a new local runtime. `loop_fn` may block for
    /// extended periods of time.
    fn block_on<T, F>(&self, loop_fn: F) -> T
    where
        F: Future<Output = T>;

    /// Spawn `future` as a task on the current executor.
    fn spawn_local<F>(future: F) -> Self::JoinHandle
    where
        F: Future<Output = ()> + 'static;

    /// Yield the current task.
    fn yield_now() -> impl Future<Output = ()>;

    /// Wait on `future` until it resolves or `duration` has elapsed. If the future times out it
    /// must return `None`.
    async fn timeout<F>(duration: Duration, future: F) -> Option<Result<Message, RecvError>>
    where
        F: Future<Output = Result<Message, RecvError>>;
}

#[cfg(feature = "tokio-rt")]
pub mod tokio_exec {
    use std::sync::Arc;

    use tokio::{
        runtime::Builder,
        task::{JoinError, JoinHandle, LocalSet},
        time::timeout,
    };

    use super::*;

    pub type TokioCallback = Arc<dyn Fn() + Send + Sync>;

    /// Executor that uses tokio.
    #[derive(Clone)]
    pub struct Tokio<const N: usize> {
        #[allow(dead_code)]
        enable_io: bool,
        on_thread_park: Option<TokioCallback>,
        on_thread_unpark: Option<TokioCallback>,
    }

    impl<const N: usize> Tokio<N> {
        /// The tokio executor can optionally enable the IO driver.
        ///
        /// Enabling IO does nothing if the `tokio-net` feature has not been enabled.
        pub const fn new(enable_io: bool) -> Self {
            Tokio {
                enable_io,
                on_thread_park: None,
                on_thread_unpark: None,
            }
        }

        /// See [`tokio::runtime::Builder::on_thread_park`]
        pub fn on_thread_park<F: Fn() + Send + Sync + 'static>(
            &mut self,
            on_thread_park: F,
        ) -> &mut Self {
            self.on_thread_park = Some(Arc::new(on_thread_park));
            self
        }

        /// See [`tokio::runtime::Builder::on_thread_unpark`]
        pub fn on_thread_unpark<F: Fn() + Send + Sync + 'static>(
            &mut self,
            on_thread_unpark: F,
        ) -> &mut Self {
            self.on_thread_unpark = Some(Arc::new(on_thread_unpark));
            self
        }
    }

    impl IsFinished for JoinHandle<()> {
        fn is_finished(&self) -> bool {
            self.is_finished()
        }
    }

    #[async_trait(?Send)]
    impl<const N: usize> Executor<N> for Tokio<N> {
        type JoinError = JoinError;
        type JoinHandle = JoinHandle<()>;

        #[inline]
        fn block_on<T, F>(&self, loop_fn: F) -> T
        where
            F: Future<Output = T>,
        {
            let mut builder = Builder::new_current_thread();
            builder.enable_time();

            if let Some(ref on_thread_park) = self.on_thread_park {
                let on_thread_park = on_thread_park.clone();
                builder.on_thread_park(move || on_thread_park.as_ref()());
            }

            if let Some(ref on_thread_unpark) = self.on_thread_unpark {
                let on_thread_unpark = on_thread_unpark.clone();
                builder.on_thread_unpark(move || on_thread_unpark.as_ref()());
            }

            #[cfg(feature = "tokio-net")]
            if self.enable_io {
                builder.enable_io();
            }

            let runtime = builder.build().expect("unable to build tokio runtime");

            let local_set = LocalSet::new();
            local_set.block_on(&runtime, loop_fn)
        }

        #[inline]
        fn spawn_local<F>(future: F) -> Self::JoinHandle
        where
            F: Future<Output = ()> + 'static,
        {
            tokio::task::spawn_local(future)
        }

        #[inline]
        fn yield_now() -> impl Future<Output = ()> {
            tokio::task::yield_now()
        }

        async fn timeout<F>(duration: Duration, future: F) -> Option<Result<Message, RecvError>>
        where
            F: Future<Output = Result<Message, RecvError>>,
        {
            timeout(duration, future).await.ok()
        }
    }
}
