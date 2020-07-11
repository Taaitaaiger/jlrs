use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck};
use jl_sys::{jl_weakref_t, jl_weakref_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct WeakRef<'frame>(*mut jl_weakref_t, PhantomData<&'frame ()>);

impl<'frame> WeakRef<'frame> {
    pub(crate) unsafe fn wrap(weak_ref: *mut jl_weakref_t) -> Self {
        WeakRef(weak_ref, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_weakref_t {
        self.0
    }
}

impl<'frame> Into<Value<'frame, 'static>> for WeakRef<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for WeakRef<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAWeakRef)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(WeakRef<'frame>, jl_weakref_type, 'frame);
impl_julia_type!(WeakRef<'frame>, jl_weakref_type, 'frame);
