//! Root data in a parent scope.
//!
//! In order to allow temporary data to be freed by the GC when it's no longer in use, this
//! data should be allocated in a new scope. Because the data returned from a scope must outlive
//! that scope, data rooted in it can't be returned from it.
//!
//! Instead, Julia data that you want to return from a scope must be rooted in a parent scope.
//! Thiscan be done by using an [`Output`] or [`OutputScope`]. An `Output` can be reserved in a
//! frame by calling [`Frame::reserve_output`], it can be turned into an `OutputScope` by calling
//! [`Output::into_scope`]. `Output` only implements [`PartialScope`], while `OutputScope`
//! implements both [`Scope`] and [`OutputScope`]
//!
//! [`PartialScope`]: crate::memory::scope::PartialScope
//! [`Scope`]: crate::memory::scope::Scope

use crate::{memory::frame::Frame, private::Private, wrappers::ptr::Wrapper};
use std::{cell::Cell, ffi::c_void, marker::PhantomData, ptr::NonNull};

/// A reserved slot in a frame.
///
/// A new `Output` can be created by calling [`Frame::reserve_output`]. `Output` implements
/// [`PartialScope`], not [`Scope`]. It can be upgraded to an [`OutputScope`], which does
/// implement `Scope`, by calling [`Output::into_scope`].
///
/// [`Scope`]: crate::memory::scope::Scope
/// [`PartialScope`]: crate::memory::scope::PartialScope
pub struct Output<'target> {
    slot: *const Cell<*mut c_void>,
    _marker: PhantomData<fn(&'target mut ())>,
}

impl<'target> Output<'target> {
    /// Convert the `Output` and a frame to an `OutputScope`.
    pub fn into_scope<'frame, 'borrow, F: Frame<'frame>>(
        self,
        frame: &'borrow mut F,
    ) -> OutputScope<'target, 'frame, 'borrow, F> {
        OutputScope {
            output: self,
            frame,
            _marker: PhantomData,
        }
    }

    pub(crate) fn new<F: Frame<'target>>(_frame: &F, slot: *const Cell<*mut c_void>) -> Self {
        Output {
            slot,
            _marker: PhantomData,
        }
    }

    pub(crate) fn set_root<'data, T: Wrapper<'target, 'data>>(self, value: NonNull<T::Wraps>) {
        unsafe {
            let cell = &*self.slot;
            cell.set(value.as_ptr().cast());
        }
    }
}

/// A [`Scope`] that roots a result using an [`Output`].
///
/// [`Scope`]: crate::memory::scope::Scope
pub struct OutputScope<'target, 'current, 'borrow, F: Frame<'current>> {
    pub(crate) output: Output<'target>,
    pub(crate) frame: &'borrow mut F,
    _marker: PhantomData<&'current ()>,
}

impl<'target, 'current, 'borrow, F: Frame<'current>> OutputScope<'target, 'current, 'borrow, F> {
    pub(crate) fn set_root<'data, T: Wrapper<'target, 'data>>(self, value: NonNull<T::Wraps>) -> T {
        unsafe {
            self.output.set_root::<T>(value);
            T::wrap_non_null(value, Private)
        }
    }
}
