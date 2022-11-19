//! Wrapper for `Task`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L1727
#[cfg(feature = "extra-fields")]
#[cfg(not(feature = "lts"))]
use std::sync::atomic::Ordering;
use std::{marker::PhantomData, ptr::NonNull};

#[cfg(feature = "extra-fields")]
use cfg_if::cfg_if;
use jl_sys::{jl_task_t, jl_task_type};

use super::Ref;
use crate::{impl_julia_typecheck, private::Private, wrappers::ptr::private::WrapperPriv};
#[cfg(feature = "extra-fields")]
use crate::{
    memory::target::Target,
    wrappers::ptr::value::{ValueData, ValueRef},
};

/// A Julia `Task` (coroutine).
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Task<'scope>(NonNull<jl_task_t>, PhantomData<&'scope ()>);

impl<'scope> Task<'scope> {
    /*
    inspect(Task):

    next: Union{Task, Nothing} (mut)
    queue: Any (mut)
    storage: Any (mut)
    donenotify: Any (mut)
    result: Any (mut)
    logstate: Any (mut)
    code: Any (mut)
    rngState0: UInt64 (mut)
    rngState1: UInt64 (mut)
    rngState2: UInt64 (mut)
    rngState3: UInt64 (mut)
    _state: UInt8 (mut) _Atomic
    sticky: Bool (mut)
    _isexception: Bool (mut) _Atomic
    priority: UInt16 (mut)
    */

    /// Invasive linked list for scheduler
    #[cfg(feature = "extra-fields")]
    pub fn next<'target, T>(self, target: T) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let next = self.unwrap_non_null(Private).as_ref().next;
            debug_assert!(!next.is_null());
            ValueRef::wrap(NonNull::new_unchecked(next)).root(target)
        }
    }

    /// Invasive linked list for scheduler
    #[cfg(feature = "extra-fields")]
    pub fn queue<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let queue = self.unwrap_non_null(Private).as_ref().queue;
            let queue = NonNull::new(queue)?;
            Some(ValueRef::wrap(queue).root(target))
        }
    }

    /// The `tls` field, called `Task.storage` in Julia.
    #[cfg(feature = "extra-fields")]
    pub fn storage<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let storage = self.unwrap_non_null(Private).as_ref().tls;
            let storage = NonNull::new(storage)?;
            Some(ValueRef::wrap(storage).root(target))
        }
    }

    /// The `donenotify` field.
    #[cfg(feature = "extra-fields")]
    pub fn done_notify<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let donenotify = self.unwrap_non_null(Private).as_ref().donenotify;
            let donenotify = NonNull::new(donenotify)?;
            Some(ValueRef::wrap(donenotify).root(target))
        }
    }

    /// The `result` field.
    #[cfg(feature = "extra-fields")]
    pub fn result<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let result = self.unwrap_non_null(Private).as_ref().result;
            let result = NonNull::new(result)?;
            Some(ValueRef::wrap(result).root(target))
        }
    }

    /// The `logstate` field.
    #[cfg(feature = "extra-fields")]
    pub fn log_state<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let logstate = self.unwrap_non_null(Private).as_ref().logstate;
            let logstate = NonNull::new(logstate)?;
            Some(ValueRef::wrap(logstate).root(target))
        }
    }

    /// The `start` field.
    #[cfg(feature = "extra-fields")]
    pub fn start<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let start = self.unwrap_non_null(Private).as_ref().start;
            let start = NonNull::new(start)?;
            Some(ValueRef::wrap(start).root(target))
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

use crate::memory::target::target_type::TargetType;

/// `Task` or `TaskRef`, depending on the target type `T`.
pub type TaskData<'target, T> = <T as TargetType<'target>>::Data<'static, Task<'target>>;

/// `JuliaResult<Task>` or `JuliaResultRef<TaskRef>`, depending on the target type `T`.
pub type TaskResult<'target, T> = <T as TargetType<'target>>::Result<'static, Task<'target>>;
