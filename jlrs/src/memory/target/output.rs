//! Outputs
//!
//! Outputs target a reserved slot in some frame. There are two variations, [`Output`] and
//! [`LocalOutput`], both behave the same way, they only only target different kinds of frame.
//!
//! When an output is taken by mutable reference it can be reused, the lifetime that is considered
//! the `'target` lifetime is the lifetime of the borrow rather than the lifetime of the `Output`.
//! This guarantees the data can only be used while it's guaranteed to be rooted.
//!
//! Examples:
//!
//! ```
//! # use jlrs::prelude::*;
//! # use jlrs::util::test::JULIA;
//! # fn main() {
//! # JULIA.with(|j| {
//! # let mut julia = j.borrow_mut();
//! # let mut frame = StackFrame::new();
//! # let mut julia = julia.instance(&mut frame);
//!
//! julia
//!     .scope(|mut frame| {
//!         let output = frame.output();
//!
//!         let _v = frame.scope(|_| {
//!             // The output has been allocated in the parent
//!             // scope's frame, so by using it as a target the
//!             // result can be returned from this subscope.
//!             Ok(Value::new(output, 1u64))
//!         })?;
//!
//!         Ok(())
//!     })
//!     .unwrap();
//! # });
//! # }
//! ```
//!
//! ```
//! # use jlrs::prelude::*;
//! # use jlrs::util::test::JULIA;
//! # fn main() {
//! # JULIA.with(|j| {
//! # let mut julia = j.borrow_mut();
//! # let mut frame = StackFrame::new();
//! # let mut julia = julia.instance(&mut frame);
//!
//! julia
//!     .scope(|mut frame| {
//!         let mut output = frame.output();
//!
//!         let _v = frame.scope(|_| {
//!             // _v1 can be used until the output is used again.
//!             let _v1 = Value::new(&mut output, 2u64);
//!
//!             Ok(Value::new(output, 1u64))
//!         })?;
//!
//!         Ok(())
//!     })
//!     .unwrap();
//! # });
//! # }
//! ```

use std::{cell::Cell, ffi::c_void, ptr::NonNull};

use crate::{data::managed::Managed, memory::context::stack::Stack, private::Private};

/// An output that targets a [`GcFrame`].
///
/// See the [module-level docs] for more information.
///
/// [module-level docs]: crate::memory::target::output
/// [`GcFrame`]: crate::memory::target::frame::GcFrame
pub struct Output<'target> {
    pub(crate) stack: &'target Stack,
    pub(crate) offset: usize,
}

impl<'scope> Output<'scope> {
    #[inline]
    pub(crate) unsafe fn consume<'data, T: Managed<'scope, 'data>>(
        self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.stack.set_root(self.offset, ptr.cast());
        T::wrap_non_null(ptr, Private)
    }

    #[inline]
    pub(crate) unsafe fn temporary<'target, 'data, T: Managed<'target, 'data>>(
        &'target mut self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.stack.set_root(self.offset, ptr.cast());
        T::wrap_non_null(ptr, Private)
    }
}

/// An output that targets a [`LocalGcFrame`].
///
/// See the [module-level docs] for more information.
///
/// [module-level docs]: crate::memory::target::output
/// [`LocalGcFrame`]: crate::memory::target::frame::LocalGcFrame
#[repr(transparent)]
pub struct LocalOutput<'target> {
    slot: &'target Cell<*mut c_void>,
}

impl<'target> LocalOutput<'target> {
    #[inline]
    pub(crate) fn new(slot: &'target Cell<*mut c_void>) -> Self {
        LocalOutput { slot }
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
    pub(crate) unsafe fn temporary<'t, 'data, T: Managed<'t, 'data>>(
        &'t mut self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.slot.set(ptr.as_ptr().cast());
        T::wrap_non_null(ptr, Private)
    }
}
