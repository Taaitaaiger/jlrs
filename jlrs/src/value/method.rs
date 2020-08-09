//! Support for values with the `Core.Method` type.

use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_method_t, jl_method_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Method<'frame>(*mut jl_method_t, PhantomData<&'frame ()>);

impl<'frame> Method<'frame> {
    pub(crate) unsafe fn wrap(method: *mut jl_method_t) -> Self {
        Method(method, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_method_t {
        self.0
    }
}

impl<'frame> Into<Value<'frame, 'static>> for Method<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Method<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAMethod)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(Method<'frame>, jl_method_type, 'frame);
impl_julia_type!(Method<'frame>, jl_method_type, 'frame);
impl_valid_layout!(Method<'frame>, 'frame);
