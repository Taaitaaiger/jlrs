use std::{ffi::c_void, fmt::Display, marker::PhantomData, pin::Pin, ptr::NonNull, sync::Arc};

use futures::{
    task::{Context, Poll, Waker},
    Future,
};
use jl_sys::{jl_call, jl_call1, jl_exception_occurred};
use jlrs_macros::julia_version;

use crate::{
    args::Values,
    call::{Call, WithKeywords},
    data::managed::{
        erase_scope_lifetime,
        module::{JlrsCore, Module},
        private::ManagedPriv,
        task::Task,
        value::Value,
        Managed,
    },
    error::{JuliaResult, CANNOT_DISPLAY_VALUE},
    gc_safe::GcSafeMutex,
    memory::target::{frame::AsyncGcFrame, private::TargetPriv, unrooted::Unrooted},
    private::Private,
};

pub(crate) struct TaskState<'frame, 'data> {
    completed: bool,
    waker: Option<Waker>,
    task: Option<Task<'frame>>,
    _marker: PhantomData<&'data ()>,
}

enum AsyncMethod {
    AsyncCall,
    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
    InteractiveCall,
    ScheduleAsync,
    ScheduleAsyncLocal,
}

impl Display for AsyncMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsyncMethod::AsyncCall => f.write_str("asynccall"),
            #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
            AsyncMethod::InteractiveCall => f.write_str("interactivecall"),
            AsyncMethod::ScheduleAsync => f.write_str("scheduleasync"),
            AsyncMethod::ScheduleAsyncLocal => f.write_str("scheduleasynclocal"),
        }
    }
}

pub(crate) struct JuliaFuture<'frame, 'data> {
    shared_state: Arc<GcSafeMutex<TaskState<'frame, 'data>>>,
}

