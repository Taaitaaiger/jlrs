//! A reusable slot in a frame.

use crate::{
    memory::stack_page::Slot,
    private::Private,
    wrappers::ptr::{private::WrapperPriv, value::Value, ValueRef},
};
use std::marker::PhantomData;

/// A reusable slot in a frame. Unlike an `Output`, a `ReusableSlot` can be used multiple times.
/// It's your responsibility to ensure a value that is rooted using a `ReusableSlot` is never
/// used after the slot has been reset.
#[derive(Clone, Copy)]
pub struct ReusableSlot<'target> {
    slot: *const Slot,
    _marker: PhantomData<fn(&'target ())>,
}

impl<'target> ReusableSlot<'target> {
    // Safety: slot must have been reserved in _frame
    pub(crate) unsafe fn new(slot: &'target Slot) -> Self {
        ReusableSlot {
            slot,
            _marker: PhantomData,
        }
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
}
