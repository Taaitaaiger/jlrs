//! Traits used to implement tasks for the async runtime.

use crate::error::JlrsResult;
use crate::frame::AsyncFrame;
use crate::global::Global;
use async_std::sync::Sender as AsyncStdSender;
use async_trait::async_trait;
use crossbeam_channel::Sender as CrossbeamSender;

/// The `JuliaTask` trait is used to create tasks that the async runtime can execute.
/// Implementations of this trait take the place of the closures used with the sync runtime.
#[async_trait(?Send)]
pub trait JuliaTask: Send + Sync + 'static {
    /// The type of the result of this task. Must be the same across all implementations.
    type T: 'static + Send + Sync;

    /// The type of the sender that is used to send the result of this task back to the original
    /// caller. Must be the same across all implementations.
    type R: ReturnChannel<T = Self::T>;

    /// The entrypoint of a task. You can use the `Global` and `AsyncFrame` to call arbitrary
    /// functions from Julia. Additionally, [`Value::call_async`] can be used to call a function
    /// on another thread and allow other tasks to progress while awaiting the result.
    /// Implementations that don't use [`Value::call_async`] will block the runtime during
    /// execution.
    ///
    /// [`Value::call_async`]: ../../value/struct.Value.html#method.call_async
    async fn run<'base>(
        &mut self,
        global: Global<'base>,
        frame: &mut AsyncFrame<'base>,
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
        self.send(response).await
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> ReturnChannel for CrossbeamSender<JlrsResult<T>> {
    type T = T;
    async fn send(&self, response: JlrsResult<Self::T>) {
        self.send(response).ok();
    }
}
