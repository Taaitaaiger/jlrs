//! Call a function on another thread and await the result.

use super::{async_frame::AsyncGcFrame, julia_future::JuliaFuture};
use crate::{
    error::{JlrsResult, JuliaResult},
    wrappers::ptr::{
        call::{Call, WithKeywords},
        function::Function,
        value::Value,
        Wrapper,
    },
};
use async_trait::async_trait;

/// This trait extends [`Call`] and adds another way to call a Julia function, `call_async`. It's
/// implemented by [`Value`], [`Function`] and [`WithKeywords`].
#[async_trait(?Send)]
pub trait CallAsync<'data>: Call<'data> {
    /// Call a function on another thread with the given arguments. This method uses
    /// `Threads.@spawn` to call the given function on another thread but return immediately.
    /// While `await`ing the result the async runtime can work on other tasks, the current task
    /// resumes after the function call on the other thread completes (either by returning or
    /// throwing).
    async fn call_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>;
}

#[async_trait(?Send)]
impl<'data> CallAsync<'data> for Value<'_, 'data> {
    async fn call_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new(frame, self, args)?.await)
    }
}

#[async_trait(?Send)]
impl<'data> CallAsync<'data> for Function<'_, 'data> {
    async fn call_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new(frame, self.as_value(), args)?.await)
    }
}

#[async_trait(?Send)]
impl<'data> CallAsync<'data> for WithKeywords<'_, 'data> {
    async fn call_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new_with_keywords(frame, self, args)?.await)
    }
}
