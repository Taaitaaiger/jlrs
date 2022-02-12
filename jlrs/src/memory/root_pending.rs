use super::frame::Frame;
use crate::{
    error::{JlrsResult, JuliaResult},
    memory::output::{PendingResult, PendingValue},
    private::Private,
    wrappers::ptr::value::Value,
};

pub(crate) trait RootPending<'frame, 'data>: Sized {
    type ClosureOutput;

    unsafe fn root_pending<F: Frame<'frame>>(
        frame: &mut F,
        value: Self::ClosureOutput,
    ) -> JlrsResult<Self>;
}

impl<'frame, 'data> RootPending<'frame, 'data> for JuliaResult<'frame, 'data> {
    type ClosureOutput = PendingResult<'frame, 'data>;

    unsafe fn root_pending<F: Frame<'frame>>(
        frame: &mut F,
        val: Self::ClosureOutput,
    ) -> JlrsResult<Self> {
        match val {
            Ok(v) => Ok(Ok(frame.push_root(v.unwrap_non_null(), Private).unwrap())),
            Err(e) => Ok(Err(frame.push_root(e.unwrap_non_null(), Private).unwrap())),
        }
    }
}

impl<'frame, 'data> RootPending<'frame, 'data> for Value<'frame, 'data> {
    type ClosureOutput = PendingValue<'frame, 'data>;

    unsafe fn root_pending<F: Frame<'frame>>(
        frame: &mut F,
        val: Self::ClosureOutput,
    ) -> JlrsResult<Self> {
        Ok(frame.push_root(val.unwrap_non_null(), Private).unwrap())
    }
}
