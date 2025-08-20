//! Call Julia functions as a new task.

use std::future::Future;

use super::{Call, ProvideKeywords, WithKeywords};
use crate::{
    args::Values,
    async_util::future::JuliaFuture,
    data::managed::{erase_scope_lifetime, module::JlrsCore},
    error::JuliaResult,
    memory::target::frame::AsyncGcFrame,
    prelude::Value,
    private::Private,
};

/// This trait provides async methods to create and schedule `Task`s that resolve when the
/// `Task` has completed. Sync methods are also provided which only schedule the `Task`,
/// those methods should only be used from [`PersistentTask::init`].
///
/// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
pub trait CallAsync<'data>: Call<'data> {
    /// Creates and schedules a new task with `Base.Threads.@spawn`, and returns a future
    /// that resolves when this task is finished.
    ///
    /// This task is spawned on the `:default` thread pool.
    ///
    /// Safety: there is no way to distinguish between obviously safe functions like `+`, and
    /// obviously unsafe ones like `unsafe_load` except through their names. If multithreading is
    /// used, either via the multithreaded runtime or internally in Julia, potential thread-safety
    /// issues must also be taken into account.
    ///
    /// More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call_async<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> impl Future<Output = JuliaResult<'target, 'data>>
    where
        V: Values<'value, 'data, N>;

    /// Does the same thing as [`CallAsync::call_async`], but the task is returned rather than an
    /// awaitable `Future`. This method should only be called in [`PersistentTask::init`],
    /// otherwise it's not guaranteed this task can make progress.
    ///
    /// This task is spawned on the `:default` thread pool.
    ///
    /// Safety: there is no way to distinguish between obviously safe functions like `+`, and
    /// obviously unsafe ones like `unsafe_load` except through their names. If multithreading is
    /// used, either via the multithreaded runtime or internally in Julia, potential thread-safety
    /// issues must also be taken into account.
    ///
    /// More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
    unsafe fn schedule_async<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> JuliaResult<'target, 'data, Value<'target, 'data>>
    where
        V: Values<'value, 'data, N>;

    /// Call a function on another thread with the given arguments. This method uses
    /// `Base.Threads.@spawn` to call the given function on another thread but return immediately.
    /// While `await`ing the result the async runtime can work on other tasks, the current task
    /// resumes after the function call on the other thread completes.
    ///
    /// This task is spawned on the `:interactive` thread pool.
    ///
    /// Safety: there is no way to distinguish between obviously safe functions like `+`, and
    /// obviously unsafe ones like `unsafe_load` except through their names. If multithreading is
    /// used, either via the multithreaded runtime or internally in Julia, potential thread-safety
    /// issues must also be taken into account.
    ///
    /// More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call_async_interactive<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> impl Future<Output = JuliaResult<'target, 'data>>
    where
        V: Values<'value, 'data, N>;

    /// Does the same thing as [`CallAsync::call_async`], but the task is returned rather than an
    /// awaitable `Future`. This method should only be called in [`PersistentTask::init`],
    /// otherwise it's not guaranteed this task can make progress.
    ///
    /// This task is spawned on the `:interactive` thread pool.
    ///
    /// Safety: there is no way to distinguish between obviously safe functions like `+`, and
    /// obviously unsafe ones like `unsafe_load` except through their names. If multithreading is
    /// used, either via the multithreaded runtime or internally in Julia, potential thread-safety
    /// issues must also be taken into account.
    ///
    /// More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
    unsafe fn schedule_async_interactive<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> JuliaResult<'target, 'data, Value<'target, 'data>>
    where
        V: Values<'value, 'data, N>;
}

impl<'data> CallAsync<'data> for Value<'_, 'data> {
    #[inline]
    async unsafe fn call_async<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> JuliaResult<'target, 'data>
    where
        V: Values<'value, 'data, N>,
    {
        JuliaFuture::new(frame, erase_scope_lifetime(self), args).await
    }

    #[inline]
    async unsafe fn call_async_interactive<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> JuliaResult<'target, 'data>
    where
        V: Values<'value, 'data, N>,
    {
        JuliaFuture::new_interactive(frame, erase_scope_lifetime(self), args).await
    }

    #[inline]
    unsafe fn schedule_async_interactive<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> JuliaResult<'target, 'data, Value<'target, 'data>>
    where
        V: Values<'value, 'data, N>,
    {
        let args = args.into_extended_with_start([erase_scope_lifetime(self)], Private);

        JlrsCore::interactive_call(&frame).call(&mut *frame, args.as_ref())
    }

    #[inline]
    unsafe fn schedule_async<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> JuliaResult<'target, 'data, Value<'target, 'data>>
    where
        V: Values<'value, 'data, N>,
    {
        let args = args.into_extended_with_start([erase_scope_lifetime(self)], Private);

        let task = JlrsCore::async_call(&frame).call(&mut *frame, args.as_ref());

        match task {
            Ok(t) => Ok(t),
            Err(e) => Err(e),
        }
    }
}

impl<'data> CallAsync<'data> for WithKeywords<'_, 'data> {
    #[inline]
    async unsafe fn call_async<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> JuliaResult<'target, 'data>
    where
        V: Values<'value, 'data, N>,
    {
        JuliaFuture::new_with_keywords(frame, self, args).await
    }

    #[inline]
    async unsafe fn call_async_interactive<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> JuliaResult<'target, 'data>
    where
        V: Values<'value, 'data, N>,
    {
        JuliaFuture::new_interactive_with_keywords(frame, self, args).await
    }

    #[inline]
    unsafe fn schedule_async_interactive<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> JuliaResult<'target, 'data, Value<'target, 'data>>
    where
        V: Values<'value, 'data, N>,
    {
        let args = args.into_extended_with_start([erase_scope_lifetime(self.function())], Private);

        let task = JlrsCore::interactive_call(&frame)
            .provide_keywords(self.keywords())
            .call(&mut *frame, args.as_ref());

        match task {
            Ok(t) => Ok(t),
            Err(e) => Err(e),
        }
    }

    #[inline]
    unsafe fn schedule_async<'target, 'value, V, const N: usize>(
        self,
        frame: &mut AsyncGcFrame<'target>,
        args: V,
    ) -> JuliaResult<'target, 'data, Value<'target, 'data>>
    where
        V: Values<'value, 'data, N>,
    {
        let args = args.into_extended_with_start([erase_scope_lifetime(self.function())], Private);

        let task = JlrsCore::async_call(&frame)
            .provide_keywords(self.keywords())
            .call(&mut *frame, args.as_ref());

        match task {
            Ok(t) => Ok(t),
            Err(e) => Err(e),
        }
    }
}
