//! Wrapper for `MethodInstance`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L321
use super::private::Wrapper;
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout};
use crate::{
    private::Private,
    wrappers::ptr::{CodeInstanceRef, SimpleVectorRef, ValueRef},
};
use jl_sys::{jl_method_instance_t, jl_method_instance_type};
use std::{marker::PhantomData, ptr::NonNull};

/// This type is a placeholder to cache data for a specType signature specialization of a `Method`
/// can can be used as a unique dictionary key representation of a call to a particular `Method`
/// with a particular set of argument types.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct MethodInstance<'scope>(NonNull<jl_method_instance_t>, PhantomData<&'scope ()>);

impl<'scope> MethodInstance<'scope> {
    /*
    for (a, b) in zip(fieldnames(Core.MethodInstance), fieldtypes(Core.MethodInstance))
        println(a, ": ", b)
    end
    def: Union{Method, Module}
    specTypes: Any
    sparam_vals: Core.SimpleVector
    uninferred: Any
    backedges: Any
    callbacks: Any
    cache: Core.CodeInstance
    inInference: Bool
    */

    /// Context for this code
    pub fn def(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().def.value) }
    }

    /// Argument types this was specialized for
    pub fn spec_types(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().specTypes) }
    }

    /// Static parameter values, indexed by def.method->sparam_syms
    pub fn sparam_vals(self) -> SimpleVectorRef<'scope> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().sparam_vals) }
    }

    /// Cached uncompressed code, for generated functions, top-level thunks, or the interpreter
    pub fn uninferred(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().uninferred) }
    }

    /// List of method-instances which contain a call into this method-instance
    pub fn backedges(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().backedges.cast()) }
    }

    /// The `callbacks` field.
    pub fn callbacks(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().callbacks.cast()) }
    }

    /// The `cache` field.
    pub fn cache(self) -> CodeInstanceRef<'scope> {
        unsafe { CodeInstanceRef::wrap(self.unwrap_non_null(Private).as_ref().cache) }
    }

    /// Flags to tell if inference is running on this object
    pub fn in_inference(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().inInference != 0 }
    }
}

impl_julia_typecheck!(MethodInstance<'scope>, jl_method_instance_type, 'scope);
impl_valid_layout!(MethodInstance<'scope>, 'scope);
impl_debug!(MethodInstance<'_>);

impl<'scope> Wrapper<'scope, '_> for MethodInstance<'scope> {
    type Wraps = jl_method_instance_t;
    const NAME: &'static str = "MethodInstance";

    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}
