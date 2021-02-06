use super::{frame::GcFrame, traits::frame::Frame};
use crate::{
    error::JlrsResult,
    value::{traits::private::Internal, UnrootedCallResult, UnrootedValue},
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
pub struct OutputScope<'scope, 'frame, 'borrow, F: Frame<'frame>>(
    pub(crate) &'borrow mut F,
    Output<'scope>,
    PhantomData<&'frame ()>,
);

impl<'scope, 'frame, 'borrow, F: Frame<'frame>> OutputScope<'scope, 'frame, 'borrow, F> {
    fn new(output: Output<'scope>, frame: &'borrow mut F) -> Self {
        OutputScope(frame, output, PhantomData)
    }

    /// Nest a `value_frame` and propagate the output to the new frame. See
    /// [`Scope::value_frame`] for more information.
    pub fn value_frame<'data, G>(self, func: G) -> JlrsResult<UnrootedValue<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>,
    {
        let mut frame = self.0.nest(0, Internal);
        let out = Output::new();
        func(out, &mut frame).map(|pv| UnrootedValue::new(pv.ptr()))
    }

    /// Nest a `value_frame` and propagate the output to the new frame. See
    /// [`Scope::value_frame`] for more information.
    pub fn value_frame_with_slots<'data, G>(
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
        let mut frame = self.0.nest(capacity, Internal);
        let out = Output::new();
        func(out, &mut frame).map(|pv| UnrootedValue::new(pv.ptr()))
    }

    /// Nest a `call_frame` and propagate the output to the new frame. See
    /// [`Scope::value_frame`] for more information.
    pub fn call_frame<'data, G>(
        self,
        func: G,
    ) -> JlrsResult<UnrootedCallResult<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'scope, 'data, 'inner>>,
    {
        let mut frame = self.0.nest(0, Internal);
        let out = Output::new();
        func(out, &mut frame).map(|pv| match pv {
            UnrootedCallResult::Ok(pv) => UnrootedCallResult::Ok(UnrootedValue::new(pv.ptr())),
            UnrootedCallResult::Err(pv) => UnrootedCallResult::Err(UnrootedValue::new(pv.ptr())),
        })
    }
    /// Nest a `call_frame` and propagate the output to the new frame. See
    /// [`Scope::value_frame`] for more information.
    pub fn call_frame_with_slots<'data, G>(
        self,
        capacity: usize,
        func: G,
    ) -> JlrsResult<UnrootedCallResult<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'scope, 'data, 'inner>>,
    {
        let mut frame = self.0.nest(capacity, Internal);
        let out = Output::new();
        func(out, &mut frame).map(|pv| match pv {
            UnrootedCallResult::Ok(pv) => UnrootedCallResult::Ok(UnrootedValue::new(pv.ptr())),
            UnrootedCallResult::Err(pv) => UnrootedCallResult::Err(UnrootedValue::new(pv.ptr())),
        })
    }
}
