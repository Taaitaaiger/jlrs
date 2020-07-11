use super::{array::Array, module::Module, symbol::Symbol, Value};
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck};
use jl_sys::{jl_methtable_t, jl_methtable_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct MethodTable<'frame>(*mut jl_methtable_t, PhantomData<&'frame ()>);

impl<'frame> MethodTable<'frame> {
    pub(crate) unsafe fn wrap(method_table: *mut jl_methtable_t) -> Self {
        MethodTable(method_table, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_methtable_t {
        self.0
    }

    pub fn name(self) -> Symbol<'frame> {
        unsafe { Symbol::wrap((&*self.ptr()).name) }
    }

    pub fn max_args(self) -> isize {
        unsafe { (&*self.ptr()).max_args }
    }

    pub fn kwsorter(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).kwsorter) }
    }

    pub fn module(self) -> Module<'frame> {
        unsafe { Module::wrap((&*self.ptr()).module) }
    }

    pub fn backedges(self) -> Array<'frame, 'static> {
        unsafe { Array::wrap((&*self.ptr()).backedges) }
    }

    pub fn offs(self) -> u8 {
        unsafe { (&*self.ptr()).offs }
    }

    pub fn frozen(self) -> u8 {
        unsafe { (&*self.ptr()).frozen }
    }
}

impl<'frame> Into<Value<'frame, 'static>> for MethodTable<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for MethodTable<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAMethTable)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(MethodTable<'frame>, jl_methtable_type, 'frame);
impl_julia_type!(MethodTable<'frame>, jl_methtable_type, 'frame);
