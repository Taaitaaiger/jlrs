//! Wrapper for `Core.WeakRef`.

use super::private::Wrapper;
use crate::{impl_julia_typecheck, impl_valid_layout};
use crate::{private::Private, wrappers::ptr::ValueRef};
use jl_sys::{jl_weakref_t, jl_weakref_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// A weak reference.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct WeakRef<'frame>(NonNull<jl_weakref_t>, PhantomData<&'frame ()>);

impl<'frame> WeakRef<'frame> {
    /*
    for (a, b) in zip(fieldnames(WeakRef), fieldtypes(WeakRef))
        println(a, ": ", b)
    end
    value: Any
    */

    /// The referenced `Value`.
    pub fn value(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().value) }
    }
}

impl<'scope> Debug for WeakRef<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("WeakRef").finish()
    }
}

impl_julia_typecheck!(WeakRef<'frame>, jl_weakref_type, 'frame);

impl_valid_layout!(WeakRef<'frame>, 'frame);

impl<'scope> Wrapper<'scope, '_> for WeakRef<'scope> {
    type Internal = jl_weakref_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
