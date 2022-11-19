//! Wrapper for `Vararg`.

use crate::{
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{
        private::WrapperPriv,
        value::{ValueData, ValueRef},
        Ref,
    },
};
use jl_sys::{jl_vararg_t, jl_vararg_type};
use std::{marker::PhantomData, ptr::NonNull};

/// A wrapper for `Vararg`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Vararg<'scope>(NonNull<jl_vararg_t>, PhantomData<&'scope ()>);

impl<'scope> Vararg<'scope> {
    /// The type of the arguments, i.e. the `T` in `Vararg{T, N}`.
    pub fn t<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        unsafe {
            let t = self.unwrap_non_null(Private).as_ref().T;
            let t = NonNull::new(t)?;
            Some(ValueRef::wrap(t).root(target))
        }
    }

    /// The number of arguments, i.e. the `N` in `Vararg{T, N}`.
    pub fn n<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        unsafe {
            let n = self.unwrap_non_null(Private).as_ref().N;
            let n = NonNull::new(n)?;
            Some(ValueRef::wrap(n).root(target))
        }
    }
}

impl_julia_typecheck!(Vararg<'scope>, jl_vararg_type, 'scope);
impl_debug!(Vararg<'_>);

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Vararg<'scope> {
    type Wraps = jl_vararg_t;
    type TypeConstructorPriv<'target, 'da> = Vararg<'target>;
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

/// A reference to a [`Vararg`] that has not been explicitly rooted.
pub type VarargRef<'scope> = Ref<'scope, 'static, Vararg<'scope>>;
impl_valid_layout!(VarargRef, Vararg);
impl_ref_root!(Vararg, VarargRef, 1);

use crate::memory::target::target_type::TargetType;

/// `Vararg` or `VarargRef`, depending on the target type `T`.
pub type VarargData<'target, T> = <T as TargetType<'target>>::Data<'static, Vararg<'target>>;

/// `JuliaResult<Vararg>` or `JuliaResultRef<VarargRef>`, depending on the target type`T`.
pub type VarargResult<'target, T> = <T as TargetType<'target>>::Result<'static, Vararg<'target>>;
