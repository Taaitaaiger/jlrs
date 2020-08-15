//! Support for values with the `Core.MethodInstance` type.

use super::array::Array;
use super::code_instance::CodeInstance;
use super::simple_vector::SimpleVector;
use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_method_instance_t, jl_method_instance_type};
use std::marker::PhantomData;

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

    pub fn def(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).def.value) }
    }

    pub fn spec_types(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).specTypes) }
    }

    pub fn sparam_vals(self) -> SimpleVector<'frame> {
        unsafe { SimpleVector::wrap((&*self.ptr()).sparam_vals) }
    }

    pub fn uninferred(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).uninferred) }
    }

    pub fn backedges(self) -> Array<'frame, 'static> {
        unsafe { Array::wrap((&*self.ptr()).backedges) }
    }

    pub fn cache(self) -> CodeInstance<'frame> {
        unsafe { CodeInstance::wrap((&*self.ptr()).cache) }
    }

    pub fn in_inference(self) -> u8 {
        unsafe { (&*self.ptr()).inInference }
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
