//! Julia-aware futures.

use std::{
    ffi::c_void,
    fmt::Display,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    ptr::NonNull,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use jl_sys::{jl_call, jl_call1, jl_exception_occurred, jlrs_current_task};

use crate::{
    args::Values,
    call::{Call, WithKeywords},
    data::managed::{
        Managed, erase_scope_lifetime,
        module::{JlrsCore, Module},
        private::ManagedPriv,
        value::Value,
    },
    error::{CANNOT_DISPLAY_VALUE, JuliaResult},
    gc_safe::GcSafeMutex,
    memory::{
        PTls,
        gc::{gc_safe_with, gc_unsafe_with},
        get_tls,
        target::{frame::AsyncGcFrame, private::TargetPriv, unrooted::Unrooted},
    },
    private::Private,
    util::kwcall_function,
};

/// A `Future` that enters a GC-safe state while it's progressing.
///
/// A typical use-case for GC-safe futures is async functions that don't interact with Julia; if
/// we're awaiting such a future, the GC can safely collect garbage while the future is pending.
/// Normally, such an operation would prevent garbage from being collected.
pub struct GcSafeFuture<T, F>
where
    F: Future<Output = T>,
{
    fut: F,
    ptls: PTls,
    _marker_t: PhantomData<T>,
}

impl<T, F> GcSafeFuture<T, F>
where
    F: Future<Output = T>,
{
    /// Create a new GC-safe future.
    ///
    /// A GC-safe future can only be created from a thread that can call into Julia.
    pub unsafe fn new(fut: F) -> Self {
        unsafe {
            assert!(!jlrs_current_task().is_null(), "No task");
            let ptls = get_tls();
            assert!(!ptls.is_null(), "no TLS");

            GcSafeFuture {
                fut,
                ptls,
                _marker_t: PhantomData,
            }
        }
    }
}

impl<T, F> Future for GcSafeFuture<T, F>
where
    F: Future<Output = T>,
{
    type Output = T;

    // Poll in GC-safe state to allow Julia to collect garbage on this thread.
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe {
            gc_safe_with(self.ptls, || {
                let pinned: Pin<&mut F> =
                    std::pin::Pin::new_unchecked(&mut self.get_unchecked_mut().fut);
                pinned.poll(cx)
            })
        }
    }
}

/// A `Future` that enters a GC-unsafe state while it's progressing.
///
/// A typical use-case for GC-unsafe futures is async functions that interact with Julia; if
/// we're awaiting such a future, we want to be able to mutate the GC state.
pub struct GcUnsafeFuture<T, F>
where
    F: Future<Output = T>,
{
    fut: F,
    ptls: PTls,
    _marker_t: PhantomData<T>,
}

impl<T, F> GcUnsafeFuture<T, F>
where
    F: Future<Output = T>,
{
    /// Create a new GC-unsafe future.
    ///
    /// A GC-safe future can only be created from a thread that can call into Julia.
    pub fn new(fut: F) -> Self {
        unsafe {
            debug_assert!(!jlrs_current_task().is_null(), "invalid_thread");
            let ptls = get_tls();
            GcUnsafeFuture {
                fut,
                ptls,
                _marker_t: PhantomData,
            }
        }
    }
}

impl<T, F> Future for GcUnsafeFuture<T, F>
where
    F: Future<Output = T>,
{
    type Output = T;

    // Poll in GC-unsafe state to allow calling into Julia
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe {
            gc_unsafe_with(self.ptls, |_| {
                let pinned: Pin<&mut F> =
                    std::pin::Pin::new_unchecked(&mut self.get_unchecked_mut().fut);
                pinned.poll(cx)
            })
        }
    }
}

pub(crate) struct TaskState<'frame, 'data> {
    completed: bool,
    waker: Option<Waker>,
    task: Option<Value<'frame, 'data>>,
}

enum AsyncMethod {
    AsyncCall,
    InteractiveCall,
}

impl Display for AsyncMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsyncMethod::AsyncCall => f.write_str("asynccall"),
            AsyncMethod::InteractiveCall => f.write_str("interactivecall"),
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
                AsyncMethod::InteractiveCall => JlrsCore::interactive_call(&frame),
            };

            f.call(&mut *frame, values.as_ref()).unwrap_or_else(|e| {
                let msg = e.display_string_or(CANNOT_DISPLAY_VALUE);
                panic!("{} threw an exception: {}", method, msg)
            })
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
        }));

        let state_ptr = Arc::into_raw(shared_state.clone()) as *mut c_void;
        let state_ptr_boxed = Value::new(&mut *frame, state_ptr);

        // Safety: module contents are globally rooted, and the function is guaranteed to be safe
        // by the caller.
        let task = unsafe {
            let f = match method {
                AsyncMethod::AsyncCall => JlrsCore::async_call(&frame),
                AsyncMethod::InteractiveCall => JlrsCore::interactive_call(&frame),
            };

            let kw_call = kwcall_function(&frame);

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
                        .global(&global, "fetch")
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
    unsafe {
        let state = Arc::from_raw(state);
        let mut state = state.lock();
        state.completed = true;
        state.waker.take().map(|waker| waker.wake());
    }
}
