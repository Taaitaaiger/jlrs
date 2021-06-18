//! A `Future` that represents a function call in Julia running on another thread.

use crate::error::CANNOT_DISPLAY_VALUE;
use crate::memory::{global::Global, scope::Scope};
use crate::wrappers::ptr::call::UnsafeCall;
use crate::wrappers::ptr::module::Module;
use crate::wrappers::ptr::task::Task;
use crate::wrappers::ptr::value::{Value, MAX_SIZE};
use crate::wrappers::ptr::Wrapper;
use crate::{
    error::{exception, JlrsError, JlrsResult, JuliaResult},
    private::Private,
    wrappers::ptr::private::Wrapper as _,
};
use futures::task::{Context, Poll, Waker};
use futures::Future;
use jl_sys::{jl_call1, jl_exception_occurred};
use smallvec::SmallVec;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use super::async_frame::AsyncGcFrame;

pub(crate) struct TaskState<'frame, 'data> {
    completed: bool,
    waker: Option<Waker>,
    task: Option<Task<'frame>>,
    _marker: PhantomData<&'data ()>,
}

/// A `Future` that runs a Julia function on a new thread with `Base.Threads.@spawn`. The function
/// is called as soon as it is created, not when it's polled for the first time. You can create a
/// `JuliaFuture` by calling [`Value::call_async`]. Calling this function uses two slots in
/// the current frame.
pub struct JuliaFuture<'frame, 'data> {
    shared_state: Arc<Mutex<TaskState<'frame, 'data>>>,
}

impl<'frame, 'data> JuliaFuture<'frame, 'data> {
    pub(crate) fn new<'value, V>(
        frame: &mut AsyncGcFrame<'frame>,
        func: Value,
        mut values: V,
    ) -> JlrsResult<Self>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        let shared_state = Arc::new(Mutex::new(TaskState {
            completed: false,
            waker: None,
            task: None,
            _marker: PhantomData,
        }));

        unsafe {
            let values = values.as_mut();
            let state_ptr = Arc::into_raw(shared_state.clone()) as *mut c_void;
            let state_ptr_boxed = Value::new(&mut *frame, state_ptr)?;

            let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(2 + values.len());

            vals.push(func);
            vals.push(state_ptr_boxed);
            vals.extend_from_slice(values);

            let global = frame.global();
            let task = Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("asynccall")?
                .wrapper_unchecked()
                .unsafe_call(frame, &mut vals)?
                .map_err(|e| {
                    let msg = e.display_string_or(CANNOT_DISPLAY_VALUE);
                    JlrsError::Exception {
                        msg: format!("asynccall threw an exception: {}", msg),
                    }
                })?
                .cast_unchecked::<Task>();

            {
                let locked = shared_state.lock();
                match locked {
                    Ok(mut data) => data.task = Some(task),
                    _ => exception("Cannot set task".into())?,
                }
            }

            Ok(JuliaFuture { shared_state })
        }
    }
}

impl<'frame, 'data> Future for JuliaFuture<'frame, 'data> {
    type Output = JuliaResult<'frame, 'data>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            if let Some(task) = shared_state.task {
                // Ensure the result of the task is set
                unsafe {
                    let global = Global::new();
                    let f = Module::base(global)
                        .function_ref("fetch")
                        .unwrap()
                        .wrapper_unchecked();

                    let res = jl_call1(f.unwrap(Private), task.unwrap(Private).cast());
                    let exc = jl_exception_occurred();

                    if exc.is_null() {
                        Poll::Ready(Ok(Value::wrap(res, Private)))
                    } else {
                        Poll::Ready(Err(Value::wrap(exc, Private)))
                    }
                }
            } else {
                // JuliaFuture is not created if task cannot be set
                unreachable!()
            }
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

// This function is set as a constant in `Main.Jlrs` and called using `ccall` to indicate a task has
// completed.
pub(crate) unsafe extern "C" fn wake_task(state: *const Mutex<TaskState>) {
    let state = Arc::from_raw(state);
    let shared_state = state.lock();
    match shared_state {
        Ok(mut state) => {
            state.completed = true;
            state.waker.take().map(|waker| waker.wake());
        }
        Err(_) => (),
    }
}