impl<'frame, 'data> JuliaFuture<'frame, 'data> {
    #[inline]
    pub(crate) fn new<'value, V, const N: usize>(
        frame: &mut AsyncGcFrame<'frame>,
        func: Value<'value, 'data>,
        values: V,
    ) -> Self
    where
        V: Values<'value, 'data, N>,
    {
        Self::new_future(frame, func, values, AsyncMethod::AsyncCall)
    }

    #[inline]
    #[julia_version(since = "1.9")]
    pub(crate) fn new_interactive<'value, V, const N: usize>(
        frame: &mut AsyncGcFrame<'frame>,
        func: Value<'value, 'data>,
        values: V,
    ) -> Self
    where
        V: Values<'value, 'data, N>,
    {
        Self::new_future(frame, func, values, AsyncMethod::InteractiveCall)
    }

    #[inline]
    pub(crate) fn new_local<'value, V, const N: usize>(
        frame: &mut AsyncGcFrame<'frame>,
        func: Value<'value, 'data>,
        values: V,
    ) -> Self
    where
        V: Values<'value, 'data, N>,
    {
        Self::new_future(frame, func, values, AsyncMethod::ScheduleAsyncLocal)
    }

    #[inline]
    pub(crate) fn new_main<'value, V, const N: usize>(
        frame: &mut AsyncGcFrame<'frame>,
        func: Value<'value, 'data>,
        values: V,
    ) -> Self
    where
        V: Values<'value, 'data, N>,
    {
        Self::new_future(frame, func, values, AsyncMethod::ScheduleAsync)
    }

    pub(crate) fn new_posted(
        frame: &mut AsyncGcFrame<'frame>,
        fn_ptr: Value<'_, '_>,
        task_ptr: Value<'_, '_>,
    ) -> Self {
        let shared_state = Arc::new(GcSafeMutex::new(TaskState {
            completed: false,
            waker: None,
            task: None,
            _marker: PhantomData,
        }));

        let state_ptr = Arc::into_raw(shared_state.clone()) as *mut c_void;
        let state_ptr_boxed = Value::new(&mut *frame, state_ptr);

        // Safety: module contents are globally rooted, and the function is guaranteed to be safe
        // by the caller.
        let task = unsafe {
            JlrsCore::post_blocking(&frame)
                .call3(&mut *frame, fn_ptr, task_ptr, state_ptr_boxed)
                .unwrap_or_else(|e| {
                    let msg = e.display_string_or(CANNOT_DISPLAY_VALUE);
                    panic!("postblocking threw an exception: {}", msg)
                })
                .cast_unchecked::<Task>()
        };

        {
            let mut locked = shared_state.lock();
            locked.task = Some(task);
        }

        JuliaFuture { shared_state }
    }

    #[inline]
    pub(crate) fn new_with_keywords<'kw, 'value, V, const N: usize>(
        frame: &mut AsyncGcFrame<'frame>,
        func: WithKeywords<'kw, 'data>,
        values: V,
    ) -> Self
    where
        V: Values<'value, 'data, N>,
    {
        Self::new_future_with_keywords(frame, func, values, AsyncMethod::AsyncCall)
    }

    #[inline]
    #[julia_version(since = "1.9")]
    pub(crate) fn new_interactive_with_keywords<'kw, 'value, V, const N: usize>(
        frame: &mut AsyncGcFrame<'frame>,
        func: WithKeywords<'kw, 'data>,
        values: V,
    ) -> Self
    where
        V: Values<'value, 'data, N>,
    {
        Self::new_future_with_keywords(frame, func, values, AsyncMethod::InteractiveCall)
    }

    #[inline]
    pub(crate) fn new_local_with_keywords<'kw, 'value, V, const N: usize>(
        frame: &mut AsyncGcFrame<'frame>,
        func: WithKeywords<'kw, 'data>,
        values: V,
    ) -> Self
    where
        V: Values<'value, 'data, N>,
    {
        Self::new_future_with_keywords(frame, func, values, AsyncMethod::ScheduleAsyncLocal)
    }

    #[inline]
    pub(crate) fn new_main_with_keywords<'kw, 'value, V, const N: usize>(
        frame: &mut AsyncGcFrame<'frame>,
        func: WithKeywords<'kw, 'data>,
        values: V,
    ) -> Self
    where
        V: Values<'value, 'data, N>,
    {
        Self::new_future_with_keywords(frame, func, values, AsyncMethod::ScheduleAsync)
    }

    fn new_future<'value, V, const N: usize>(
        frame: &mut AsyncGcFrame<'frame>,
        func: Value<'value, 'data>,
        values: V,
        method: AsyncMethod,
    ) -> Self
    where
        V: Values<'value, 'data, N>,
    {
        let shared_state = Arc::new(GcSafeMutex::new(TaskState {
            completed: false,
            waker: None,
            task: None,
            _marker: PhantomData,
        }));
        let state_ptr = Arc::into_raw(shared_state.clone()) as *mut c_void;
        let state_ptr_boxed = Value::new(&mut *frame, state_ptr);

        // Safety: module contents are globally rooted, and the function is guaranteed to be safe
        // by the caller.
        let task = unsafe {
            let values = values.into_extended_with_start(
                [
                    erase_scope_lifetime(func),
                    erase_scope_lifetime(state_ptr_boxed),
                ],
                Private,
            );

            let f = match method {
                AsyncMethod::AsyncCall => JlrsCore::async_call(&frame),
                #[cfg(not(any(
                    feature = "julia-1-6",
                    feature = "julia-1-7",
                    feature = "julia-1-8"
                )))]
                AsyncMethod::InteractiveCall => JlrsCore::interactive_call(&frame),
                AsyncMethod::ScheduleAsync => JlrsCore::schedule_async(&frame),
                AsyncMethod::ScheduleAsyncLocal => JlrsCore::schedule_async_local(&frame),
            };

            f.call(&mut *frame, values.as_ref())
                .unwrap_or_else(|e| {
                    let msg = e.display_string_or(CANNOT_DISPLAY_VALUE);
                    panic!("{} threw an exception: {}", method, msg)
                })
                .cast_unchecked::<Task>()
        };

        {
            let mut locked = shared_state.lock();
            locked.task = Some(task);
        }

        JuliaFuture { shared_state }
    }

    fn new_future_with_keywords<'kw, 'value, V, const N: usize>(
        frame: &mut AsyncGcFrame<'frame>,
        func: WithKeywords<'kw, 'data>,
        values: V,
        method: AsyncMethod,
    ) -> Self
    where
        V: Values<'value, 'data, N>,
    {
        let shared_state = Arc::new(GcSafeMutex::new(TaskState {
            completed: false,
            waker: None,
            task: None,
            _marker: PhantomData,
        }));

        let state_ptr = Arc::into_raw(shared_state.clone()) as *mut c_void;
        let state_ptr_boxed = Value::new(&mut *frame, state_ptr);

        // Safety: module contents are globally rooted, and the function is guaranteed to be safe
        // by the caller.
        let task = unsafe {
            let f = match method {
                AsyncMethod::AsyncCall => JlrsCore::async_call(&frame),
                #[cfg(not(any(
                    feature = "julia-1-6",
                    feature = "julia-1-7",
                    feature = "julia-1-8"
                )))]
                AsyncMethod::InteractiveCall => JlrsCore::interactive_call(&frame),
                AsyncMethod::ScheduleAsync => JlrsCore::schedule_async(&frame),
                AsyncMethod::ScheduleAsyncLocal => JlrsCore::schedule_async_local(&frame),
            };

            #[cfg(not(any(feature = "julia-1-10", feature = "julia-1-9")))]
            let kw_call = jl_sys::jl_get_kwsorter(f.datatype().unwrap(Private).cast());
            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            let kw_call = jl_sys::jl_kwcall_func;

            // WithKeywords::call has to extend the provided arguments, it has been inlined so
            // we only need to extend them once.
            let values = values.into_extended_pointers_with_start(
                [
                    func.keywords().unwrap(Private),
                    f.unwrap(Private),
                    func.function().unwrap(Private),
                    state_ptr_boxed.unwrap(Private),
                ],
                Private,
            );

            let values = values.as_ref();
            let res = jl_call(kw_call, values.as_ptr() as *mut _, values.len() as _);
            let exc = jl_exception_occurred();

            let res = if exc.is_null() {
                Ok(NonNull::new_unchecked(res))
            } else {
                Err(NonNull::new_unchecked(exc))
            };

            frame
                .result_from_ptr::<Value>(res, Private)
                .unwrap_or_else(|e| {
                    let msg = e.display_string_or(CANNOT_DISPLAY_VALUE);
                    panic!("{} threw an exception: {}", method, msg)
                })
                .cast_unchecked::<Task>()
        };

        {
            let mut locked = shared_state.lock();
            locked.task = Some(task);
        }

        JuliaFuture { shared_state }
    }
}

