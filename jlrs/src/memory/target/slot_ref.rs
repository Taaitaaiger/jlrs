//! A reference to a slot in a frame.

use std::{cell::Cell, os::raw::c_void};

use crate::memory::context::stack::Stack;

/// A reference to a slot in a [`LocalGcFrame`].
///
/// [`LocalGcFrame`]: crate::memory::target::frame::LocalGcFrame
pub struct LocalSlotRef<'target> {
    slot: &'target Cell<*mut c_void>,
}

impl<'target> LocalSlotRef<'target> {
    #[inline]
    pub(crate) fn new(slot: &'target Cell<*mut c_void>) -> Self {
        LocalSlotRef { slot }
    }
}

/// A reference to a slot in a [`GcFrame`].
///
/// [`GcFrame`]: crate::memory::target::frame::GcFrame
pub struct StackSlotRef<'target> {
    stack: &'target Stack,
    offset: usize,
}

impl<'target> StackSlotRef<'target> {
    #[inline]
    pub(crate) fn new(stack: &'target Stack, offset: usize) -> Self {
        StackSlotRef { stack, offset }
    }
}

/// A reference to a slot in a frame.
pub unsafe trait SlotRef: private::SlotRefPriv {}

unsafe impl<'target> SlotRef for LocalSlotRef<'target> {}

unsafe impl<'target> SlotRef for StackSlotRef<'target> {}

mod private {
    use std::{os::raw::c_void, ptr::NonNull};

    use super::{LocalSlotRef, StackSlotRef};
    use crate::private::Private;

    pub unsafe trait SlotRefPriv {
        unsafe fn set(&self, data: NonNull<c_void>, _: Private);
    }

    unsafe impl<'target> SlotRefPriv for StackSlotRef<'target> {
        #[inline]
        unsafe fn set(&self, data: NonNull<c_void>, _: Private) {
            self.stack.set_root(self.offset, data.cast());
        }
    }

    unsafe impl<'target> SlotRefPriv for LocalSlotRef<'target> {
        #[inline]
        unsafe fn set(&self, data: NonNull<c_void>, _: Private) {
            self.slot.set(data.as_ptr().cast());
        }
    }
}
