//! Wrapper for `TypeMapLevel`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#525

use super::private::Wrapper;
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout};
use crate::{private::Private, wrappers::ptr::ValueRef};
use jl_sys::{jl_typemap_level_t, jl_typemap_level_type};
use std::{marker::PhantomData, ptr::NonNull};

#[cfg(not(feature = "lts"))]
use super::atomic_value;
#[cfg(not(feature = "lts"))]
use std::sync::atomic::Ordering;

/// One level in a TypeMap tree
/// Indexed by key if it is a sublevel in an array
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeMapLevel<'scope>(NonNull<jl_typemap_level_t>, PhantomData<&'scope ()>);

impl<'scope> TypeMapLevel<'scope> {
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
    #[cfg(feature = "lts")]
    pub fn arg1(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().arg1.cast()) }
    }

    /// The `arg1` field.
    #[cfg(not(feature = "lts"))]
    pub fn arg1(self) -> ValueRef<'scope, 'static> {
        unsafe {
            let arg1 = atomic_value(self.unwrap_non_null(Private).as_ref().arg1);
            let ptr = arg1.load(Ordering::Relaxed);
            ValueRef::wrap(ptr.cast())
        }
    }

    /// The `targ` field.
    #[cfg(feature = "lts")]
    pub fn targ(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().targ.cast()) }
    }

    /// The `targ` field.
    #[cfg(not(feature = "lts"))]
    pub fn targ(self) -> ValueRef<'scope, 'static> {
        unsafe {
            let targ = atomic_value(self.unwrap_non_null(Private).as_ref().targ);
            let ptr = targ.load(Ordering::Relaxed);
            ValueRef::wrap(ptr.cast())
        }
    }

    /// The `name1` field.
    #[cfg(feature = "lts")]
    pub fn name1(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().name1.cast()) }
    }

    /// The `name1` field.
    #[cfg(not(feature = "lts"))]
    pub fn name1(self) -> ValueRef<'scope, 'static> {
        unsafe {
            let name1 = atomic_value(self.unwrap_non_null(Private).as_ref().name1);
            let ptr = name1.load(Ordering::Relaxed);
            ValueRef::wrap(ptr.cast())
        }
    }

    /// The `tname` field.
    #[cfg(feature = "lts")]
    pub fn tname(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().tname.cast()) }
    }

    /// The `tname` field.
    #[cfg(not(feature = "lts"))]
    pub fn tname(self) -> ValueRef<'scope, 'static> {
        unsafe {
            let tname = atomic_value(self.unwrap_non_null(Private).as_ref().tname);
            let ptr = tname.load(Ordering::Relaxed);
            ValueRef::wrap(ptr.cast())
        }
    }

    /// The `linear` field, which is called `list` in `Core.TypemapLevel`.
    #[cfg(feature = "lts")]
    pub fn list(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().linear.cast()) }
    }

    /// The `linear` field, which is called `list` in `Core.TypemapLevel`.
    #[cfg(not(feature = "lts"))]
    pub fn list(self) -> ValueRef<'scope, 'static> {
        unsafe {
            let linear = atomic_value(self.unwrap_non_null(Private).as_ref().linear);
            let ptr = linear.load(Ordering::Relaxed);
            ValueRef::wrap(ptr.cast())
        }
    }

    /// The `any` field.
    #[cfg(feature = "lts")]
    pub fn any(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().any.cast()) }
    }

    /// The `any` field.
    #[cfg(not(feature = "lts"))]
    pub fn any(self) -> ValueRef<'scope, 'static> {
        unsafe {
            let any = atomic_value(self.unwrap_non_null(Private).as_ref().any);
            let ptr = any.load(Ordering::Relaxed);
            ValueRef::wrap(ptr.cast())
        }
    }
}

impl_julia_typecheck!(TypeMapLevel<'scope>, jl_typemap_level_type, 'scope);
impl_debug!(TypeMapLevel<'_>);
impl_valid_layout!(TypeMapLevel<'scope>, 'scope);

impl<'scope> Wrapper<'scope, '_> for TypeMapLevel<'scope> {
    type Wraps = jl_typemap_level_t;
    const NAME: &'static str = "TypeMapLevel";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}
