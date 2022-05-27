//! A reusable slot in a frame.

use super::frame::Frame;
use crate::{
    private::Private,
    wrappers::ptr::{private::WrapperPriv, value::Value, ValueRef},
};
use std::{cell::Cell, ffi::c_void, marker::PhantomData};

/// A reusable slot in a frame. Unlike an `Output`, a `ReusableSlot` can be used multiple times.
/// It's your responsibility to ensure a value that is rooted using a `ReusableSlot` is never
/// used after the slot has been reset.
#[derive(Clone, Copy)]
pub struct ReusableSlot<'target> {
    slot: *const Cell<*mut c_void>,
    _marker: PhantomData<fn(&'target mut ())>,
}

impl<'target> ReusableSlot<'target> {
    pub(crate) fn new<F: Frame<'target>>(_frame: &F, slot: *const Cell<*mut c_void>) -> Self {
        ReusableSlot {
            slot,
            _marker: PhantomData,
        }
    }

    /// Root the given value in this slot, any data currently rooted in this slot is potentially
    /// unreachable after calling this method.
    pub fn reset<'data>(self, new: Value<'_, 'data>) -> ValueRef<'target, 'data> {
        unsafe {
            let ptr = new.unwrap_non_null(Private).as_ptr();
            (&*self.slot).set(ptr.cast());
            ValueRef::wrap(ptr)
        }
    }
}
