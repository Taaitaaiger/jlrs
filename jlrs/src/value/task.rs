//! Support for values with the `Core.Task` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L1727
use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_task_t, jl_task_type};
use std::marker::PhantomData;

/// A Julia `Task` (coroutine).
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Task<'frame>(*mut jl_task_t, PhantomData<&'frame ()>);

impl<'frame> Task<'frame> {
    pub(crate) unsafe fn wrap(task: *mut jl_task_t) -> Self {
        Task(task, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_task_t {
        self.0
    }

    /// Invasive linked list for scheduler
    pub fn next(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let next = (&*self.ptr()).next;
            if next.is_null() {
                None
            } else {
                Some(Value::wrap(next))
            }
        }
    }

    /// Invasive linked list for scheduler
    pub fn queue(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let queue = (&*self.ptr()).queue;
            if queue.is_null() {
                None
            } else {
                Some(Value::wrap(queue))
            }
        }
    }

    /// The `tls` field.
    pub fn tls(self) -> Value<'frame, 'static> {
        unsafe {
            let tls = (&*self.ptr()).tls;
            Value::wrap(tls)
        }
    }

    /// The `donenotify` field.
    pub fn donenotify(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let donenotify = (&*self.ptr()).donenotify;
            if donenotify.is_null() {
                None
            } else {
                Some(Value::wrap(donenotify))
            }
        }
    }

    /// The `result` field.
    pub fn result(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let result = (&*self.ptr()).result;
            if result.is_null() {
                None
            } else {
                Some(Value::wrap(result))
            }
        }
    }

    pub fn logstate(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let logstate = (&*self.ptr()).logstate;
            if logstate.is_null() {
                None
            } else {
                Some(Value::wrap(logstate))
            }
        }
    }

    pub fn start(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let start = (&*self.ptr()).start;
            if start.is_null() {
                None
            } else {
                Some(Value::wrap(start))
            }
        }
    }

    /// Record whether this Task can be migrated to a new thread
    pub fn sticky(self) -> u8 {
        unsafe { (&*self.ptr()).sticky }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for Task<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Task<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotATask)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(Task<'frame>, jl_task_type, 'frame);
impl_julia_type!(Task<'frame>, jl_task_type, 'frame);
impl_valid_layout!(Task<'frame>, 'frame);
