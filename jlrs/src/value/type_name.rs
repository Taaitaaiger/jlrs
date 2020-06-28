use super::{
    array::Array, meth_table::MethTable, module::Module, s_vec::SVec, symbol::Symbol, Value,
};
use crate::{impl_julia_typecheck, impl_julia_type};
use crate::traits::Cast;
use crate::error::{JlrsError, JlrsResult};
use jl_sys::{jl_typename_t, jl_typename_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct TypeName<'frame>(*mut jl_typename_t, PhantomData<&'frame ()>);

impl<'frame> TypeName<'frame> {
    pub(crate) unsafe fn wrap(typename: *mut jl_typename_t) -> Self {
        TypeName(typename, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_typename_t {
        self.0
    }

    pub fn name(self) -> Symbol<'frame> {
        unsafe { Symbol::wrap((&*self.ptr()).name) }
    }

    pub fn module(self) -> Module<'frame> {
        unsafe { Module::wrap((&*self.ptr()).module) }
    }

    pub fn names(self) -> SVec<'frame> {
        unsafe { SVec::wrap((&*self.ptr()).names) }
    }

    pub fn wrapper(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).wrapper) }
    }

    pub fn cache(self) -> SVec<'frame> {
        unsafe { SVec::wrap((&*self.ptr()).cache) }
    }

    pub fn linearcache(self) -> SVec<'frame> {
        unsafe { SVec::wrap((&*self.ptr()).linearcache) }
    }

    pub fn hash(self) -> isize {
        unsafe { (&*self.ptr()).hash }
    }

    pub fn mt(self) -> MethTable<'frame> {
        unsafe { MethTable::wrap((&*self.ptr()).mt) }
    }

    pub fn partial(self) -> Array<'frame, 'static> {
        unsafe { Array::wrap((&*self.ptr()).partial) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for TypeName<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotASymbol)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(TypeName<'frame>, jl_typename_type, 'frame);
impl_julia_type!(TypeName<'frame>, jl_typename_type, 'frame);
