//! Convert a rooted value or function call result to its unrooted counterpart.
//!
//! When the async runtime is used, it can be useful to use [`AsyncGcFrame::async_value_frame`] or 
//! [`AsyncGcFrame::async_call_frame`] in order to ensure temporary values are unrooted as fast 
//! as possible. Because the result of an async Julia function call is rooted, it must be unrooted 
//! before it is returned from the closure.

use super::frame::Frame;
use crate::{
    error::CallResult,
    memory::output::OutputScope,
    value::{UnrootedCallResult, UnrootedValue, Value},
};

/// Converts a [`Value`] or [`CallResult`] to their unrooted counterparts.
pub trait AsUnrooted<'scope, 'frame, 'data, 'inner>: private::AsUnrooted {
    type Unrooted;
    fn as_unrooted<F: Frame<'frame>>(
        self,
        _output: OutputScope<'scope, 'frame, 'inner, F>,
    ) -> Self::Unrooted;
}

impl<'scope, 'frame, 'data, 'inner> AsUnrooted<'scope, 'frame, 'data, 'inner>
    for Value<'frame, 'data>
{
    type Unrooted = UnrootedValue<'scope, 'data, 'inner>;
    fn as_unrooted<F: Frame<'frame>>(
        self,
        _output: OutputScope<'scope, 'frame, 'inner, F>,
    ) -> Self::Unrooted {
        unsafe { UnrootedValue::new(self.ptr()) }
    }
}

impl<'scope, 'frame, 'data, 'inner> AsUnrooted<'scope, 'frame, 'data, 'inner>
    for CallResult<'frame, 'data>
{
    type Unrooted = UnrootedCallResult<'scope, 'data, 'inner>;
    fn as_unrooted<F: Frame<'frame>>(
        self,
        output: OutputScope<'scope, 'frame, 'inner, F>,
    ) -> Self::Unrooted {
        match self {
            Ok(v) => UnrootedCallResult::Ok(v.as_unrooted(output)),
            Err(v) => UnrootedCallResult::Err(v.as_unrooted(output)),
        }
    }
}

mod private {
    use crate::{prelude::CallResult, value::Value};

    pub trait AsUnrooted {}
    impl<'frame, 'data> AsUnrooted for Value<'frame, 'data> {}
    impl<'frame, 'data> AsUnrooted for CallResult<'frame, 'data> {}
}