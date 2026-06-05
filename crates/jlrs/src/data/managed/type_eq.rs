use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_typeeq_t, jl_typeeq_type};
use jlrs_sys::jlrs_typeeq_T;

use crate::{
    data::managed::{Weak, private::ManagedPriv, value::Value},
    impl_julia_typecheck,
    memory::target::{TargetResult, TargetType},
    private::Private,
};

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeEq<'scope>(NonNull<jl_typeeq_t>, PhantomData<&'scope ()>);

impl<'scope> TypeEq<'scope> {
    pub fn t(self) -> Value<'scope, 'static> {
        unsafe {
            let v = jlrs_typeeq_T(self.unwrap(Private));
            debug_assert!(!v.is_null());
            Value::wrap_non_null(NonNull::new_unchecked(v), Private)
        }
    }
}

impl<'scope> ManagedPriv<'scope, '_> for TypeEq<'scope> {
    type Wraps = jl_typeeq_t;
    type WithLifetimes<'target, 'da> = TypeEq<'target>;
    const NAME: &'static str = "TypeEq";

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

/// A [`TypeEq`] that has not been explicitly rooted.
pub type WeakTypeEq<'scope> = Weak<'scope, 'static, TypeEq<'scope>>;

/// A [`WeakTypeEq`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`TypeEq`].
pub type TypeEqRet = WeakTypeEq<'static>;

/// `TypeEq` or `WeakTypeEq`, depending on the target type `Tgt`.
pub type TypeEqData<'target, Tgt> = <Tgt as TargetType<'target>>::Data<'static, TypeEq<'target>>;

/// `JuliaResult<TypeEq>` or `WeakJuliaResult<WeakTypeEq>`, depending on the target type `Tgt`.
pub type TypeEqResult<'target, Tgt> = TargetResult<'target, 'static, TypeEq<'target>, Tgt>;

impl_julia_typecheck!(TypeEq<'scope>, jl_typeeq_type, 'scope);
impl_debug!(TypeEq<'_>);
impl_construct_type_managed!(TypeEq, 1, jl_typeeq_type);
impl_valid_layout!(WeakTypeEq, TypeEq, jl_typeeq_type);
impl_ccall_arg_managed!(TypeEq, 1);
impl_into_typed!(TypeEq);
