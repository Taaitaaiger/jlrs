//! Wrapper for `Task`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L1727
use super::private::Wrapper;
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout};
use crate::{
    private::Private,
    wrappers::ptr::{TaskRef, ValueRef},
};
use jl_sys::{jl_task_t, jl_task_type};
use std::{marker::PhantomData, ptr::NonNull};

/// A Julia `Task` (coroutine).
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Task<'scope>(NonNull<jl_task_t>, PhantomData<&'scope ()>);

impl<'scope> Task<'scope> {
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
    pub fn next(self) -> TaskRef<'scope> {
        unsafe { TaskRef::wrap(self.unwrap_non_null(Private).as_ref().next.cast()) }
    }

    /// Invasive linked list for scheduler
    pub fn queue(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().queue) }
    }

    /// The `tls` field, called `Task.storage` in Julia.
    pub fn storage(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().tls) }
    }

    /// The `donenotify` field.
    pub fn done_notify(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().donenotify) }
    }

    /// The `result` field.
    pub fn result(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().result) }
    }

    /// The `logstate` field.
    pub fn log_state(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().logstate) }
    }

    /// The `start` field.
    pub fn start(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().start) }
    }

    /// The `state` field.
    pub fn state(self) -> u8 {
        unsafe { self.unwrap_non_null(Private).as_ref()._state }
    }

    /// Record whether this Task can be migrated to a new thread
    pub fn sticky(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().sticky != 0 }
    }

    /// set if `result` is an exception to throw or that we exited with
    pub fn is_exception(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref()._isexception != 0 }
    }
}

impl_julia_typecheck!(Task<'scope>, jl_task_type, 'scope);
impl_debug!(Task<'_>);
impl_valid_layout!(Task<'scope>, 'scope);

impl<'scope> Wrapper<'scope, '_> for Task<'scope> {
    type Wraps = jl_task_t;
    const NAME: &'static str = "Task";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}
