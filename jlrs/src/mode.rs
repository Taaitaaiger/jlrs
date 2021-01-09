//! The runtime modes
//!
//! The structs you find in this module implement the `Mode` trait which is responsible for
//! handling the differences between pushing and popping frames from the GC stack for the
//! different modes.

#[cfg(all(feature = "async", target_os = "linux"))]
use std::cell::Cell;
#[cfg(all(feature = "async", target_os = "linux"))]
use std::ffi::c_void;

/// Mode used by the synchronous runtime.
#[derive(Clone)]
pub struct Sync;

/// Mode used by the asynchronous runtime.
#[derive(Clone)]
#[cfg(all(feature = "async", target_os = "linux"))]
pub struct Async<'a>(pub(crate) &'a Cell<*mut c_void>);
