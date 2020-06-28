use super::symbol::Symbol;
use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck};
use jl_sys::{jl_tvar_t, jl_tvar_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct TypeVar<'frame>(*mut jl_tvar_t, PhantomData<&'frame ()>);

impl<'frame> TypeVar<'frame> {
    pub(crate) unsafe fn wrap(type_var: *mut jl_tvar_t) -> Self {
        TypeVar(type_var, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_tvar_t {
        self.0
    }

    pub fn name(self) -> Symbol<'frame> {
        unsafe { Symbol::wrap((&*self.ptr()).name) }
    }

    pub fn lower_bound(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).lb) }
    }

    pub fn upper_bound(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).ub) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for TypeVar<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotATypeVar)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(TypeVar<'frame>, jl_tvar_type, 'frame);
impl_julia_type!(TypeVar<'frame>, jl_tvar_type, 'frame);
