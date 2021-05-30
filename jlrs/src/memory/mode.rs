//! The runtime modes
//!
//! The structs you find in this module implement the `Mode` trait which is responsible for
//! handling the differences between pushing and popping GC frames from the stack for the
//! different modes.

#[cfg(feature = "async")]
use std::cell::Cell;
#[cfg(feature = "async")]
use std::ffi::c_void;

/// Mode used by the synchronous runtime.
#[derive(Clone, Copy)]
pub struct Sync;

/// Mode used by the asynchronous runtime.
#[derive(Clone, Copy)]
#[cfg(feature = "async")]
pub struct Async<'a>(pub(crate) &'a Cell<*mut c_void>);
