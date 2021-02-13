use crate::value::{PendingCallResult, PendingValue};
use crate::{
    error::{CallResult, JlrsResult},
    value::{traits::private::Internal, Value},
};

use super::frame::Frame;
pub(crate) trait Root<'frame, 'data>: Sized {
    type ClosureOutput;

    unsafe fn root<F: Frame<'frame>>(frame: &mut F, value: Self::ClosureOutput)
        -> JlrsResult<Self>;
}

impl<'frame, 'data> Root<'frame, 'data> for CallResult<'frame, 'data> {
    type ClosureOutput = PendingCallResult<'frame, 'data>;

    unsafe fn root<F: Frame<'frame>>(frame: &mut F, val: Self::ClosureOutput) -> JlrsResult<Self> {
        match val {
            Ok(v) => frame
                .push_root(v.inner(), Internal)
                .map(|v| Ok(v))
                .map_err(Into::into),
            Err(e) => frame
                .push_root(e.inner(), Internal)
                .map(|v| Err(v))
                .map_err(Into::into),
        }
    }
}

impl<'frame, 'data> Root<'frame, 'data> for Value<'frame, 'data> {
    type ClosureOutput = PendingValue<'frame, 'data>;

    unsafe fn root<F: Frame<'frame>>(frame: &mut F, val: Self::ClosureOutput) -> JlrsResult<Self> {
        frame.push_root(val.inner(), Internal).map_err(Into::into)
    }
}
