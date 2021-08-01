//! Call a Julia function asynchronously and await the result.

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

/// This trait provides methods that create and start new Julia tasks, and return a `Future` that
/// resolves when the task is completed. The task can either be scheduled on the main thread or on
/// another thread. Note that tasks running on the main thread will block the runtime unless it's
/// waiting for something like IO.
#[async_trait(?Send)]
pub trait CallAsync<'data>: Call<'data> {
    /// Call a function on another thread with the given arguments. This method uses
    /// `Base.Threads.@spawn` to call the given function on another thread but return immediately.
    /// While `await`ing the result the async runtime can work on other tasks, the current task
    /// resumes after the function call on the other thread completes.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global.
    async unsafe fn call_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>;

    /// Call a function with the given arguments in an `@async` block. Unlike `call_async`,
    /// the function is not executed on another thread, but on the main thread. This method
    /// should only be used with functions that do very little computational work but mostly
    /// spend their time waiting on IO.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global.
    async unsafe fn call_async_local<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>;
}

#[async_trait(?Send)]
impl<'data> CallAsync<'data> for Value<'_, 'data> {
    async unsafe fn call_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new(frame, self, args)?.await)
    }

    async unsafe fn call_async_local<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new_local(frame, self, args)?.await)
    }
}

#[async_trait(?Send)]
impl<'data> CallAsync<'data> for Function<'_, 'data> {
    async unsafe fn call_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new(frame, self.as_value(), args)?.await)
    }

    async unsafe fn call_async_local<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new_local(frame, self.as_value(), args)?.await)
    }
}

#[async_trait(?Send)]
impl<'data> CallAsync<'data> for WithKeywords<'_, 'data> {
    async unsafe fn call_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new_with_keywords(frame, self, args)?.await)
    }

    async unsafe fn call_async_local<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new_local_with_keywords(frame, self, args)?.await)
    }
}
