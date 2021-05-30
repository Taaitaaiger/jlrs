//! Wrapper for `Core.TypeMapLevel`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#525

use super::private::Wrapper;
use crate::{impl_julia_typecheck, impl_valid_layout};
use crate::{private::Private, wrappers::ptr::ValueRef};
use jl_sys::{jl_typemap_level_t, jl_typemap_level_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// One level in a TypeMap tree
/// Indexed by key if it is a sublevel in an array
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeMapLevel<'frame>(NonNull<jl_typemap_level_t>, PhantomData<&'frame ()>);

impl<'frame> TypeMapLevel<'frame> {
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
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().arg1.cast()) }
    }

    /// The `targ` field.
    pub fn targ(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().targ.cast()) }
    }

    /// The `name1` field.
    pub fn name1(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().name1.cast()) }
    }

    /// The `tname` field.
    pub fn tname(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().tname.cast()) }
    }

    /// The `linear` field, which is called `list` in `Core.TypemapLevel`.
    pub fn list(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().linear.cast()) }
    }

    /// The `any` field.
    pub fn any(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().any) }
    }
}

impl<'scope> Debug for TypeMapLevel<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("TypeMapLevel").finish()
    }
}

impl_julia_typecheck!(TypeMapLevel<'frame>, jl_typemap_level_type, 'frame);

impl_valid_layout!(TypeMapLevel<'frame>, 'frame);

impl<'scope> Wrapper<'scope, '_> for TypeMapLevel<'scope> {
    type Internal = jl_typemap_level_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
