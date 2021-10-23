//! A reusable slot in a GC frame.
use super::frame::Frame;
use crate::{
    error::JlrsResult,
    private::Private,
    wrappers::ptr::{private::Wrapper, value::Value, ValueRef},
};
use std::{ffi::c_void, marker::PhantomData};

/// A reusable slot in a GC frame. Unlike an `Output`, a `ReusableSlot` can be used multiple
/// times. It's the user's responsibility to ensure a value that is rooted using a `ReusableSlot`
/// is not used after the slot has been reused.
#[derive(Clone, Copy)]
pub struct ReusableSlot<'scope>(*mut *mut c_void, PhantomData<&'scope ()>);

impl<'scope> ReusableSlot<'scope> {
    pub(crate) fn new<F: Frame<'scope>>(frame: &mut F) -> JlrsResult<Self> {
        unsafe {
            let slot = frame.reserve_slot(Private)?;
            Ok(ReusableSlot(slot, PhantomData))
        }
    }

    /// Root the given value in this slot.
    pub fn reset<'data: 'scope>(self, new: Value<'_, 'data>) -> ValueRef<'scope, 'data> {
        unsafe {
            let ptr = new.unwrap_non_null(Private).as_ptr();
            self.0.write(ptr.cast());
            ValueRef::wrap(ptr)
        }
    }
}
