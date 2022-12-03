//! Managed for `WeakRef`.

use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_weakref_t, jl_weakref_type};

use crate::{
    data::managed::{
        private::ManagedPriv,
        value::{ValueData, ValueRef},
        Ref,
    },
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
};

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
    pub fn value<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        unsafe {
            let value = self.unwrap_non_null(Private).as_ref().value;
            let value = NonNull::new(value)?;
            Some(ValueRef::wrap(value).root(target))
        }
    }
}

impl_julia_typecheck!(WeakRef<'scope>, jl_weakref_type, 'scope);
impl_debug!(WeakRef<'_>);

impl<'scope> ManagedPriv<'scope, '_> for WeakRef<'scope> {
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

/// A reference to a [`WeakRef`] that has not been explicitly rooted.
pub type WeakRefRef<'scope> = Ref<'scope, 'static, WeakRef<'scope>>;
impl_valid_layout!(WeakRefRef, WeakRef);

use crate::memory::target::target_type::TargetType;

/// `WeakRef` or `WeakRefRef`, depending on the target type `T`.
pub type WeakRefData<'target, T> = <T as TargetType<'target>>::Data<'static, WeakRef<'target>>;

/// `JuliaResult<WeakRef>` or `JuliaResultRef<WeakRefRef>`, depending on the target type`T`.
pub type WeakRefResult<'target, T> = <T as TargetType<'target>>::Result<'static, WeakRef<'target>>;
