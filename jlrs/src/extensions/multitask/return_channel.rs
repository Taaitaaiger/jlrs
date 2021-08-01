//! A trait implemented by senders that can be used with the async runtime.

use crate::error::JlrsResult;
use async_std::channel::Sender as AsyncStdSender;
use async_trait::async_trait;
use crossbeam_channel::Sender as CrossbeamSender;

/// The `ReturnChannel` trait can be implemented by the sending half of a channel. It's
/// implemented for both `async_std::channel::Sender` and `crossbeam_channel::Sender`. It's also
/// implemented for `()`, in this case nothing is sent back.
#[async_trait]
pub trait ReturnChannel: 'static + Send + Sync + Sized {
    type T: 'static + Send + Sync;

    /// Send the result.
    async fn send(&self, response: JlrsResult<Self::T>);
}
#[async_trait]
impl<T: 'static + Send + Sync> ReturnChannel for AsyncStdSender<JlrsResult<T>> {
    type T = T;
    async fn send(&self, response: JlrsResult<Self::T>) {
        self.send(response).await.ok();
    }
}

#[async_trait]
impl<T: 'static + Send + Sync> ReturnChannel for CrossbeamSender<JlrsResult<T>> {
    type T = T;

    async fn send(&self, response: JlrsResult<Self::T>) {
        self.send(response).ok();
    }
}

#[async_trait]
impl ReturnChannel for () {
    type T = ();
    async fn send(&self, _: JlrsResult<()>) {}
}
