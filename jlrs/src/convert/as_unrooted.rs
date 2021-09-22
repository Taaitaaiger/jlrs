//! Convert a rooted value or result to its unrooted counterpart.
//!
//! When the async runtime is used, it can be useful to use [`AsyncGcFrame::async_value_scope`] or
//! [`AsyncGcFrame::async_result_scope`] in order to ensure temporary values can be freed by the
//! garbage collector as fast as possible. Because the result of an async Julia function call is
//! rooted, it must be unrooted before it is returned from the closure.
//!
//! [`AsyncGcFrame::async_result_scope`]: crate::extensions::multitask::async_frame::AsyncGcFrame::async_result_scope
//! [`AsyncGcFrame::async_value_scope`]: crate::extensions::multitask::async_frame::AsyncGcFrame::async_value_scope

use crate::{
    error::JuliaResult,
    memory::{
        frame::Frame,
        output::{OutputResult, OutputScope, OutputValue},
    },
    private::Private,
    wrappers::ptr::{private::Wrapper, value::Value},
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
    type Unrooted = OutputValue<'scope, 'data, 'inner>;
    #[inline(always)]
    fn as_unrooted<F: Frame<'frame>>(
        self,
        _output: OutputScope<'scope, 'frame, 'inner, F>,
    ) -> Self::Unrooted {
        OutputValue::wrap_non_null(self.unwrap_non_null(Private))
    }
}

impl<'scope, 'frame, 'data, 'inner> AsUnrooted<'scope, 'frame, 'data, 'inner>
    for JuliaResult<'frame, 'data>
{
    type Unrooted = OutputResult<'scope, 'data, 'inner>;
    #[inline(always)]
    fn as_unrooted<F: Frame<'frame>>(
        self,
        output: OutputScope<'scope, 'frame, 'inner, F>,
    ) -> Self::Unrooted {
        match self {
            Ok(v) => OutputResult::Ok(v.as_unrooted(output)),
            Err(v) => OutputResult::Err(v.as_unrooted(output)),
        }
    }
}

mod private {
    use crate::{error::JuliaResult, wrappers::ptr::value::Value};

    pub trait AsUnrooted {}
    impl<'frame, 'data> AsUnrooted for Value<'frame, 'data> {}
    impl<'frame, 'data> AsUnrooted for JuliaResult<'frame, 'data> {}
}
