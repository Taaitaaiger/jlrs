//! Async tasks and channels that can be used with an async runtime.

pub mod channel;
#[cfg(feature = "async-rt")]
pub(crate) mod internal;
pub(crate) mod future;
pub mod task;
