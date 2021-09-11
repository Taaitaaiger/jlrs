//! Wrapper for `Vararg`.

use super::private::Wrapper;
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout};
use crate::{private::Private, wrappers::ptr::ValueRef};
use jl_sys::{jl_vararg_t, jl_vararg_type};
use std::{marker::PhantomData, ptr::NonNull};

/// A wrapper for `Vararg`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Vararg<'scope>(NonNull<jl_vararg_t>, PhantomData<&'scope ()>);

impl<'scope> Vararg<'scope> {
    /// The type of the arguments, i.e. the `T` in `Vararg{T, N}`.
    pub fn t(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().T) }
    }

    /// The number of arguments, i.e. the `N` in `Vararg{T, N}`.
    pub fn n(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().N) }
    }
}

impl_julia_typecheck!(Vararg<'scope>, jl_vararg_type, 'scope);
impl_debug!(Vararg<'_>);
impl_valid_layout!(Vararg<'scope>, 'scope);

impl<'scope> Wrapper<'scope, 'static> for Vararg<'scope> {
    type Wraps = jl_vararg_t;
    const NAME: &'static str = "Vararg";

    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}
