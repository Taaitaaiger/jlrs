//! Support for values with the `Core.MethodInstance` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L321
use super::wrapper_ref::{CodeInstanceRef, SimpleVectorRef, ValueRef};
use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_method_instance_t, jl_method_instance_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// This type is a placeholder to cache data for a specType signature specialization of a `Method`
/// can can be used as a unique dictionary key representation of a call to a particular `Method`
/// with a particular set of argument types
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct MethodInstance<'frame>(NonNull<jl_method_instance_t>, PhantomData<&'frame ()>);

impl<'frame> MethodInstance<'frame> {
    pub(crate) unsafe fn wrap(method_instance: *mut jl_method_instance_t) -> Self {
        debug_assert!(!method_instance.is_null());
        MethodInstance(NonNull::new_unchecked(method_instance), PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_method_instance_t> {
        self.0
    }

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
    pub fn def(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).def.value) }
    }

    /// Argument types this was specialized for
    pub fn spec_types(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).specTypes) }
    }

    /// Static parameter values, indexed by def.method->sparam_syms
    pub fn sparam_vals(self) -> SimpleVectorRef<'frame> {
        unsafe { SimpleVectorRef::wrap((&*self.inner().as_ptr()).sparam_vals) }
    }

    /// Cached uncompressed code, for generated functions, top-level thunks, or the interpreter
    pub fn uninferred(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).uninferred) }
    }

    /// List of method-instances which contain a call into this method-instance
    pub fn backedges(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).backedges.cast()) }
    }

    /// The `callbacks` field.
    pub fn callbacks(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).callbacks.cast()) }
    }

    /// The `cache` field.
    pub fn cache(self) -> CodeInstanceRef<'frame> {
        unsafe { CodeInstanceRef::wrap((&*self.inner().as_ptr()).cache) }
    }

    /// Flags to tell if inference is running on this object
    pub fn in_inference(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).inInference != 0 }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for MethodInstance<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("MethodInstance").finish()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for MethodInstance<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap_non_null(self.inner().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for MethodInstance<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAMethodInstance)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.inner().as_ptr().cast())
    }
}

impl_julia_typecheck!(MethodInstance<'frame>, jl_method_instance_type, 'frame);

impl_valid_layout!(MethodInstance<'frame>, 'frame);
