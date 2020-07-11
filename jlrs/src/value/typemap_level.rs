use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck};
use jl_sys::{jl_typemap_level_t, jl_typemap_level_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct TypeMapLevel<'frame>(*mut jl_typemap_level_t, PhantomData<&'frame ()>);

impl<'frame> TypeMapLevel<'frame> {
    pub(crate) unsafe fn wrap(typemap_level: *mut jl_typemap_level_t) -> Self {
        TypeMapLevel(typemap_level, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_typemap_level_t {
        self.0
    }
}

impl<'frame> Into<Value<'frame, 'static>> for TypeMapLevel<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for TypeMapLevel<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotATypeMapLevel)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(TypeMapLevel<'frame>, jl_typemap_level_type, 'frame);
impl_julia_type!(TypeMapLevel<'frame>, jl_typemap_level_type, 'frame);
