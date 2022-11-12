//! Wrapper for `Task`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L1727
use crate::{impl_julia_typecheck, private::Private, wrappers::ptr::private::WrapperPriv};

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
    pub fn next(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe {
            let next = self.unwrap_non_null(Private).as_ref().next;
            debug_assert!(!next.is_null());
            ValueRef::wrap(NonNull::new_unchecked(next))
        }
    }

    /// Invasive linked list for scheduler
    #[cfg(feature = "extra-fields")]
    pub fn queue(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let queue = self.unwrap_non_null(Private).as_ref().queue;
            let queue = NonNull::new(queue)?;
            Some(ValueRef::wrap(queue))
        }
    }

    /// The `tls` field, called `Task.storage` in Julia.
    #[cfg(feature = "extra-fields")]
    pub fn storage(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let storage = self.unwrap_non_null(Private).as_ref().tls;
            let storage = NonNull::new(storage)?;
            Some(ValueRef::wrap(storage))
        }
    }

    /// The `donenotify` field.
    #[cfg(feature = "extra-fields")]
    pub fn done_notify(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let donenotify = self.unwrap_non_null(Private).as_ref().donenotify;
            let donenotify = NonNull::new(donenotify)?;
            Some(ValueRef::wrap(donenotify))
        }
    }

    /// The `result` field.
    #[cfg(feature = "extra-fields")]
    pub fn result(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let result = self.unwrap_non_null(Private).as_ref().result;
            let result = NonNull::new(result)?;
            Some(ValueRef::wrap(result))
        }
    }

    /// The `logstate` field.
    #[cfg(feature = "extra-fields")]
    pub fn log_state(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let logstate = self.unwrap_non_null(Private).as_ref().logstate;
            let logstate = NonNull::new(logstate)?;
            Some(ValueRef::wrap(logstate))
        }
    }

    /// The `start` field.
    #[cfg(feature = "extra-fields")]
    pub fn start(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let start = self.unwrap_non_null(Private).as_ref().start;
            let start = NonNull::new(start)?;
            Some(ValueRef::wrap(start))
        }
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
}

impl_julia_typecheck!(Task<'scope>, jl_task_type, 'scope);
impl_debug!(Task<'_>);

impl<'scope> WrapperPriv<'scope, '_> for Task<'scope> {
    type Wraps = jl_task_t;
    type TypeConstructorPriv<'target, 'da> = Task<'target>;
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

/// A reference to a [`Task`] that has not been explicitly rooted.
pub type TaskRef<'scope> = Ref<'scope, 'static, Task<'scope>>;
impl_valid_layout!(TaskRef, Task);
impl_ref_root!(Task, TaskRef, 1);

use crate::memory::target::target_type::TargetType;

/// `Task` or `TaskRef`, depending on the target type `T`.
pub type TaskData<'target, T> = <T as TargetType<'target>>::Data<'static, Task<'target>>;

/// `JuliaResult<Task>` or `JuliaResultRef<TaskRef>`, depending on the target type `T`.
pub type TaskResult<'target, T> = <T as TargetType<'target>>::Result<'static, Task<'target>>;
