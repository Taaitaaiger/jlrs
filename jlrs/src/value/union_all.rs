use super::type_var::TypeVar;
use super::Value;
use crate::traits::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_typecheck, impl_julia_type};
use jl_sys::{jl_unionall_t, jl_unionall_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct UnionAll<'frame>(*mut jl_unionall_t, PhantomData<&'frame ()>);

impl<'frame> UnionAll<'frame> {
    pub(crate) unsafe fn wrap(union_all: *mut jl_unionall_t) -> Self {
        UnionAll(union_all, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_unionall_t {
        self.0
    }

    pub fn body(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).body) }
    }

    pub fn var(self) -> TypeVar<'frame> {
        unsafe { TypeVar::wrap((&*self.ptr()).var) }
    }
}


unsafe impl<'frame, 'data> Cast<'frame, 'data> for UnionAll<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAUnionAll)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(UnionAll<'frame>, jl_unionall_type, 'frame);
impl_julia_type!(UnionAll<'frame>, jl_unionall_type, 'frame);
