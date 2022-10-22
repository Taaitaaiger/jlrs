//! Channel traits used to communicate with the async runtime.
//!
//! In order to communicate with the async runtime you must use channels that implement the traits
//! defined in this module. Async runtimes and persistent tasks need a backing channel whose
//! sending half implements [`ChannelSender`], whose receiving half implements
//! [`ChannelReceiver`], and the pair must implement [`Channel`]. Async tasks, blocking tasks and
//! calling persistent tasks need an implementation of [`OneshotSender`] to send their result.
//!
//! Several implementations of this trait are provided by jlrs if the `async-std-rt` or `tokio-rt`
//! feature is enabled.

use crate::error::JlrsResult;
use async_trait::async_trait;
use std::{fmt, num::NonZeroUsize};

/// A sending error that indicates the channel is closed.
pub struct SendError<T>(pub T);

impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "channel closed")
    }
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "channel closed")
    }
}

impl<T> std::error::Error for SendError<T> {}

/// A sending error that indicates the channel is closed or full.
pub enum TrySendError<T> {
    Full(T),
    Closed(T),
}

impl<T> fmt::Debug for TrySendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed(_) => write!(fmt, "channel closed"),
            Self::Full(_) => write!(fmt, "channel full"),
        }
    }
}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed(_) => write!(fmt, "channel closed"),
            Self::Full(_) => write!(fmt, "channel full"),
        }
    }
}

impl<T> std::error::Error for TrySendError<T> {}

/// An async channel.
///
/// This channel is used to communicate with an async runtime or persistent task.
pub trait Channel<M: Send + Sync + 'static>: 'static + Send + Sync {
    type Sender: ChannelSender<M>;
    type Receiver: ChannelReceiver<M>;

    /// Create a new channel.
    ///
    /// This method is used to create the backing channel for a task or runtime.
    fn channel(capacity: Option<NonZeroUsize>) -> (Self::Sender, Self::Receiver);
}

/// The sending half of an async channel.
///
/// The handle to a persistent task or async runtime contains a sender that implements this trait.
#[async_trait]
pub trait ChannelSender<M: Send + Sync + 'static>: 'static + Send + Sync {
    /// Send a message to the receiving half.
    ///
    /// This method must wait until the message can be sent. If the channel is closed it must
    /// return a `SendError`.
    async fn send(&self, msg: M) -> Result<(), SendError<M>>;

    /// Send a message to the receiving half.
    ///
    /// This method must return `TrySendError::Full` immediately if the channel is full. If the
    /// channel is closed it must `TrySendError::Closed`.
    fn try_send(&self, msg: M) -> Result<(), TrySendError<M>>;
}

/// The receiving half of an async channel.
///
/// An async runtime and a persistent task use a receiver that implements this trait to wait for
/// new messages.
#[async_trait]
pub trait ChannelReceiver<M: Send + Sync + 'static>: 'static + Send + Sync {
    /// Receive a new message.
    ///
    /// This method is called by an async runtime and persistent tasks to receive new commands.
    /// It must be truly async, i.e. wait until a message is available without blocking the
    /// thread it's called from.
    async fn recv(&mut self) -> JlrsResult<M>;
}

/// The sending half of a channel that sends back a result.
///
/// Every time you send a new async or blocking task to the runtime, or call a persistent task,
/// you must provide the sending half of a channel that implements this trait. When the task is
/// done the result is sent to the receiving half.
pub trait OneshotSender<M: Send + 'static>: 'static + Send + Sync {
    fn send(self, msg: M);
}

#[async_trait::async_trait]
impl<M: Send + 'static> OneshotSender<M> for crossbeam_channel::Sender<M> {
    fn send(self, msg: M) {
        (&self).send(msg).ok();
    }
}
