//! Async tasks and channels that can be used with an async runtime.
//!
//! In this module you'll find several traits that can be used to implement async tasks and async
//! channels that can be used wth an async runtime. This module is available if the `async`
//! feature is enabled, it doesn't require using an async runtime.

pub mod channel;
#[cfg(feature = "async-rt")]
pub(crate) mod internal;
pub(crate) mod julia_future;
pub mod task;
