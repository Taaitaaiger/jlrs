use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck};
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
