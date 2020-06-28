use crate::{impl_julia_typecheck, impl_julia_type};
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use jl_sys::{jl_simplevector_type, jl_svec_t};
use std::marker::PhantomData;
use crate::value::Value;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct SVec<'frame>(*mut jl_svec_t, PhantomData<&'frame ()>);

impl<'frame> SVec<'frame> {
    pub(crate) unsafe fn wrap(svec: *mut jl_svec_t) -> Self {
        SVec(svec, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_svec_t {
        self.0
    }

    pub fn len(self) -> usize {
        unsafe { (&*self.ptr()).length }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for SVec<'frame> {
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

impl_julia_typecheck!(SVec<'frame>, jl_simplevector_type, 'frame);
impl_julia_type!(SVec<'frame>, jl_simplevector_type, 'frame);
