use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck};
use jl_sys::{jl_task_t, jl_task_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Task<'frame>(*mut jl_task_t, PhantomData<&'frame ()>);

impl<'frame> Task<'frame> {
    pub(crate) unsafe fn wrap(task: *mut jl_task_t) -> Self {
        Task(task, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_task_t {
        self.0
    }
}

impl<'frame> Into<Value<'frame, 'static>> for Task<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Task<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotATask)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(Task<'frame>, jl_task_type, 'frame);
impl_julia_type!(Task<'frame>, jl_task_type, 'frame);
