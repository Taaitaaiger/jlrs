use std::{
    ffi::c_void,
    mem::MaybeUninit,
    sync::atomic::{AtomicPtr, Ordering},
};

use jlrs_sys::jlrs_gc_wb;

use crate::{
    data::managed::{Managed, Weak},
    private::Private,
};

/// Mutate Julia data atomically
pub unsafe trait WriteSlotAtomic<'data>: private::ParentPtr {
    /// Insert a write barrier and atomically write a managed value to a slot.
    ///
    /// This function must be called whenever a new managed value is written to a field of another
    /// managed value. `val` is atomically written to the atomic pointer returned by `slot`.
    ///
    /// Safety:
    ///
    ///  - The Julia type of `self` must be mutable.
    ///  - No invariants of the Julia type of `self` must be broken.
    ///  - `slot` must return either a field of `self`, or a slot in a non-Julia type field of `self`.
    ///  - The slot must be an atomic slot.
    ///
    /// An example of the latter option in the last requirement is a foreign type with a `Vec` of
    /// Julia values. If a new element is pushed, `slot` must return the slot of the vec that the
    /// data will be written to.
    unsafe fn write_slot_atomic<'v_scope, V>(
        &self,
        slot: impl Fn(&Self) -> &AtomicPtr<c_void>,
        val: Option<V>,
        ordering: Ordering,
    ) where
        V: Managed<'v_scope, 'data>,
    {
        let parent = self.parent_ptr(Private);
        let slot = slot(self);
        let slot_ptr = slot as *const _ as *mut _;

        let val_ptr = val.map(|v| v.unwrap(Private).cast()).unwrap_or_default();
        unsafe { jlrs_gc_wb(parent as *mut _, slot_ptr, val_ptr) }

        slot.store(val_ptr, ordering);
    }
}

/// Mutate Julia data
pub unsafe trait WriteSlot<'data>: WriteSlotAtomic<'data> {
    /// Insert a write barrier and write a managed value to a slot.
    ///
    /// This function must be called whenever a new managed value is written to a field of another
    /// managed value. `val` is written to the mutable reference returned by `slot`.
    ///
    /// Safety:
    ///
    ///  - The Julia type of `self` must be mutable.
    ///  - No invariants of the Julia type of `self` must be broken.
    ///  - `slot` must return either a field of `self`, or a slot in a non-Julia type field of `self`.
    ///  - The slot must not be an atomic slot.
    ///
    /// An example of the latter option in the last requirement is a foreign type with a `Vec` of
    /// Julia values. If a new element is pushed, `slot` must return the slot of the vec that the
    /// data will be written to.
    unsafe fn write_slot<'v_scope, V>(
        &mut self,
        slot: impl Fn(&mut Self) -> &mut MaybeUninit<Option<Weak<'v_scope, 'data, V>>>,
        val: Option<V>,
    ) where
        V: Managed<'v_scope, 'data>,
    {
        let parent = self.parent_ptr(Private);
        let slot = slot(self);
        let slot_ptr = slot as *mut _ as *mut _;

        let val_ptr = val.map(|v| v.unwrap(Private).cast()).unwrap_or_default();
        unsafe { jlrs_gc_wb(parent as *mut _, slot_ptr, val_ptr) }

        slot.write(val.map(|v| v.as_weak()));
    }
}

pub(crate) mod private {
    use std::ffi::c_void;

    use crate::private::Private;

    pub unsafe trait ParentPtr {
        fn parent_ptr(&self, _: Private) -> *const c_void;
    }
}
