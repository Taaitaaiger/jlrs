//! Embed Julia in a Rust application.
//!
//! There are two ways Julia can be embedded in a Rust application using jlrs, either as a sync or
//! async runtime. The sync runtime initializes Julia on the current thread and you can interact
//! with it directly, while an async runtime is run on one or more separate threads and uses a
//! task-based system. More information is available in the [`sync_rt`] and [`async_rt`] modules
//! respectively.
//!
//! To create a runtime, you must use a [`RuntimeBuilder`]. See the [`builder`] module for more
//! information.
//!
//! [`RuntimeBuilder`]: crate::runtime::builder::RuntimeBuilder

use std::sync::atomic::AtomicBool;

#[cfg(feature = "async-rt")]
pub mod async_rt;
pub mod builder;
#[cfg(feature = "sync-rt")]
pub mod sync_rt;

pub(crate) static INIT: AtomicBool = AtomicBool::new(false);
