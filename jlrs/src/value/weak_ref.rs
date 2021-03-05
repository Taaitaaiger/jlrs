//! Support for values with the `Core.WeakRef` type.

use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_weakref_t, jl_weakref_type};
use std::{fmt::{Debug, Formatter, Result as FmtResult}, marker::PhantomData};

/// A weak reference.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct WeakRef<'frame>(*mut jl_weakref_t, PhantomData<&'frame ()>);

impl<'frame> WeakRef<'frame> {
    pub(crate) unsafe fn wrap(weak_ref: *mut jl_weakref_t) -> Self {
        WeakRef(weak_ref, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_weakref_t {
        self.0
    }

    /// The referenced `Value`.
    pub fn value(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).value) }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for WeakRef<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("WeakRef").finish()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for WeakRef<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for WeakRef<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAWeakRef)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(WeakRef<'frame>, jl_weakref_type, 'frame);
impl_julia_type!(WeakRef<'frame>, jl_weakref_type, 'frame);
impl_valid_layout!(WeakRef<'frame>, 'frame);
