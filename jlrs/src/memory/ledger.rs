use std::{cell::RefCell, ops::Range};

use crate::{
    error::AccessError,
    prelude::{Array, JlrsResult},
    wrappers::ptr::array::{
        dimensions::Dims,
        tracked::{ArrayWrapper, TrackedArray, TrackedArrayMut},
    },
};

// Adapted from neon:
// https://github.com/neon-bindings/neon/blob/09c04b3129798b16021549352c74323f629d5bb0/crates/neon/src/types/buffer/lock.rs

#[derive(Debug, Default)]
pub struct Ledger {
    pub(crate) owned: Vec<Range<*const u8>>,
    pub(crate) shared: Vec<Range<*const u8>>,
    // call_owned
    // call_shared
}

impl Ledger {
    // Convert a slice of arbitrary type and size to a range of bytes addresses
    //
    // Alignment does not matter because we are only interested in bytes.
    pub fn slice_to_range(data: Array) -> Range<*const u8> {
        let start = data.data_ptr().cast::<u8>();
        let len = data.element_size() * unsafe { data.dimensions() }.size();
        let end = unsafe { start.add(len) };

        start..end
    }

    // Dynamically check a slice conforms to borrow rules before returning by
    // using interior mutability of the ledger.
    pub fn try_borrow_array<'borrow, 'scope, 'data, T>(
        ledger: &'borrow RefCell<Self>,
        data: T,
    ) -> JlrsResult<TrackedArray<'borrow, 'scope, 'data, T>>
    where
        T: ArrayWrapper<'scope, 'data>,
    {
        ledger.borrow_mut().try_add_borrow(data)?;
        unsafe { Ok(TrackedArray::new(ledger, data)) }
    }

    // Dynamically check a mutable slice conforms to borrow rules before returning by
    // using interior mutability of the ledger.
    pub fn try_borrow_array_mut<'borrow, 'scope, 'data, T>(
        ledger: &'borrow RefCell<Self>,
        data: T,
    ) -> JlrsResult<TrackedArrayMut<'borrow, 'scope, 'data, T>>
    where
        T: ArrayWrapper<'scope, 'data>,
    {
        ledger.borrow_mut().try_add_borrow_mut(data)?;
        unsafe { Ok(TrackedArrayMut::new(ledger, data)) }
    }

    // Try to add an immutable borrow to the ledger
    fn try_add_borrow<'scope, 'data, T: ArrayWrapper<'scope, 'data>>(
        &mut self,
        data: T,
    ) -> JlrsResult<()> {
        let range = data.data_range();

        // Check if the borrow overlaps with any active mutable borrow
        check_overlap(&self.owned, &range)?;

        // Record a record of the immutable borrow
        self.shared.push(range);

        Ok(())
    }

    // Try to add an immutable borrow to the ledger
    pub(crate) unsafe fn clone_shared<'borrow, 'scope, 'data, T: ArrayWrapper<'scope, 'data>>(
        ledger: &'borrow RefCell<Self>,
        data: T,
    ) {
        let mut ledger = ledger.borrow_mut();
        let range = data.data_range();
        ledger.shared.push(range);
    }

    // Try to add a mutable borrow to the ledger
    fn try_add_borrow_mut<'scope, 'data, T: ArrayWrapper<'scope, 'data>>(
        &mut self,
        data: T,
    ) -> JlrsResult<()> {
        let range = data.data_range();

        // Check if the borrow overlaps with any active mutable borrow
        check_overlap(&self.owned, &range)?;

        // Check if the borrow overlaps with any active immutable borrow
        check_overlap(&self.shared, &range)?;

        // Record a record of the mutable borrow
        self.owned.push(range);

        Ok(())
    }
}

fn is_disjoint(a: &Range<*const u8>, b: &Range<*const u8>) -> bool {
    b.start >= a.end || a.start >= b.end
}

fn check_overlap(existing: &[Range<*const u8>], range: &Range<*const u8>) -> JlrsResult<()> {
    if existing.iter().all(|i| is_disjoint(i, range)) {
        Ok(())
    } else {
        Err(AccessError::BorrowError)?
    }
}
