//! Support for values with the `Core.TypeMapLevel` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#525

use super::array::Array;
use super::typemap_entry::TypeMapEntry;
use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_typemap_level_t, jl_typemap_level_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
};

/// One level in a TypeMap tree
/// Indexed by key if it is a sublevel in an array
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct TypeMapLevel<'frame>(*mut jl_typemap_level_t, PhantomData<&'frame ()>);

impl<'frame> TypeMapLevel<'frame> {
    pub(crate) unsafe fn wrap(typemap_level: *mut jl_typemap_level_t) -> Self {
        TypeMapLevel(typemap_level, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_typemap_level_t {
        self.0
    }

    /// The `arg1` field.
    pub fn arg1(self) -> Array<'frame, 'static> {
        unsafe { Array::wrap((&*self.ptr()).arg1) }
    }

    /// The `targ` field.
    pub fn targ(self) -> Array<'frame, 'static> {
        unsafe { Array::wrap((&*self.ptr()).targ) }
    }

    /// The `linear` field.
    pub fn linear(self) -> TypeMapEntry<'frame> {
        unsafe { TypeMapEntry::wrap((&*self.ptr()).linear) }
    }

    /// The `any` field.
    pub fn any(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).any) }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for TypeMapLevel<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("TypeMapLevel").finish()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for TypeMapLevel<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for TypeMapLevel<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotATypeMapLevel)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(TypeMapLevel<'frame>, jl_typemap_level_type, 'frame);
impl_julia_type!(TypeMapLevel<'frame>, jl_typemap_level_type, 'frame);
impl_valid_layout!(TypeMapLevel<'frame>, 'frame);
