//! Traits used to implement tasks for the async runtime.

use super::async_frame::AsyncGcFrame;
use crate::error::JlrsResult;
use crate::memory::global::Global;
use async_std::channel::Sender as AsyncStdSender;
use async_trait::async_trait;
use crossbeam_channel::Sender as CrossbeamSender;

/// The `AsyncTask` trait is used to create tasks that the async runtime can execute.
/// Implementations of this trait take the place of the closures used with the sync runtime.
#[async_trait(?Send)]
pub trait AsyncTask: Send + Sync + 'static {
    /// The type of the result of this task. Must be the same across all implementations.
    type T: 'static + Send + Sync;

    /// The type of the sender that is used to send the result of this task back to the original
    /// caller. Must be the same across all implementations.
    type R: ReturnChannel<T = Self::T>;

    /// The entrypoint of a task. You can use the `Global` and `AsyncGcFrame` to call arbitrary
    /// functions from Julia. Additionally, [`CallAsync::call_async`] can be used to call a function
    /// on another thread and allow other tasks to progress while awaiting the result.
    /// Implementations that don't use [`CallAsync::call_async`] will block the runtime during
    /// execution.
    ///
    /// [`CallAsync::call_async`]: crate::extensions::multitask::call_async::CallAsync
    async fn run<'base>(
        &mut self,
        global: Global<'base>,
        frame: &mut AsyncGcFrame<'base>,
    ) -> JlrsResult<Self::T>;

    /// The return channel for this task, or `None` if the result doesn't need to be returned.
    /// Returns `None` by default.
    fn return_channel(&self) -> Option<&Self::R> {
        None
    }
}

/// The `ReturnChannel` trait is implemented by types that can send a result back to a caller. It
/// is implemented for both `async_std::sync::Sender` and `crossbeam_channel::Sender`.
#[async_trait]
pub trait ReturnChannel: 'static {
    type T: Send + Sync + 'static;

    /// Send the result.
    async fn send(&self, response: JlrsResult<Self::T>);
}

#[async_trait]
impl<T: Send + Sync + 'static> ReturnChannel for AsyncStdSender<JlrsResult<T>> {
    type T = T;
    async fn send(&self, response: JlrsResult<Self::T>) {
        self.send(response).await.ok();
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> ReturnChannel for CrossbeamSender<JlrsResult<T>> {
    type T = T;
    async fn send(&self, response: JlrsResult<Self::T>) {
        self.send(response).ok();
    }
}
