//! Wrapper for `Core.WeakRef`.

use super::private::Wrapper;
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout};
use crate::{private::Private, wrappers::ptr::ValueRef};
use jl_sys::{jl_weakref_t, jl_weakref_type};
use std::{marker::PhantomData, ptr::NonNull};

/// A weak reference.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct WeakRef<'scope>(NonNull<jl_weakref_t>, PhantomData<&'scope ()>);

impl<'scope> WeakRef<'scope> {
    /*
    for (a, b) in zip(fieldnames(WeakRef), fieldtypes(WeakRef))
        println(a, ": ", b)
    end
    value: Any
    */

    /// The referenced `Value`.
    pub fn value(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().value) }
    }
}

impl_julia_typecheck!(WeakRef<'scope>, jl_weakref_type, 'scope);
impl_debug!(WeakRef<'_>);
impl_valid_layout!(WeakRef<'scope>, 'scope);

impl<'scope> Wrapper<'scope, '_> for WeakRef<'scope> {
    type Internal = jl_weakref_t;
    const NAME: &'static str = "WeakRef";

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
