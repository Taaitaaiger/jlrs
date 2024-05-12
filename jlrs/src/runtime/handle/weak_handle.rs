use std::{
    marker::{PhantomData, PhantomPinned},
    pin::Pin,
};

use jl_sys::jlrs_task_gc_state;

use super::IsActive;
use crate::{error::RuntimeError, prelude::JlrsResult, runtime::state::GC_UNSAFE};

/// Create a new `WeakHandle` pinned to the current scope.
///
/// Returns an error if the current thread is unknown to Julia, or in an inactive state.
///
/// The macro should only be used in a `match` expression:
///
/// ```
/// # use jlrs::weak_handle;
///
/// match weak_handle!() {
///     Ok(_handle) => {
///         // use handle
///     }
///     Err(_e) => {
///         // can't call into Julia from this thread.
///     }
/// }
/// ```
#[macro_export]
macro_rules! weak_handle {
    () => {
        unsafe {
            $crate::error::project_jlrs_result(::std::pin::pin!(
                $crate::runtime::handle::weak_handle::WeakHandle::new()
            ))
        }
    };
}

/// Create a new `WeakHandle` pinned to the current scope without checking if this is allowed.
///
/// Safety: the current thread must be known to Julia, and must be in an inactive state (i.e. the
/// GC-state must be `GC_UNSAFE`).
#[macro_export]
macro_rules! weak_handle_unchecked {
    () => {
        ::std::pin::pin!($crate::runtime::handle::weak_handle::WeakHandle::new_unchecked())
    };
}

/// A weak handle to the current thread.
///
/// Weak handles are not tracked, i.e. they can't cause a shutdown of the global Julia runtime
/// when they're dropped. They also don't *prevent* a shutdown.
///
/// Weak handles must be pinned before they can be used, you should use the [`weak_handle`] and
/// [`weak_handle_unchecked`] macros to create new handles and immediately pin them. The first one
/// is the only way to safely create new weak handles.
pub struct WeakHandle {
    _marker: PhantomData<*mut ()>,
    _ph: PhantomPinned,
}

impl WeakHandle {
    /// Create a new `WeakHandle` to the current thread.
    ///
    /// Returns `RuntimeError::IncorrectState` if the current GC state doesn't allow creating a
    /// handle.
    ///
    /// Safety: The handle must not outlive the Julia runtime or escape from a call to
    /// `MtHandle::with`.
    #[inline]
    pub unsafe fn new() -> JlrsResult<Self> {
        if jlrs_task_gc_state() != GC_UNSAFE {
            Err(RuntimeError::IncorrectState)?;
        }

        Ok(Self::new_unchecked())
    }

    /// Create a new `WeakHandle` to the current thread without checking if this is allowed.
    ///
    /// Safety: The handle must not outlive the Julia runtime or escape from a call to
    /// `MtHandle::with`. The current thread must be known to Julia, and must be in an inactive
    ///  state (i.e. the GC-state must be `GC_UNSAFE`).
    #[inline(always)]
    pub const unsafe fn new_unchecked<'a>() -> Self {
        WeakHandle {
            _marker: PhantomData,
            _ph: PhantomPinned,
        }
    }
}

impl IsActive for Pin<&mut WeakHandle> {}