impl<'frame, 'data> Future for JuliaFuture<'frame, 'data> {
    type Output = JuliaResult<'frame, 'data>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock();
        if shared_state.completed {
            if let Some(task) = shared_state.task {
                // Safety: module contents are globally rooted, and fetch is safe to call. The
                // result is reachable through the task which must be rooted at ths point.
                unsafe {
                    let global = Unrooted::new();
                    let f = Module::base(&global)
                        .function(&global, "fetch")
                        .unwrap()
                        .as_managed();

                    let res = jl_call1(f.unwrap(Private), task.unwrap(Private).cast());
                    let exc = jl_exception_occurred();

                    if exc.is_null() {
                        Poll::Ready(Ok(Value::wrap_non_null(
                            NonNull::new_unchecked(res),
                            Private,
                        )))
                    } else {
                        Poll::Ready(Err(Value::wrap_non_null(
                            NonNull::new_unchecked(exc),
                            Private,
                        )))
                    }
                }
            } else {
                // JuliaFuture is not created if task cannot be set
                unreachable!()
            }
        } else if shared_state.waker.is_none() {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Pending
        }
    }
}

// This function is called using `ccall` to indicate a task has completed.
#[cfg(feature = "async-rt")]
pub(crate) unsafe extern "C" fn wake_task(state: *const GcSafeMutex<TaskState>) {
    let state = Arc::from_raw(state);
    let mut state = state.lock();
    state.completed = true;
    state.waker.take().map(|waker| waker.wake());
}
