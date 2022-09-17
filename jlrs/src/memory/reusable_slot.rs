//! A reusable slot in a frame.

use crate::{
    private::Private,
    wrappers::ptr::{
        private::WrapperPriv,
        value::{Value, ValueRef},
    },
};

use super::context::Stack;

/*
/// A reusable slot in a frame. Unlike an `Output`, a `ReusableSlot` can be used multiple times.
/// It's your responsibility to ensure a value that is rooted using a `ReusableSlot` is never
/// used after the slot has been reset.
#[derive(Clone, Copy)]
pub struct ReusableSlot<'target> {
    slot: &'target Slot,
}

impl<'target> ReusableSlot<'target> {
    // Safety: slot must have been reserved in _frame
    pub(crate) unsafe fn new(slot: &'target Slot) -> Self {
        ReusableSlot { slot }
    }

    /// Root the given value in this slot, any data currently rooted in this slot is potentially
    /// unreachable after calling this method.
    pub fn reset<'data>(self, new: Value<'_, 'data>) -> ValueRef<'target, 'data> {
        let ptr = new.unwrap(Private);

        // the slot is valid as long as the ReusableSlot is.
        unsafe {
            (&*self.slot).set(ptr.cast());
            ValueRef::wrap(ptr)
        }
    }
}*/

/// A reusable slot in a frame. Unlike an `Output`, a `ReusableSlot` can be used multiple times.
/// It's your responsibility to ensure a value that is rooted using a `ReusableSlot` is never
/// used after the slot has been reset.
#[derive(Clone, Copy)]
pub struct ReusableSlot<'target> {
    context: &'target Stack,
    offset: usize,
}

impl<'target> ReusableSlot<'target> {
    // Safety: slot must have been reserved in _frame
    pub(crate) unsafe fn new(context: &'target Stack, offset: usize) -> Self {
        ReusableSlot { context, offset }
    }

    /// Root the given value in this slot, any data currently rooted in this slot is potentially
    /// unreachable after calling this method.
    pub fn reset<'data>(self, new: Value<'_, 'data>) -> ValueRef<'target, 'data> {
        let ptr = new.unwrap_non_null(Private);

        // the slot is valid as long as the ReusableSlot is.
        unsafe {
            self.context.set_root(self.offset, ptr);
            ValueRef::wrap(ptr.as_ptr())
        }
    }
}
