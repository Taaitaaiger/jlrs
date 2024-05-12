use std::fmt;

use async_channel::Sender;
use tokio::sync::oneshot::channel as oneshot_channel;

use super::{
    dispatch::Dispatch,
    envelope::{CallPersistentTask, InnerPersistentMessage},
};
use crate::async_util::task::PersistentTask;

/// The message type used by persistent handles for communication with persistent tasks.
pub struct PersistentMessage<P>
where
    P: PersistentTask,
{
    pub(super) msg: InnerPersistentMessage<P>,
}

unsafe impl<P> Sync for PersistentMessage<P> where P: PersistentTask {}

impl<P> fmt::Debug for PersistentMessage<P>
where
    P: PersistentTask,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PersistentMessage")
    }
}

/// A handle to a [`PersistentTask`].
///
/// This handle can be used to call the task and shared across threads. The `PersistentTask` is
/// dropped when its final handle has been dropped and all remaining pending calls have completed.
#[derive(Clone)]
pub struct PersistentHandle<P>
where
    P: PersistentTask,
{
    sender: Sender<PersistentMessage<P>>,
}

impl<P> PersistentHandle<P>
where
    P: PersistentTask,
{
    pub(crate) fn new(sender: Sender<PersistentMessage<P>>) -> Self {
        PersistentHandle { sender }
    }

    /// Prepare to call the persistent task with the provided input.
    pub fn call(&self, input: P::Input) -> Dispatch<PersistentMessage<P>, P::Output> {
        let (sender, receiver) = oneshot_channel();
        let msg = PersistentMessage {
            msg: Box::new(CallPersistentTask {
                input: Some(input),
                sender,
            }),
        };

        Dispatch::new(msg, &self.sender, receiver)
    }
}
