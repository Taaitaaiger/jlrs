use super::{async_frame::AsyncGcFrame, julia_future::JuliaFuture};
use crate::{
    error::{JlrsResult, JuliaResult},
    wrappers::ptr::{function::Function, value::Value, Wrapper},
};
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait CallAsync<'data> {
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
        unsafe { Ok(JuliaFuture::new(frame, self, args)?.await) }
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
        unsafe { Ok(JuliaFuture::new(frame, self.as_value(), args)?.await) }
    }
}
