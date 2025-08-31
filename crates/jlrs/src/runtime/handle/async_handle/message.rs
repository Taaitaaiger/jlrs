use std::fmt;

use super::envelope::{
    BlockingTaskEnvelope, IncludeTaskEnvelope, PendingTaskEnvelope, SetErrorColorTaskEnvelope,
};

/// The message type used by the async runtime for communication.
pub struct Message {
    pub(super) inner: MessageInner,
}

pub(super) enum MessageInner {
    Task(Box<dyn PendingTaskEnvelope>),
    BlockingTask(Box<dyn BlockingTaskEnvelope>),
    Include(Box<dyn IncludeTaskEnvelope>),
    ErrorColor(Box<dyn SetErrorColorTaskEnvelope>),
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Message")
    }
}

unsafe impl Sync for Message {}

impl MessageInner {
    #[inline]
    pub(super) fn wrap(self) -> Message {
        Message { inner: self }
    }
}
