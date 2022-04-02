//! Root data in parent scope.
//!
//! In order to allow temporary data to be freed by the GC when it's no longer in use, these
//! temporaries should be allocated in a new scope. Because the data returned from a scope must
//! outlive that scope, data rooted in it can't be returned from it.
//!
//! Instead, Julia data that you want to return from a scope must be rooted in a parent scope. For
//! this purpose [`Output`] and [`OutputScope`] can be used. An `Output` can be reserved in a
//! frame by calling [`Frame::reserve_output`], an `Output` can be turned into an `OutputScope` by
//! calling [`Output::into_scope`].

use crate::{
    prelude::{Frame, Wrapper},
    private::Private,
};
use std::{cell::Cell, ffi::c_void, marker::PhantomData, ptr::NonNull};

/// A reserved slot in a frame. A new output can be created by calling [`Frame::reserve_output`].
/// While an `Output` doesn't implement [`Scope`], it does implement [`PartialScope`].
///
/// [`Scope`]: crate::memory::scope::Scope
/// [`PartialScope`]: crate::memory::scope::PartialScope
pub struct Output<'target> {
    output: *const Cell<*mut c_void>,
    _marker: PhantomData<fn(&'target mut ()) -> ()>,
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

    pub(crate) fn new<F: Frame<'target>>(_frame: &F, output: *const Cell<*mut c_void>) -> Self {
        Output {
            output,
            _marker: PhantomData,
        }
    }

    pub(crate) fn set_root<'data, X: Wrapper<'target, 'data>>(self, value: NonNull<X::Wraps>) {
        unsafe {
            let cell = &*self.output;
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
    pub(crate) fn set_root<'data, X: Wrapper<'target, 'data>>(self, value: NonNull<X::Wraps>) -> X {
        unsafe {
            self.output.set_root::<X>(value);
            X::wrap_non_null(value, Private)
        }
    }
}
