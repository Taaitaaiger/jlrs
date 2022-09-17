//! Root data in a parent scope.
//!
//! In order to allow temporary data to be freed by the GC when it's no longer in use, this
//! data should be allocated in a new scope. Because the data returned from a scope must outlive
//! that scope, data rooted in it can't be returned from it.
//!
//! Instead, Julia data that you want to return from a scope must be rooted in a parent scope.
//! This can be done by using an [`Output`]. An `Output` can be reserved in a frame by calling
//! [`GcFrame::output`].

use crate::{
    memory::{context::Stack, frame::Frame, scope::OutputScope},
    private::Private,
    wrappers::ptr::Wrapper,
};
use std::{ptr::NonNull, cell::RefCell};

use super::ledger::Ledger;

/// A reserved slot in a frame.
///
/// A new `Output` can be created by calling [`GcFrame::output`]. `Output` implements
/// [`PartialScope`], not [`Scope`]. It can be upgraded to an [`OutputScope`], which does
/// implement `Scope`, by calling [`Output::into_scope`].
///
/// [`Scope`]: crate::memory::scope::Scope
/// [`PartialScope`]: crate::memory::scope::PartialScope
/// [`OutputScope`]: crate::memory::scope::OutputScope
pub struct Output<'target> {
    context: &'target Stack,
    pub(crate) ledger: &'target RefCell<Ledger>,
    offset: usize,
}

impl<'target> Output<'target> {
    /// Convert the `Output` and a frame to an `OutputScope`.
    pub fn into_scope<'frame, 'borrow, F: Frame<'frame>>(
        self,
        frame: &'borrow mut F,
    ) -> OutputScope<'target, 'frame, 'borrow, F> {
        OutputScope::new(self, frame)
    }

    // Safety: slot must have been reserved in _frame
    pub(crate) unsafe fn new(context: &'target Stack, ledger: &'target RefCell<Ledger>, offset: usize) -> Self {
        Output { context, ledger, offset }
    }

    // Safety: value must point to valid Jula data
    pub(crate) unsafe fn set_root<'data, T: Wrapper<'target, 'data>>(
        self,
        value: NonNull<T::Wraps>,
    ) -> T {
        self.context.set_root(self.offset, value.cast());
        T::wrap_non_null(value, Private)
    }
}
