//! Support for values with the `Core.Task` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L1727
use super::{
    wrapper_ref::{TaskRef, ValueRef},
    Value,
};
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_task_t, jl_task_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// A Julia `Task` (coroutine).
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Task<'frame>(NonNull<jl_task_t>, PhantomData<&'frame ()>);

impl<'frame> Task<'frame> {
    pub(crate) unsafe fn wrap(task: *mut jl_task_t) -> Self {
        debug_assert!(!task.is_null());
        Task(NonNull::new_unchecked(task), PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_task_t> {
        self.0
    }

    /*
    for (a, b) in zip(fieldnames(Task), fieldtypes(Task))
        println(a, ": ", b)
    end
    next: Union{Task, Nothing}
    queue: Any
    storage: Any
    donenotify: Any
    result: Any
    logstate: Any
    code: Any
    _state: UInt8
    sticky: Bool
    _isexception: Bool
    */

    /// Invasive linked list for scheduler
    pub fn next(self) -> TaskRef<'frame> {
        unsafe { TaskRef::wrap((&*self.inner().as_ptr()).next.cast()) }
    }

    /// Invasive linked list for scheduler
    pub fn queue(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).queue) }
    }

    /// The `tls` field, called `Task.storage` in Julia.
    pub fn storage(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).tls) }
    }

    /// The `donenotify` field.
    pub fn donenotify(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).donenotify) }
    }

    /// The `result` field.
    pub fn result(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).result) }
    }

    pub fn logstate(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).logstate) }
    }

    pub fn start(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).start) }
    }

    pub fn state(self) -> u8 {
        unsafe { (&*self.inner().as_ptr())._state }
    }

    /// Record whether this Task can be migrated to a new thread
    pub fn sticky(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).sticky != 0 }
    }

    pub fn _isexception(self) -> bool {
        unsafe { (&*self.inner().as_ptr())._isexception != 0 }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for Task<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Task").finish()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for Task<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap_non_null(self.inner().cast()) }
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
        Self::wrap(value.inner().as_ptr().cast())
    }
}

impl_julia_typecheck!(Task<'frame>, jl_task_type, 'frame);

impl_valid_layout!(Task<'frame>, 'frame);
