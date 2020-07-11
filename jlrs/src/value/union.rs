use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck};
use jl_sys::{jl_uniontype_t, jl_uniontype_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Union<'frame>(*mut jl_uniontype_t, PhantomData<&'frame ()>);

impl<'frame> Union<'frame> {
    pub(crate) unsafe fn wrap(union: *mut jl_uniontype_t) -> Self {
        Union(union, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_uniontype_t {
        self.0
    }

    pub fn a(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).a) }
    }

    pub fn b(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).b) }
    }
}

impl<'frame> Into<Value<'frame, 'static>> for Union<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Union<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAUnion)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(Union<'frame>, jl_uniontype_type, 'frame);
impl_julia_type!(Union<'frame>, jl_uniontype_type, 'frame);
