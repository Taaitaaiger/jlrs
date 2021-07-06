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
    unsafe fn push_frame(&self, raw_frame: &mut [*mut c_void], capacity: usize, _: Private) {
        raw_frame[0] = (capacity << 1) as _;
        raw_frame[1] = self.0.get();

        for i in 0..capacity {
            raw_frame[2 + i] = null_mut();
        }

        self.0.set(raw_frame[..].as_mut_ptr().cast());
    }

    unsafe fn pop_frame(&self, raw_frame: &mut [*mut c_void], _: Private) {
        self.0.set(raw_frame[1]);
    }
}
