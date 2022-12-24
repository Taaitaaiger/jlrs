//! Managed type for `Binding`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/7b10d5fe0159e21e8299681c33605f0b10dbdcfa/src/julia.h#L562

use std::{marker::PhantomData, ptr::NonNull, sync::atomic::Ordering};

use jl_sys::{jl_binding_t, jl_binding_type};

use crate::{
    data::managed::{module::ModuleData, private::ManagedPriv, value::ValueData, Ref},
    impl_julia_typecheck,
    memory::target::Target,
    prelude::Symbol,
    private::Private,
};

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Binding<'scope>(NonNull<jl_binding_t>, PhantomData<&'scope ()>);

impl<'scope> Binding<'scope> {
    /*
    inspect(Core.Binding):

    name: Symbol (const)
    value: Any (mut)
    globalref: GlobalRef (mut)
    owner: Module (mut)
    ty: Any (mut)
    flags: UInt8 (mut)
    */

    pub fn name<'target, T>(self, _: &T) -> Symbol<'target>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let name = self.unwrap_non_null(Private).as_ref().name;
            debug_assert!(!name.is_null());
            Symbol::wrap_non_null(NonNull::new_unchecked(name), Private)
        }
    }

    pub fn value<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let value = self
                .unwrap_non_null(Private)
                .as_ref()
                .value
                .load(Ordering::Relaxed);
            let ptr = NonNull::new(value)?;
            Some(target.data_from_ptr(ptr, Private))
        }
    }

    /// cached GlobalRef for this binding
    pub fn globalref<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let globalref = self
                .unwrap_non_null(Private)
                .as_ref()
                .globalref
                .load(Ordering::Relaxed);
            let ptr = NonNull::new(globalref)?;
            Some(target.data_from_ptr(ptr, Private))
        }
    }

    /// for individual imported bindings -- TODO: make _Atomic
    pub fn owner<'target, T>(self, target: T) -> Option<ModuleData<'target, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let owner = self.unwrap_non_null(Private).as_ref().owner;
            let ptr = NonNull::new(owner)?;
            Some(target.data_from_ptr(ptr, Private))
        }
    }

    /// binding type
    pub fn ty<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let ty = self
                .unwrap_non_null(Private)
                .as_ref()
                .ty
                .load(Ordering::Relaxed);
            let ptr = NonNull::new(ty)?;
            Some(target.data_from_ptr(ptr, Private))
        }
    }

    pub fn constp(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().constp() != 0 }
    }

    pub fn exportp(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().exportp() != 0 }
    }

    pub fn imported(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().imported() != 0 }
    }

    /// 0=not deprecated, 1=renamed, 2=moved to another package
    pub fn deprecated(self) -> u8 {
        unsafe { self.unwrap_non_null(Private).as_ref().deprecated() }
    }
}

impl_julia_typecheck!(Binding<'scope>, jl_binding_type, 'scope);
impl_debug!(Binding<'_>);

impl<'scope> ManagedPriv<'scope, '_> for Binding<'scope> {
    type Wraps = jl_binding_t;
    type TypeConstructorPriv<'target, 'da> = Binding<'target>;
    const NAME: &'static str = "Binding";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, ::std::marker::PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// A reference to a [`Binding`] that has not been explicitly rooted.
pub type BindingRef<'scope> = Ref<'scope, 'static, Binding<'scope>>;
impl_valid_layout!(BindingRef, Binding);

use crate::memory::target::target_type::TargetType;

/// `Binding` or `BindingRef`, depending on the target type `T`.
pub type BindingData<'target, T> = <T as TargetType<'target>>::Data<'static, Binding<'target>>;

/// `JuliaResult<Binding>` or `JuliaResultRef<BindingRef>`, depending on the target type
/// `T`.
pub type BindingResult<'target, T> = <T as TargetType<'target>>::Result<'static, Binding<'target>>;
