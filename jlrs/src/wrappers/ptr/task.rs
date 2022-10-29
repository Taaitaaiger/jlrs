//! Wrapper for `Task`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L1727
use crate::{
    impl_julia_typecheck, memory::target::Target, private::Private,
    wrappers::ptr::private::WrapperPriv,
};

#[cfg(feature = "extra-fields")]
use crate::wrappers::ptr::value::ValueRef;
use jl_sys::{jl_task_t, jl_task_type};
use std::{marker::PhantomData, ptr::NonNull};

#[cfg(feature = "extra-fields")]
use cfg_if::cfg_if;
#[cfg(feature = "extra-fields")]
#[cfg(not(feature = "lts"))]
use std::sync::atomic::Ordering;

use super::Ref;

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
    rngState0: UInt64
    rngState1: UInt64
    rngState2: UInt64
    rngState3: UInt64
    _state: UInt8 _Atomic
    sticky: Bool
    _isexception: Bool _Atomic
    */

    /// Invasive linked list for scheduler
    #[cfg(feature = "extra-fields")]
    pub fn next(self) -> TaskRef<'scope> {
        // Safety: the pointer points to valid data
        unsafe { TaskRef::wrap(self.unwrap_non_null(Private).as_ref().next.cast()) }
    }

    /// Invasive linked list for scheduler
    #[cfg(feature = "extra-fields")]
    pub fn queue(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().queue) }
    }

    /// The `tls` field, called `Task.storage` in Julia.
    #[cfg(feature = "extra-fields")]
    pub fn storage(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().tls) }
    }

    /// The `donenotify` field.
    #[cfg(feature = "extra-fields")]
    pub fn done_notify(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().donenotify) }
    }

    /// The `result` field.
    #[cfg(feature = "extra-fields")]
    pub fn result(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().result) }
    }

    /// The `logstate` field.
    #[cfg(feature = "extra-fields")]
    pub fn log_state(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().logstate) }
    }

    /// The `start` field.
    #[cfg(feature = "extra-fields")]
    pub fn start(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().start) }
    }

    /// The `_state` field.
    #[cfg(feature = "extra-fields")]
    pub fn state(self) -> u8 {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref()._state }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    self.unwrap_non_null(Private).as_ref()._state.load(Ordering::SeqCst)
                }
            }
        }
    }

    /// Record whether this Task can be migrated to a new thread
    #[cfg(feature = "extra-fields")]
    pub fn sticky(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().sticky != 0 }
    }

    /// set if `result` is an exception to throw or that we exited with
    #[cfg(feature = "extra-fields")]
    pub fn is_exception(self) -> bool {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref()._isexception != 0 }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    self.unwrap_non_null(Private).as_ref()._isexception.load(Ordering::SeqCst) != 0
                }
            }
        }
    }

    /// Use the target to reroot this data.
    pub fn root<'target, T>(self, target: T) -> T::Data
    where
        T: Target<'target, 'static, Task<'target>>,
    {
        // Safety: the data is valid.
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
    }
}

impl_julia_typecheck!(Task<'scope>, jl_task_type, 'scope);
impl_debug!(Task<'_>);

impl<'scope> WrapperPriv<'scope, '_> for Task<'scope> {
    type Wraps = jl_task_t;
    type StaticPriv = Task<'static>;
    const NAME: &'static str = "Task";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(Task, 1);

/// A reference to a [`Task`] that has not been explicitly rooted.
pub type TaskRef<'scope> = Ref<'scope, 'static, Task<'scope>>;
impl_valid_layout!(TaskRef, Task);
impl_ref_root!(Task, TaskRef, 1);
