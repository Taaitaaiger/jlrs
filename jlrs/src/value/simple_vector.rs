use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::value::Value;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_simplevector_type, jl_svec_t};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct SimpleVector<'frame>(*mut jl_svec_t, PhantomData<&'frame ()>);

impl<'frame> SimpleVector<'frame> {
    pub(crate) unsafe fn wrap(svec: *mut jl_svec_t) -> Self {
        SimpleVector(svec, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_svec_t {
        self.0
    }

    pub fn len(self) -> usize {
        unsafe { (&*self.ptr()).length }
    }
}

impl<'frame> Into<Value<'frame, 'static>> for SimpleVector<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for SimpleVector<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAnSVec)?
    }

    unsafe fn cast_unchecked<'fr, 'da>(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(SimpleVector<'frame>, jl_simplevector_type, 'frame);
impl_julia_type!(SimpleVector<'frame>, jl_simplevector_type, 'frame);
impl_valid_layout!(SimpleVector<'frame>, 'frame);
