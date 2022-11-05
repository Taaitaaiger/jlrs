//! Wrapper for `WeakRef`.

use crate::{
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{private::WrapperPriv, value::ValueRef, Ref},
};
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
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().value) }
    }

    /// Use the target to reroot this data.
    pub fn root<'target, T>(self, target: T) -> WeakRefData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the data is valid.
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
    }
}

impl_julia_typecheck!(WeakRef<'scope>, jl_weakref_type, 'scope);
impl_debug!(WeakRef<'_>);

impl<'scope> WrapperPriv<'scope, '_> for WeakRef<'scope> {
    type Wraps = jl_weakref_t;
    type TypeConstructorPriv<'target, 'da> = WeakRef<'target>;
    const NAME: &'static str = "WeakRef";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(WeakRef, 1);

/// A reference to a [`WeakRef`] that has not been explicitly rooted.
pub type WeakRefRef<'scope> = Ref<'scope, 'static, WeakRef<'scope>>;
impl_valid_layout!(WeakRefRef, WeakRef);
impl_ref_root!(WeakRef, WeakRefRef, 1);

use crate::memory::target::target_type::TargetType;

/// `WeakRef` or `WeakRefRef`, depending on the target type `T`.
pub type WeakRefData<'target, T> = <T as TargetType<'target>>::Data<'static, WeakRef<'target>>;

/// `JuliaResult<WeakRef>` or `JuliaResultRef<WeakRefRef>`, depending on the target type`T`.
pub type WeakRefResult<'target, T> = <T as TargetType<'target>>::Result<'static, WeakRef<'target>>;
