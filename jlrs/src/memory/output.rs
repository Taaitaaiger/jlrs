//! Root a value in the frame of an earlier scope.
//!
//! In order to prevent temporary values from remaining rooted, it's often desirable to call some
//! function or create a new value in a new scope but root the final result in the frame of the
//! current scope. This can be done with the methods like [`Scope::result_scope`] and
//! [`Scope::value_scope`] respectively. These methods take a closure that provides an `Output`
//! and a mutable reference to a frame. The frame can be used to root temporary values, the
//! [`Output`] can be converted to an [`OutputScope`]. An [`OutputScope`] is a [`Scope`] that
//! roots the result in an earlier frame and can only be used once.
//!
//! [`Scope`]: crate::memory::scope::Scope
//! [`Scope::result_scope`]: crate::memory::scope::Scope::result_scope
//! [`Scope::value_scope`]: crate::memory::scope::Scope::value_scope

use jl_sys::jl_value_t;

use super::{frame::Frame, frame::GcFrame};
use crate::{error::JlrsResult, private::Private};
use std::{marker::PhantomData, ptr::NonNull};

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
/// [`Scope`]: crate::memory::scope::Scope
pub struct OutputScope<'scope, 'frame, 'borrow, F: Frame<'frame>>(
    pub(crate) &'borrow mut F,
    Output<'scope>,
    PhantomData<&'frame ()>,
);

impl<'scope, 'frame, 'borrow, F: Frame<'frame>> OutputScope<'scope, 'frame, 'borrow, F> {
    fn new(output: Output<'scope>, frame: &'borrow mut F) -> Self {
        OutputScope(frame, output, PhantomData)
    }

    pub(crate) fn value_scope<'data, G>(
        self,
        func: G,
    ) -> JlrsResult<OutputValue<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'scope, 'data, 'inner>>,
    {
        let mut frame = self.0.nest(0, Private);
        let out = Output::new();
        func(out, &mut frame).map(|pv| OutputValue::wrap_non_null(pv.unwrap_non_null()))
    }

    pub(crate) fn value_scope_with_slots<'data, G>(
        self,
        capacity: usize,
        func: G,
    ) -> JlrsResult<OutputValue<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'scope, 'data, 'inner>>,
    {
        let mut frame = self.0.nest(capacity, Private);
        let out = Output::new();
        func(out, &mut frame).map(|pv| OutputValue::wrap_non_null(pv.unwrap_non_null()))
    }

    pub(crate) fn result_scope<'data, G>(
        self,
        func: G,
    ) -> JlrsResult<OutputResult<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputResult<'scope, 'data, 'inner>>,
    {
        let mut frame = self.0.nest(0, Private);
        let out = Output::new();
        func(out, &mut frame).map(|pv| match pv {
            OutputResult::Ok(pv) => {
                OutputResult::Ok(OutputValue::wrap_non_null(pv.unwrap_non_null()))
            }
            OutputResult::Err(pv) => {
                OutputResult::Err(OutputValue::wrap_non_null(pv.unwrap_non_null()))
            }
        })
    }

    pub(crate) fn result_scope_with_slots<'data, G>(
        self,
        capacity: usize,
        func: G,
    ) -> JlrsResult<OutputResult<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputResult<'scope, 'data, 'inner>>,
    {
        let mut frame = self.0.nest(capacity, Private);
        let out = Output::new();
        func(out, &mut frame).map(|pv| match pv {
            OutputResult::Ok(pv) => {
                OutputResult::Ok(OutputValue::wrap_non_null(pv.unwrap_non_null()))
            }
            OutputResult::Err(pv) => {
                OutputResult::Err(OutputValue::wrap_non_null(pv.unwrap_non_null()))
            }
        })
    }
}

/// A `Value` that has not yet been rooted.
#[repr(transparent)]
pub struct OutputValue<'frame, 'data, 'borrow>(
    NonNull<jl_value_t>,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
    PhantomData<&'borrow ()>,
);

impl<'frame, 'data, 'borrow> OutputValue<'frame, 'data, 'borrow> {
    pub(crate) fn into_pending(self) -> PendingValue<'frame, 'data> {
        PendingValue::wrap_non_null(self.0)
    }

    pub(crate) fn unwrap_non_null(self) -> NonNull<jl_value_t> {
        self.0
    }

    pub(crate) fn wrap_non_null(contents: NonNull<jl_value_t>) -> Self {
        OutputValue(contents, PhantomData, PhantomData, PhantomData)
    }
}

/// A `JuliaResult` that has not yet been rooted.
pub enum OutputResult<'frame, 'data, 'inner> {
    Ok(OutputValue<'frame, 'data, 'inner>),
    Err(OutputValue<'frame, 'data, 'inner>),
}

impl<'frame, 'data, 'inner> OutputResult<'frame, 'data, 'inner> {
    pub(crate) fn into_pending(self) -> PendingResult<'frame, 'data> {
        match self {
            Self::Ok(pov) => Ok(pov.into_pending()),
            Self::Err(pov) => Err(pov.into_pending()),
        }
    }

    /// Returns true if the result is an exception.
    pub fn is_exception(&self) -> bool {
        match self {
            Self::Ok(_) => false,
            Self::Err(_) => true,
        }
    }
}

#[repr(transparent)]
pub(crate) struct PendingValue<'frame, 'data>(
    NonNull<jl_value_t>,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
);

impl<'frame, 'data> PendingValue<'frame, 'data> {
    pub(crate) fn unwrap_non_null(self) -> NonNull<jl_value_t> {
        self.0
    }

    pub(crate) fn wrap_non_null(contents: NonNull<jl_value_t>) -> Self {
        PendingValue(contents, PhantomData, PhantomData)
    }
}

pub(crate) type PendingResult<'frame, 'data> =
    Result<PendingValue<'frame, 'data>, PendingValue<'frame, 'data>>;
