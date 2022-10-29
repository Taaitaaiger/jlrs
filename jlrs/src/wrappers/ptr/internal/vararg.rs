//! Wrapper for `Vararg`.

use crate::{
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{private::WrapperPriv, value::ValueRef, Ref},
};
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

    /// Use the target to reroot this data.
    pub fn root<'target, T>(self, target: T) -> VarargData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the data is valid.
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
    }
}

impl_julia_typecheck!(Vararg<'scope>, jl_vararg_type, 'scope);
impl_debug!(Vararg<'_>);

impl<'scope> WrapperPriv<'scope, 'static> for Vararg<'scope> {
    type Wraps = jl_vararg_t;
    type StaticPriv = Vararg<'static>;
    const NAME: &'static str = "Vararg";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(Vararg, 1);

/// A reference to a [`Vararg`] that has not been explicitly rooted.
pub type VarargRef<'scope> = Ref<'scope, 'static, Vararg<'scope>>;
impl_valid_layout!(VarargRef, Vararg);
impl_ref_root!(Vararg, VarargRef, 1);

use crate::memory::target::target_type::TargetType;
pub type VarargData<'target, T> = <T as TargetType<'target>>::Data<'static, Vararg<'target>>;
pub type VarargResult<'target, T> = <T as TargetType<'target>>::Result<'static, Vararg<'target>>;
