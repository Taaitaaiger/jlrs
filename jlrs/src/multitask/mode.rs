//! Custom mode for the async runtime.

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::null_mut;

use crate::memory::mode::{private::Mode as ModePriv, Mode};
use crate::private::Private;

/// Mode used by the async runtime.
#[derive(Clone, Copy)]
pub struct Async<'a>(pub(crate) &'a Cell<*mut c_void>);

impl<'a> Mode for Async<'a> {}

impl<'a> ModePriv for Async<'a> {
    unsafe fn push_frame(&self, raw_frame: &mut [Cell<*mut c_void>], _: Private) {
        raw_frame[0].set(null_mut());
        raw_frame[1].set(self.0.get());
        self.0.set(raw_frame.as_mut_ptr().cast());
    }

    unsafe fn pop_frame(&self, raw_frame: &mut [Cell<*mut c_void>], _: Private) {
        self.0.set(raw_frame[1].get());
    }
}
