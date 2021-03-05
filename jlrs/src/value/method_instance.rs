//! Support for values with the `Core.MethodInstance` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L321
use super::array::Array;
use super::code_instance::CodeInstance;
use super::simple_vector::SimpleVector;
use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_method_instance_t, jl_method_instance_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
};

/// This type is a placeholder to cache data for a specType signature specialization of a `Method`
/// can can be used as a unique dictionary key representation of a call to a particular `Method`
/// with a particular set of argument types
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct MethodInstance<'frame>(*mut jl_method_instance_t, PhantomData<&'frame ()>);

impl<'frame> MethodInstance<'frame> {
    pub(crate) unsafe fn wrap(method_instance: *mut jl_method_instance_t) -> Self {
        MethodInstance(method_instance, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_method_instance_t {
        self.0
    }

    /// Context for this code
    pub fn def(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).def.value) }
    }

    // Argument types this was specialized for
    pub fn spec_types(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).specTypes) }
    }

    // Static parameter values, indexed by def.method->sparam_syms
    pub fn sparam_vals(self) -> SimpleVector<'frame> {
        unsafe { SimpleVector::wrap((&*self.ptr()).sparam_vals) }
    }

    // Cached uncompressed code, for generated functions, top-level thunks, or the interpreter
    pub fn uninferred(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).uninferred) }
    }

    /// List of method-instances which contain a call into this method-instance
    pub fn backedges(self) -> Array<'frame, 'static> {
        unsafe { Array::wrap((&*self.ptr()).backedges) }
    }

    /// The `cache` field.
    pub fn cache(self) -> CodeInstance<'frame> {
        unsafe { CodeInstance::wrap((&*self.ptr()).cache) }
    }

    /// Flags to tell if inference is running on this object
    pub fn in_inference(self) -> u8 {
        unsafe { (&*self.ptr()).inInference }
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
        unsafe { Value::wrap(self.ptr().cast()) }
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
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(MethodInstance<'frame>, jl_method_instance_type, 'frame);
impl_julia_type!(MethodInstance<'frame>, jl_method_instance_type, 'frame);
impl_valid_layout!(MethodInstance<'frame>, 'frame);
