//! Support for values with the `Core.TypeMapLevel` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#525

use super::{wrapper_ref::ValueRef, Value};
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_typemap_level_t, jl_typemap_level_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// One level in a TypeMap tree
/// Indexed by key if it is a sublevel in an array
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct TypeMapLevel<'frame>(NonNull<jl_typemap_level_t>, PhantomData<&'frame ()>);

impl<'frame> TypeMapLevel<'frame> {
    pub(crate) unsafe fn wrap(typemap_level: *mut jl_typemap_level_t) -> Self {
        debug_assert!(!typemap_level.is_null());
        TypeMapLevel(NonNull::new_unchecked(typemap_level), PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_typemap_level_t> {
        self.0
    }

    /*
    for (a,b) in zip(fieldnames(Core.TypeMapEntry), fieldtypes(Core.TypeMapEntry))
         println(a,": ", b)
    end
    arg1: Any
    targ: Any
    name1: Any
    tname: Any
    list: Any
    any: Any
    */

    /// The `arg1` field.
    pub fn arg1(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).arg1.cast()) }
    }

    /// The `targ` field.
    pub fn targ(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).targ.cast()) }
    }

    /// The `name1` field.
    pub fn name1(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).name1.cast()) }
    }

    /// The `tname` field.
    pub fn tname(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).tname.cast()) }
    }

    /// The `linear` field, which is called `list` in `Core.TypemapLevel`.
    pub fn list(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).linear.cast()) }
    }

    /// The `any` field.
    pub fn any(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).any) }
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
        unsafe { Value::wrap_non_null(self.inner().cast()) }
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
        Self::wrap(value.inner().as_ptr().cast())
    }
}

impl_julia_typecheck!(TypeMapLevel<'frame>, jl_typemap_level_type, 'frame);

impl_valid_layout!(TypeMapLevel<'frame>, 'frame);
