//! Wrapper for `CodeInstance`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L273

use super::private::Wrapper;
use crate::{impl_debug, impl_julia_typecheck};
use crate::{
    private::Private,
    wrappers::ptr::{CodeInstanceRef, MethodInstanceRef, ValueRef},
};
use jl_sys::{jl_code_instance_t, jl_code_instance_type};
use std::{marker::PhantomData, ptr::NonNull};

/// A `CodeInstance` represents an executable operation.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct CodeInstance<'scope>(NonNull<jl_code_instance_t>, PhantomData<&'scope ()>);

impl<'scope> CodeInstance<'scope> {
    /*
    for (a, b) in zip(fieldnames(Core.CodeInstance), fieldtypes(Core.CodeInstance))
        println(a, ": ", b)
    end
    def: Core.MethodInstance
    next: Core.CodeInstance
    min_world: UInt64
    max_world: UInt64
    rettype: Any
    rettype_const: Any
    inferred: Any
    isspecsig: Bool
    precompile: Bool
    invoke: Ptr{Nothing}
    specptr: Ptr{Nothing}
    */

    /// Method this instance is specialized from.
    pub fn def(self) -> MethodInstanceRef<'scope> {
        unsafe { MethodInstanceRef::wrap(self.unwrap_non_null(Private).as_ref().def) }
    }

    /// Next cache entry.
    pub fn next(self) -> CodeInstanceRef<'scope> {
        unsafe { CodeInstanceRef::wrap(self.unwrap_non_null(Private).as_ref().next) }
    }

    /// Returns the minimum of the world range for which this object is valid to use.
    pub fn min_world(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().min_world }
    }

    /// Returns the maximum of the world range for which this object is valid to use.
    pub fn max_world(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().max_world }
    }

    /// Return type for fptr.
    pub fn rettype(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().rettype) }
    }

    /// Inferred constant return value, or null
    pub fn rettype_const(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().rettype_const) }
    }

    /// Inferred `CodeInfo`, `Nothing`, or `None`.
    pub fn inferred(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().inferred) }
    }

    /// If `specptr` is a specialized function signature for specTypes->rettype
    pub fn is_specsig(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().isspecsig != 0 }
    }

    /// If `specptr` is a specialized function signature for specTypes->rettype
    pub fn precompile(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().precompile != 0 }
    }
}

impl_julia_typecheck!(CodeInstance<'scope>, jl_code_instance_type, 'scope);
impl_debug!(CodeInstance<'_>);

impl<'scope> Wrapper<'scope, '_> for CodeInstance<'scope> {
    type Internal = jl_code_instance_t;
    const NAME: &'static str = "CodeInstance";

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, ::std::marker::PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
