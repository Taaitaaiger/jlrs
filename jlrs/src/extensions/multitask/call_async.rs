//! Create, schedule and `await` Julia `Task`s.

use super::{async_frame::AsyncGcFrame, julia_future::JuliaFuture, yield_task};
use crate::{
    error::{JlrsResult, JuliaResult},
    memory::scope::Scope,
    wrappers::ptr::call::CallExt,
    wrappers::ptr::{
        call::{Call, WithKeywords},
        function::Function,
        module::Module,
        task::Task,
        value::{Value, MAX_SIZE},
        Wrapper,
    },
};
use async_trait::async_trait;
use smallvec::SmallVec;

/// This trait provides async methods to create and schedule `Task`s that resolve when the `Task`
/// has completed. Non-async methods are also provided which only schedule the `Task`, those
/// methods should only be used from [`PersistentTask::init`].
///
/// [`PersistentTask::init`]: crate::extensions::multitask::async_task::PersistentTask::init
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

    /// Does the same thing as [`CallAsync::call_async`], but the task is returned rather than an
    /// awaitable `Future`. This method should only be called in [`PersistentTask::init`],
    /// otherwise it's not guaranteed this task can progress.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global.
    ///
    /// [`PersistentTask::init`]: crate::extensions::multitask::async_task::PersistentTask::init
    unsafe fn schedule_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>;

    /// Call a function with the given arguments in an `@async` block. Like `call_async`, the
    /// function is not called on the main thread, but on a separate thread that handles all
    /// tasks created by this method. This method should only be used with functions that do very
    /// little computational work but mostly spend their time waiting on IO.
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

    /// Does the same thing as [`CallAsync::call_async_local`], but the task is returned rather
    /// than an awaitable `Future`. This method should only be called in [`PersistentTask::init`],
    /// otherwise it's not guaranteed this task can progress.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global.
    ///
    /// [`PersistentTask::init`]: crate::extensions::multitask::async_task::PersistentTask::init
    unsafe fn schedule_async_local<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>;

    /// Call a function with the given arguments in an `@async` block. The task is scheduled on
    /// the main. This method should only be used with functions that must run on the main thread
    /// but do very little computational work. The runtime is blocked while this task is active.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global.
    async unsafe fn call_async_main<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>;

    /// Does the same thing as [`CallAsync::call_async_main`], but the task is returned rather
    /// than an awaitable `Future`. This method should only be called in [`PersistentTask::init`],
    /// otherwise it's not guaranteed this task can progress.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global.
    ///
    /// [`PersistentTask::init`]: crate::extensions::multitask::async_task::PersistentTask::init
    unsafe fn schedule_async_main<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
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

    unsafe fn schedule_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        mut args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        let values = args.as_mut();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

        vals.push(self);
        vals.extend_from_slice(values);

        let global = frame.global();
        let task = Module::main(global)
            .submodule_ref("JlrsMultitask")?
            .wrapper_unchecked()
            .function_ref("asynccall")?
            .wrapper_unchecked()
            .call(&mut *frame, &mut vals)?;

        yield_task(frame);

        match task {
            Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
            Err(e) => Ok(Err(e)),
        }
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

    unsafe fn schedule_async_local<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        mut args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        let values = args.as_mut();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

        vals.push(self);
        vals.extend_from_slice(values);

        let global = frame.global();
        let task = Module::main(global)
            .submodule_ref("JlrsMultitask")?
            .wrapper_unchecked()
            .function_ref("scheduleasynclocal")?
            .wrapper_unchecked()
            .call(&mut *frame, &mut vals)?;

        yield_task(frame);

        match task {
            Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
            Err(e) => Ok(Err(e)),
        }
    }

    async unsafe fn call_async_main<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new_main(frame, self, args)?.await)
    }

    unsafe fn schedule_async_main<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        mut args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        let values = args.as_mut();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

        vals.push(self);
        vals.extend_from_slice(values);

        let global = frame.global();
        let task = Module::main(global)
            .submodule_ref("JlrsMultitask")?
            .wrapper_unchecked()
            .function_ref("scheduleasync")?
            .wrapper_unchecked()
            .call(&mut *frame, &mut vals)?;

        yield_task(frame);

        match task {
            Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
            Err(e) => Ok(Err(e)),
        }
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

    unsafe fn schedule_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        self.as_value().schedule_async(frame, args)
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

    unsafe fn schedule_async_local<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        self.as_value().schedule_async_local(frame, args)
    }

    async unsafe fn call_async_main<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new_main(frame, self.as_value(), args)?.await)
    }

    unsafe fn schedule_async_main<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        self.as_value().schedule_async_main(frame, args)
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

    unsafe fn schedule_async<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        mut args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        let values = args.as_mut();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

        vals.push(self.function());
        vals.extend_from_slice(values);

        let global = frame.global();
        let task = Module::main(global)
            .submodule_ref("JlrsMultitask")?
            .wrapper_unchecked()
            .function_ref("asynccall")?
            .wrapper_unchecked()
            .with_keywords(self.keywords())?
            .call(&mut *frame, &mut vals)?;

        yield_task(frame);

        match task {
            Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
            Err(e) => Ok(Err(e)),
        }
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

    unsafe fn schedule_async_local<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        mut args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        let values = args.as_mut();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

        vals.push(self.function());
        vals.extend_from_slice(values);

        let global = frame.global();
        let task = Module::main(global)
            .submodule_ref("JlrsMultitask")?
            .wrapper_unchecked()
            .function_ref("scheduleasynclocal")?
            .wrapper_unchecked()
            .with_keywords(self.keywords())?
            .call(&mut *frame, &mut vals)?;

        yield_task(frame);

        match task {
            Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
            Err(e) => Ok(Err(e)),
        }
    }

    async unsafe fn call_async_main<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: V,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        Ok(JuliaFuture::new_main_with_keywords(frame, self, args)?.await)
    }

    unsafe fn schedule_async_main<'frame, 'value, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        mut args: V,
    ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        let values = args.as_mut();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

        vals.push(self.function());
        vals.extend_from_slice(values);

        let global = frame.global();
        let task = Module::main(global)
            .submodule_ref("JlrsMultitask")?
            .wrapper_unchecked()
            .function_ref("scheduleasync")?
            .wrapper_unchecked()
            .with_keywords(self.keywords())?
            .call(&mut *frame, &mut vals)?;

        yield_task(frame);

        match task {
            Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
            Err(e) => Ok(Err(e)),
        }
    }
}
