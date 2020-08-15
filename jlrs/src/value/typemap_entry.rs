//! Support for values with the `Core.TypeMapEntry` type.

use super::Value;
use super::simple_vector::SimpleVector;
use super::datatype::DataType;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_typemap_entry_t, jl_typemap_entry_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct TypeMapEntry<'frame>(*mut jl_typemap_entry_t, PhantomData<&'frame ()>);

impl<'frame> TypeMapEntry<'frame> {
    pub(crate) unsafe fn wrap(typemap_entry: *mut jl_typemap_entry_t) -> Self {
        TypeMapEntry(typemap_entry, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_typemap_entry_t {
        self.0
    }

    pub fn next(self) -> Option<Self> {
        unsafe {
            let next = (&*self.ptr()).next;
            if next.is_null() {
                None
            } else {
                Some(TypeMapEntry::wrap(next))
            }
        }
    }

    pub fn signature(self) -> DataType<'frame> {
        unsafe {
            DataType::wrap((&*self.ptr()).sig)
        }
    }

    pub fn simple_signature(self) -> DataType<'frame> {
        unsafe {
            DataType::wrap((&*self.ptr()).simplesig)
        }
    }

    pub fn guard_signature(self) -> SimpleVector<'frame> {
        unsafe {
            SimpleVector::wrap((&*self.ptr()).guardsigs)
        }
    }

    pub fn min_world(self) -> usize{
        unsafe {
            (&*self.ptr()).min_world
        }
    }

    pub fn max_world(self) -> usize{
        unsafe {
            (&*self.ptr()).max_world
        }
    }

    pub fn func(self) -> Value<'frame, 'static> {
        unsafe {
            Value::wrap((&*self.ptr()).func.value)
        }
    }

    pub fn is_leaf_signature(self) -> bool {
        unsafe {
            (&*self.ptr()).isleafsig != 0
        }
    }

    pub fn is_simple_signature(self) -> bool {
        unsafe {
            (&*self.ptr()).issimplesig != 0
        }
    }

    pub fn is_vararg(self) -> bool {
        unsafe {
            (&*self.ptr()).va != 0
        }
    }
}

impl<'frame> Into<Value<'frame, 'static>> for TypeMapEntry<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for TypeMapEntry<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotATypeMapEntry)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(TypeMapEntry<'frame>, jl_typemap_entry_type, 'frame);
impl_julia_type!(TypeMapEntry<'frame>, jl_typemap_entry_type, 'frame);
impl_valid_layout!(TypeMapEntry<'frame>, 'frame);
