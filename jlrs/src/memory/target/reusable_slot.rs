//! Reusable slots
//!
//! Reusable slots target a reserved slot in some frame. There are two variations,
//! [`ReusableSlot`] and  [`LocalReusableSlot`], both behave the same way, they only only target
//! different kinds of frame.
//!
//! When a reusable slot is taken by mutable reference it can be reused, the lifetime that is
//! considered the `'target` lifetime is the lifetime of the reusable slot. Because this means
//! that the data can become while it is in use, a `Ref` is returned as if an unrooting target
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
//!     let reusable_slot = frame.local_reusable_slot();
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
//!     let mut reusable_slot = frame.local_reusable_slot();
//!
//!     let _v = frame.local_scope::<_, 0>(|_| {
//!         // This data can be used until you leave the parent scope,
//!         // it will be rooted until the reusable slot is used again.
//!         Value::new(&mut reusable_slot, 2u64)
//!     });
//! });
//! # }
//! ```

use std::{cell::Cell, ffi::c_void, ptr::NonNull};

use crate::{
    data::managed::{Managed, Ref},
    memory::context::stack::Stack,
    private::Private,
};

/// An reusable slot that targets a [`GcFrame`].
///
/// See the [module-level docs] for more information.
///
/// [module-level docs]: crate::memory::target::output
/// [`GcFrame`]: crate::memory::target::frame::GcFrame

pub struct ReusableSlot<'target> {
    pub(crate) stack: &'target Stack,
    pub(crate) offset: usize,
}

impl<'scope> ReusableSlot<'scope> {
    #[inline]
    pub(crate) unsafe fn consume<'data, T: Managed<'scope, 'data>>(
        self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.stack.set_root(self.offset, ptr.cast());
        T::wrap_non_null(ptr, Private)
    }

    #[inline]
    pub(crate) unsafe fn temporary<'data, T: Managed<'scope, 'data>>(
        &mut self,
        ptr: NonNull<T::Wraps>,
    ) -> Ref<'scope, 'data, T> {
        self.stack.set_root(self.offset, ptr.cast());
        Ref::<T>::wrap(ptr)
    }
}

/// An reusable slot that targets a [`LocalGcFrame`].
///
/// See the [module-level docs] for more information.
///
/// [module-level docs]: crate::memory::target::output
/// [`LocalGcFrame`]: crate::memory::target::frame::LocalGcFrame
pub struct LocalReusableSlot<'target> {
    slot: &'target Cell<*mut c_void>,
}

impl<'target> LocalReusableSlot<'target> {
    #[inline]
    pub(crate) fn new(slot: &'target Cell<*mut c_void>) -> Self {
        LocalReusableSlot { slot }
    }

    #[inline]
    pub(crate) unsafe fn consume<'data, T: Managed<'target, 'data>>(
        self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.slot.set(ptr.as_ptr().cast());
        T::wrap_non_null(ptr, Private)
    }

    #[inline]
    pub(crate) unsafe fn temporary<'t, 'data, T: Managed<'target, 'data>>(
        &'t mut self,
        ptr: NonNull<T::Wraps>,
    ) -> Ref<'target, 'data, T> {
        self.slot.set(ptr.as_ptr().cast());
        Ref::<T>::wrap(ptr)
    }
}
