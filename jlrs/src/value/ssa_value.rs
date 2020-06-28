use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::value::Value;
use crate::{impl_julia_type, impl_julia_typecheck};
use jl_sys::{jl_ssavalue_t, jl_ssavalue_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct SSAValue<'frame>(*mut jl_ssavalue_t, PhantomData<&'frame ()>);

impl<'frame> SSAValue<'frame> {
    pub(crate) unsafe fn wrap(ssa_value: *mut jl_ssavalue_t) -> Self {
        SSAValue(ssa_value, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_ssavalue_t {
        self.0
    }

    pub fn id(self) -> isize {
        unsafe { (&*self.ptr()).id }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for SSAValue<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAnSSAValue)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(SSAValue<'frame>, jl_ssavalue_type, 'frame);
impl_julia_type!(SSAValue<'frame>, jl_ssavalue_type, 'frame);
