//! Reusable slots
//!
//! Reusable slots target a reserved slot in some frame.
//!
//! When a reusable slot is taken by mutable reference it can be reused, the lifetime that is
//! considered the `'target` lifetime is the lifetime of the reusable slot. Because this means
//! that the data can become while it is in use, a `Weak` is returned as if an unrooting target
//! has been used.
//!
//! Examples:
//!
//! ```
//! # use jlrs::prelude::*;
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//!
//! julia.local_scope::<_, 1>(|mut frame| {
//!     let reusable_slot = frame.reusable_slot();
//!
//!     let _v = frame.local_scope::<_, 0>(|_| {
//!         // The reusable slot has been allocated in the parent
//!         // scope's frame, so by using it as a target the
//!         // result can be returned from this subscope.
//!         Value::new(reusable_slot, 1u64)
//!     });
//! });
//! # }
//! ```
//!
//! ```
//! # use jlrs::prelude::*;
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//!
//! julia.local_scope::<_, 1>(|mut frame| {
//!     let mut reusable_slot = frame.reusable_slot();
//!
//!     let _v = frame.local_scope::<_, 0>(|_| {
//!         // This data can be used until you leave the parent scope,
//!         // it will be rooted until the reusable slot is used again.
//!         Value::new(&mut reusable_slot, 2u64)
//!     });
//! });
//! # }
//! ```

use std::{marker::PhantomData, ptr::NonNull};

use super::slot_ref::SlotRef;
use crate::{
    data::managed::{Managed, Weak},
    private::Private,
};

/// A reusable slot.
///
/// See the [module-level docs] for more information.
///
/// [module-level docs]: crate::memory::target::output

pub struct ReusableSlot<'target, S> {
    slot: S,
    _marker: PhantomData<&'target ()>,
}

impl<'target, S: SlotRef> ReusableSlot<'target, S> {
    pub(crate) unsafe fn new(slot: S) -> Self {
        ReusableSlot {
            slot,
            _marker: PhantomData,
        }
    }
}

impl<'target, S: SlotRef> ReusableSlot<'target, S> {
    #[inline]
    pub(crate) unsafe fn consume<'data, T: Managed<'target, 'data>>(
        self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.slot.set(ptr.cast(), Private);
        T::wrap_non_null(ptr, Private)
    }

    #[inline]
    pub(crate) unsafe fn temporary<'t, 'data, T: Managed<'target, 'data>>(
        &'t mut self,
        ptr: NonNull<T::Wraps>,
    ) -> Weak<'target, 'data, T> {
        self.slot.set(ptr.cast(), Private);
        Weak::<T>::wrap(ptr)
    }
}
