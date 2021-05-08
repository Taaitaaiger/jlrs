//! Convert a rooted value or function call result to its unrooted counterpart.
//!
//! When the async runtime is used, it can be useful to use [`AsyncGcFrame::async_value_scope`] or
//! [`AsyncGcFrame::async_result_scope`] in order to ensure temporary values can be freed by the
//! garbage collector as fast as possible. Because the result of an async Julia function call is
//! rooted, it must be unrooted before it is returned from the closure.
//!
//! [`AsyncGcFrame::async_result_scope`]: crate::memory::frame::AsyncGcFrame::async_result_scope
//! [`AsyncGcFrame::async_value_scope`]: crate::memory::frame::AsyncGcFrame::async_value_scope

use super::frame::Frame;
use crate::{
    error::JuliaResult,
    memory::output::OutputScope,
    value::{UnrootedResult, UnrootedValue, Value},
};

/// Converts a [`Value`] or [`JuliaResult`] to their unrooted counterparts.
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
        unsafe { UnrootedValue::new(self.inner().as_ptr()) }
    }
}

impl<'scope, 'frame, 'data, 'inner> AsUnrooted<'scope, 'frame, 'data, 'inner>
    for JuliaResult<'frame, 'data>
{
    type Unrooted = UnrootedResult<'scope, 'data, 'inner>;
    fn as_unrooted<F: Frame<'frame>>(
        self,
        output: OutputScope<'scope, 'frame, 'inner, F>,
    ) -> Self::Unrooted {
        match self {
            Ok(v) => UnrootedResult::Ok(v.as_unrooted(output)),
            Err(v) => UnrootedResult::Err(v.as_unrooted(output)),
        }
    }
}

mod private {
    use crate::{error::JuliaResult, value::Value};

    pub trait AsUnrooted {}
    impl<'frame, 'data> AsUnrooted for Value<'frame, 'data> {}
    impl<'frame, 'data> AsUnrooted for JuliaResult<'frame, 'data> {}
}
