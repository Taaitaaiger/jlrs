//! Outputs
//!
//! Outputs target a reserved slot in some frame.
//!
//! When an output is taken by mutable reference it can be reused, the lifetime that is considered
//! the `'target` lifetime is the lifetime of the borrow rather than the lifetime of the `Output`.
//! This guarantees the data can only be used while it's guaranteed to be rooted.
//!
//! Examples:
//!
//! ```
//! # use jlrs::prelude::*;
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//!
//! julia.local_scope::<_, 1>(|mut frame| {
//!     let output = frame.output();
//!
//!     let _v = frame.local_scope::<_, 0>(|_| {
//!         // The output has been allocated in the parent
//!         // scope's frame, so by using it as a target the
//!         // result can be returned from this subscope.
//!         Value::new(output, 1u64)
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
//!     let mut output = frame.output();
//!
//!     let _v = frame.local_scope::<_, 0>(|_| {
//!         // _v1 can be used until the output is used again.
//!         let _v1 = Value::new(&mut output, 2u64);
//!
//!         Value::new(output, 1u64)
//!     });
//! });
//! # }
//! ```

use std::{marker::PhantomData, ptr::NonNull};

use super::slot_ref::SlotRef;
use crate::{data::managed::Managed, private::Private};

/// An output that targets a slot in some frame.
///
/// See the [module-level docs] for more information.
///
/// [module-level docs]: crate::memory::target::output
/// [`GcFrame`]: crate::memory::target::frame::GcFrame
pub struct Output<'target, S> {
    slot: S,
    _marker: PhantomData<&'target ()>,
}

impl<'target, S: SlotRef> Output<'target, S> {
    pub(crate) unsafe fn new(slot: S) -> Self {
        Output {
            slot,
            _marker: PhantomData,
        }
    }
}

impl<'target, S: SlotRef> Output<'target, S> {
    #[inline]
    pub(crate) unsafe fn consume<'data, T: Managed<'target, 'data>>(
        self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.slot.set(ptr.cast(), Private);
        T::wrap_non_null(ptr, Private)
    }

    #[inline]
    pub(crate) unsafe fn temporary<'t, 'data, T: Managed<'t, 'data>>(
        &'t mut self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.slot.set(ptr.cast(), Private);
        T::wrap_non_null(ptr, Private)
    }
}
