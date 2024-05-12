//! Dispatch a task to the async runtime.

use std::fmt;

use async_channel::{SendError, Sender, TrySendError};

use super::channel::OneshotReceiver;
use crate::{
    error::{JlrsError, RuntimeError},
    prelude::JlrsResult,
};

/// Dispatch a task to the async runtime.
pub struct Dispatch<'a, M, T> {
    msg: M,
    sender: &'a Sender<M>,
    receiver: OneshotReceiver<T>,
}

impl<'a, M, T> Dispatch<'a, M, T> {
    #[inline]
    pub(crate) const fn new(msg: M, sender: &'a Sender<M>, receiver: OneshotReceiver<T>) -> Self {
        Dispatch {
            msg,
            sender,
            receiver,
        }
    }

    /// Dispatch the task.
    pub async fn dispatch(self) -> JlrsResult<OneshotReceiver<T>> {
        match self.sender.send(self.msg).await {
            Ok(_) => Ok(self.receiver),
            Err(SendError(_)) => Err(RuntimeError::ChannelClosed)?,
        }
    }

    /// Try to dispatch the task.
    ///
    /// If the channel is full, the dispatcher is returned to allow retrying.
    pub fn try_dispatch(self) -> Result<OneshotReceiver<T>, JlrsResult<Self>> {
        match self.sender.try_send(self.msg) {
            Ok(_) => Ok(self.receiver),
            Err(TrySendError::Closed(_)) => Err(Err(Box::new(JlrsError::RuntimeError(
                RuntimeError::ChannelClosed,
            )))),
            Err(TrySendError::Full(msg)) => Err(Ok(Dispatch {
                msg,
                sender: self.sender,
                receiver: self.receiver,
            })),
        }
    }
}

impl<'a, M, T> fmt::Debug for Dispatch<'a, M, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dispatch").finish()
    }
}
