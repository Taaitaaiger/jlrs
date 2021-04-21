//! Root a value in an earlier frame.
//!
//! In order to prevent temporary values from remaining rooted, it's often desirable to call some
//! function or create a new value in a new scope and root the final result in the frame of the
//! current scope. This can be done with the methods like [`Scope::result_scope`] and
//! [`Scope::value_scope`] respectively. These methods take a closure that provides an `Output`
//! and a mutable reference to a frame. The frame can be used to root temporary values, before
//! converting the [`Output`] to an [`OutputScope`]. An [`OutputScope`] is a [`Scope`] that roots
//! the result in an earlier frame and can only be used once, the closure should immediately
//! return this result.
//!
//! [`Scope`]: crate::memory::traits::scope::Scope
//! [`Scope::result_scope`]: crate::memory::traits::scope::Scope::result_scope
//! [`Scope::value_scope`]: crate::memory::traits::scope::Scope::value_scope

use super::{frame::GcFrame, traits::frame::Frame};
use crate::{
    error::JlrsResult,
    private::Private,
    value::{UnrootedResult, UnrootedValue},
};
use std::marker::PhantomData;

/// An output that can be converted into an [`OutputScope`] to root a value in an earlier frame.
pub struct Output<'scope>(PhantomData<&'scope ()>);

impl<'scope> Output<'scope> {
    pub(crate) fn new() -> Self {
        Output(PhantomData)
    }

    /// Convert the output to an [`OutputScope`].
    pub fn into_scope<'frame, 'borrow, F: Frame<'frame>>(
        self,
        frame: &'borrow mut F,
    ) -> OutputScope<'scope, 'frame, 'borrow, F> {
        OutputScope::new(self, frame)
    }
}

/// A [`Scope`] that can be used once to root a value in an earlier frame.
///
/// [`Scope`]: crate::memory::traits::scope::Scope
pub struct OutputScope<'scope, 'frame, 'borrow, F: Frame<'frame>>(
    pub(crate) &'borrow mut F,
    Output<'scope>,
    PhantomData<&'frame ()>,
);

impl<'scope, 'frame, 'borrow, F: Frame<'frame>> OutputScope<'scope, 'frame, 'borrow, F> {
    fn new(output: Output<'scope>, frame: &'borrow mut F) -> Self {
        OutputScope(frame, output, PhantomData)
    }

    /// Create a new scope and root the output in the current frame. See [`Scope::value_scope`]
    /// for more information.
    ///
    /// [`Scope::value_scope`]: crate::memory::traits::scope::Scope::value_scope
    pub fn value_scope<'data, G>(self, func: G) -> JlrsResult<UnrootedValue<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>,
    {
        // Safe: frame is dropped
        let mut frame = unsafe { self.0.nest(0, Private) };
        let out = Output::new();
        func(out, &mut frame).map(|pv| UnrootedValue::new(pv.ptr()))
    }

    /// Create a new scope and root the output in the current frame. See
    /// [`Scope::value_scope_with_slots`] for more information.
    ///
    /// [`Scope::value_scope_with_slots`]: crate::memory::traits::scope::Scope::value_scope_with_slots
    pub fn value_scope_with_slots<'data, G>(
        self,
        capacity: usize,
        func: G,
    ) -> JlrsResult<UnrootedValue<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>,
    {
        // Safe: frame is dropped
        let mut frame = unsafe { self.0.nest(capacity, Private) };
        let out = Output::new();
        func(out, &mut frame).map(|pv| UnrootedValue::new(pv.ptr()))
    }

    /// Create a new scope and root the output in the current frame. See [`Scope::result_scope`]
    /// for more information.
    ///
    /// [`Scope::result_scope`]: crate::memory::traits::scope::Scope::result_scope
    pub fn result_scope<'data, G>(
        self,
        func: G,
    ) -> JlrsResult<UnrootedResult<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedResult<'scope, 'data, 'inner>>,
    {
        // Safe: frame is dropped
        let mut frame = unsafe { self.0.nest(0, Private) };
        let out = Output::new();
        func(out, &mut frame).map(|pv| match pv {
            UnrootedResult::Ok(pv) => UnrootedResult::Ok(UnrootedValue::new(pv.ptr())),
            UnrootedResult::Err(pv) => UnrootedResult::Err(UnrootedValue::new(pv.ptr())),
        })
    }

    /// Create a new scope and root the output in the current frame. See
    /// [`Scope::result_scope_with_slots`] for more information.
    ///
    /// [`Scope::result_scope_with_slots`]: crate::memory::traits::scope::Scope::result_scope_with_slots
    pub fn result_scope_with_slots<'data, G>(
        self,
        capacity: usize,
        func: G,
    ) -> JlrsResult<UnrootedResult<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedResult<'scope, 'data, 'inner>>,
    {
        // Safe: frame is dropped
        let mut frame = unsafe { self.0.nest(capacity, Private) };
        let out = Output::new();
        func(out, &mut frame).map(|pv| match pv {
            UnrootedResult::Ok(pv) => UnrootedResult::Ok(UnrootedValue::new(pv.ptr())),
            UnrootedResult::Err(pv) => UnrootedResult::Err(UnrootedValue::new(pv.ptr())),
        })
    }
}
