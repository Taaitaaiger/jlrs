//! Managed type for `Task`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L1727
#[cfg(feature = "extra-fields")]
#[julia_version(since = "1.7")]
use std::sync::atomic::Ordering;
use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_task_t, jl_task_type};
#[cfg(feature = "extra-fields")]
use jlrs_macros::julia_version;

use super::Ref;
use crate::{
    data::managed::private::ManagedPriv, impl_julia_typecheck, memory::target::TargetResult,
    private::Private,
};
#[cfg(feature = "extra-fields")]
use crate::{
    data::managed::value::{ValueData, ValueRef},
    memory::target::Target,
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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

    #[cfg(feature = "extra-fields")]
    #[julia_version(until = "1.6")]
    /// The `_state` field.
    #[inline]
    pub fn state(self) -> u8 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref()._state }
    }

    #[cfg(feature = "extra-fields")]
    #[julia_version(since = "1.7")]
    /// The `_state` field.
    #[inline]
    pub fn state(self) -> u8 {
        // Safety: the pointer points to valid data
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                ._state
                .load(Ordering::Relaxed)
        }
    }

    /// Record whether this Task can be migrated to a new thread
    #[cfg(feature = "extra-fields")]
    #[inline]
    pub fn sticky(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().sticky != 0 }
    }

    #[cfg(feature = "extra-fields")]
    #[julia_version(until = "1.6")]
    /// set if `result` is an exception to throw or that we exited with
    #[inline]
    pub fn is_exception(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref()._isexception != 0 }
    }

    #[cfg(feature = "extra-fields")]
    #[julia_version(since = "1.7")]
    /// set if `result` is an exception to throw or that we exited with
    #[inline]
    pub fn is_exception(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                ._isexception
                .load(Ordering::Relaxed)
                != 0
        }
    }
}

impl_julia_typecheck!(Task<'scope>, jl_task_type, 'scope);
impl_debug!(Task<'_>);

impl<'scope> ManagedPriv<'scope, '_> for Task<'scope> {
    type Wraps = jl_task_t;
    type TypeConstructorPriv<'target, 'da> = Task<'target>;
    const NAME: &'static str = "Task";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_construct_type_managed!(Task, 1, jl_task_type);

/// A reference to a [`Task`] that has not been explicitly rooted.
pub type TaskRef<'scope> = Ref<'scope, 'static, Task<'scope>>;

/// A [`TaskRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`Task`].
pub type TaskRet = Ref<'static, 'static, Task<'static>>;

impl_valid_layout!(TaskRef, Task, jl_task_type);

use crate::memory::target::TargetType;

/// `Task` or `TaskRef`, depending on the target type `T`.
pub type TaskData<'target, T> = <T as TargetType<'target>>::Data<'static, Task<'target>>;

/// `JuliaResult<Task>` or `JuliaResultRef<TaskRef>`, depending on the target type `T`.
pub type TaskResult<'target, T> = TargetResult<'target, 'static, Task<'target>, T>;

impl_ccall_arg_managed!(Task, 1);
impl_into_typed!(Task);
