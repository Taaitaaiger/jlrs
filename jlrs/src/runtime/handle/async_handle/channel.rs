//! Re-export tokio oneshot channel sender and receiver types, and async-channel's `RecvError`.

use async_channel::{bounded, unbounded, Receiver, Sender};

/// Reexport of the sending half of a tokio oneshot channel
pub type OneshotSender<T> = tokio::sync::oneshot::Sender<T>;
/// Reexport of the receiving half of a tokio oneshot channel
pub type OneshotReceiver<T> = tokio::sync::oneshot::Receiver<T>;

#[inline]
pub(crate) fn channel<T>(channel_capacity: usize) -> (Sender<T>, Receiver<T>) {
    if channel_capacity == 0 {
        unbounded()
    } else {
        bounded(channel_capacity)
    }
}

///async-channel's `RecvError`
pub type RecvError = async_channel::RecvError;
