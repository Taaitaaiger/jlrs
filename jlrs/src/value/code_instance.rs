//! Support for values with the `Core.CodeInstance` type.

use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_code_instance_t, jl_code_instance_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct CodeInstance<'frame>(*mut jl_code_instance_t, PhantomData<&'frame ()>);

impl<'frame> CodeInstance<'frame> {
    pub(crate) unsafe fn wrap(code_instance: *mut jl_code_instance_t) -> Self {
        CodeInstance(code_instance, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_code_instance_t {
        self.0
    }
}

impl<'frame> Into<Value<'frame, 'static>> for CodeInstance<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for CodeInstance<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotACodeInstance)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(CodeInstance<'frame>, jl_code_instance_type, 'frame);
impl_julia_type!(CodeInstance<'frame>, jl_code_instance_type, 'frame);
impl_valid_layout!(CodeInstance<'frame>, 'frame);