//! A trait implemented by senders that can be used with the async runtime.

use crate::error::JlrsResult;
use async_std::channel::Sender as AsyncStdSender;
use async_trait::async_trait;
use crossbeam_channel::Sender as CrossbeamSender;

/// The `ReturnChannel` trait can be implemented by the sending half of a channel. It's
/// implemented for both `async_std::channel::Sender` and `crossbeam_channel::Sender`. It's also
/// implemented for `()`, in this case nothing is sent back.
///
/// Note that if `ReturnChannel::send` fails, it's never retried. This is not an issue for the
/// provided implementations, which only fail if the channel has been disconnected and no result
/// can ever be successfully sent back to the receiving end.
#[async_trait(?Send)]
pub trait ReturnChannel: 'static + Send + Sync + Sized {
    /// The type of data that is sent if the result was computed successfully.
    type Ok: 'static + Send;

    /// Send the result.
    async fn send(&self, response: JlrsResult<Self::Ok>);
}

#[async_trait(?Send)]
impl<T: 'static + Send> ReturnChannel for AsyncStdSender<JlrsResult<T>> {
    type Ok = T;
    async fn send(&self, response: JlrsResult<Self::Ok>) {
        self.send(response).await.ok();
    }
}

#[async_trait(?Send)]
impl<T: 'static + Send> ReturnChannel for CrossbeamSender<JlrsResult<T>> {
    type Ok = T;

    async fn send(&self, response: JlrsResult<Self::Ok>) {
        self.send(response).ok();
    }
}

#[async_trait(?Send)]
impl ReturnChannel for () {
    type Ok = ();
    async fn send(&self, _: JlrsResult<()>) {}
}
