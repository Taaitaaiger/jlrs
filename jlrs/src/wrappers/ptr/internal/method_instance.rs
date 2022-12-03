//! Wrapper for `MethodInstance`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L321
use std::{marker::PhantomData, ptr::NonNull};

use cfg_if::cfg_if;
use jl_sys::{jl_method_instance_t, jl_method_instance_type};

use crate::{
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{
        internal::code_instance::CodeInstanceRef,
        private::WrapperPriv,
        simple_vector::{SimpleVectorData, SimpleVectorRef},
        value::{ValueData, ValueRef},
        Ref,
    },
};

cfg_if! {
    if #[cfg(not(feature = "julia-1-6"))] {
        use std::sync::atomic::Ordering;
    }
}

/// This type is a placeholder to cache data for a specType signature specialization of a `Method`
/// can can be used as a unique dictionary key representation of a call to a particular `Method`
/// with a particular set of argument types.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct MethodInstance<'scope>(NonNull<jl_method_instance_t>, PhantomData<&'scope ()>);

impl<'scope> MethodInstance<'scope> {
    /*
    inspect(Core.MethodInstance):

    def: Union{Method, Module} (mut)
    specTypes: Any (mut)
    sparam_vals: Core.SimpleVector (mut)
    uninferred: Any (mut)
    backedges: Vector{Any} (mut)
    callbacks: Any (mut)
    cache: Core.CodeInstance (mut) _Atomic
    inInference: Bool (mut)
    precompiled: Bool (mut)
    */

    /// pointer back to the context for this code
    pub fn def<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let def = self.unwrap_non_null(Private).as_ref().def.value;
            let def = NonNull::new(def)?;
            Some(ValueRef::wrap(def).root(target))
        }
    }

    /// Argument types this was specialized for
    pub fn spec_types<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let spec_types = self.unwrap_non_null(Private).as_ref().specTypes;
            let spec_types = NonNull::new(spec_types)?;
            Some(ValueRef::wrap(spec_types).root(target))
        }
    }

    /// Static parameter values, indexed by def.method->sparam_syms
    pub fn sparam_vals<'target, T>(self, target: T) -> Option<SimpleVectorData<'target, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let sparam_vals = self.unwrap_non_null(Private).as_ref().sparam_vals;
            let sparam_vals = NonNull::new(sparam_vals)?;
            Some(SimpleVectorRef::wrap(sparam_vals).root(target))
        }
    }

    /// Cached uncompressed code, for generated functions, top-level thunks, or the interpreter
    pub fn uninferred<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let uninferred = self.unwrap_non_null(Private).as_ref().uninferred;
            let uninferred = NonNull::new(uninferred)?;
            Some(ValueRef::wrap(uninferred).root(target))
        }
    }

    /// List of method-instances which contain a call into this method-instance
    pub fn backedges<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let backedges = self.unwrap_non_null(Private).as_ref().backedges;
            let backedges = NonNull::new(backedges)?;
            Some(ValueRef::wrap(backedges.cast()).root(target))
        }
    }

    /// list of callback functions to inform external caches about invalidations
    pub fn callbacks<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let callbacks = self.unwrap_non_null(Private).as_ref().callbacks;
            let callbacks = NonNull::new(callbacks)?;
            Some(ValueRef::wrap(callbacks.cast()).root(target))
        }
    }

    /// The `cache` field.
    pub fn cache<'target, T>(self, target: T) -> Option<CodeInstanceData<'target, T>>
    where
        T: Target<'target>,
    {
        cfg_if! {
            if #[cfg(feature = "julia-1-6")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let cache = self.unwrap_non_null(Private).as_ref().cache;
                    let cache = NonNull::new(cache)?;
                    Some(CodeInstanceRef::wrap(cache).root(target))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let cache = self.unwrap_non_null(Private).as_ref().cache.load(Ordering::Relaxed);
                    let cache = NonNull::new(cache)?;
                    Some(CodeInstanceRef::wrap(cache).root(target))
                }
            }
        }
    }

    /// Flags to tell if inference is running on this object
    pub fn in_inference(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().inInference != 0 }
    }

    /// `true` if this instance was generated by an explicit `precompile(...)` call
    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7")))]
    pub fn precompiled(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().precompiled != 0 }
    }
}

impl_julia_typecheck!(MethodInstance<'scope>, jl_method_instance_type, 'scope);
impl_debug!(MethodInstance<'_>);

impl<'scope> WrapperPriv<'scope, '_> for MethodInstance<'scope> {
    type Wraps = jl_method_instance_t;
    type TypeConstructorPriv<'target, 'da> = MethodInstance<'target>;
    const NAME: &'static str = "MethodInstance";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// A reference to a [`MethodInstance`] that has not been explicitly rooted.
pub type MethodInstanceRef<'scope> = Ref<'scope, 'static, MethodInstance<'scope>>;
impl_valid_layout!(MethodInstanceRef, MethodInstance);

use super::code_instance::CodeInstanceData;
use crate::memory::target::target_type::TargetType;

/// `MethodInstance` or `MethodInstanceRef`, depending on the target type `T`.
pub type MethodInstanceData<'target, T> =
    <T as TargetType<'target>>::Data<'static, MethodInstance<'target>>;

/// `JuliaResult<MethodInstance>` or `JuliaResultRef<MethodInstanceRef>`, depending on the target
/// type `T`.
pub type MethodInstanceResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, MethodInstance<'target>>;
