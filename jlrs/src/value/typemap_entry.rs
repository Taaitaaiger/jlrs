//! Support for values with the `Core.TypeMapEntry` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#505

use super::datatype::DataType;
use super::simple_vector::SimpleVector;
use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_typemap_entry_t, jl_typemap_entry_type};
use std::{fmt::{Debug, Formatter, Result as FmtResult}, marker::PhantomData};

/// One Type-to-Value entry
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

    /// Invasive linked list
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

    /// The type signature for this entry
    pub fn signature(self) -> DataType<'frame> {
        unsafe { DataType::wrap((&*self.ptr()).sig) }
    }

    /// A simple signature for fast rejection
    pub fn simple_signature(self) -> DataType<'frame> {
        unsafe { DataType::wrap((&*self.ptr()).simplesig) }
    }

    /// The `guard_signature` field.
    pub fn guard_signature(self) -> SimpleVector<'frame> {
        unsafe { SimpleVector::wrap((&*self.ptr()).guardsigs) }
    }

    /// The `min_world` field.
    pub fn min_world(self) -> usize {
        unsafe { (&*self.ptr()).min_world }
    }

    /// The `max_world` field.
    pub fn max_world(self) -> usize {
        unsafe { (&*self.ptr()).max_world }
    }

    /// The `func` field.
    pub fn func(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).func.value) }
    }

    /// `isleaftype(sig) & !any(isType, sig)` : unsorted and very fast
    pub fn is_leaf_signature(self) -> bool {
        unsafe { (&*self.ptr()).isleafsig != 0 }
    }

    /// `all(isleaftype | isAny | isType | isVararg, sig)` : sorted and fast
    pub fn is_simple_signature(self) -> bool {
        unsafe { (&*self.ptr()).issimplesig != 0 }
    }

    /// `isVararg(sig)`
    pub fn is_vararg(self) -> bool {
        unsafe { (&*self.ptr()).va != 0 }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for TypeMapEntry<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("TypeMapEntry").finish()
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
