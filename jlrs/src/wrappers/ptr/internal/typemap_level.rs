//! Wrapper for `TypeMapLevel`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#525

use crate::{
    impl_debug, impl_julia_typecheck,
    memory::output::Output,
    private::Private,
    wrappers::ptr::{private::Wrapper as WrapperPriv, ValueRef},
};
use cfg_if::cfg_if;
use jl_sys::{jl_typemap_level_t, jl_typemap_level_type};
use std::{marker::PhantomData, ptr::NonNull};

cfg_if! {
    if #[cfg(not(feature = "lts"))] {
        use crate::wrappers::ptr::atomic_value;
        use std::sync::atomic::Ordering;
    }
}

/// One level in a TypeMap tree
/// Indexed by key if it is a sublevel in an array
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeMapLevel<'scope>(NonNull<jl_typemap_level_t>, PhantomData<&'scope ()>);

impl<'scope> TypeMapLevel<'scope> {
    /*
    for (a,b) in zip(fieldnames(Core.TypeMapLevel), fieldtypes(Core.TypeMapLevel))
         println(a,": ", b)
    end
    arg1: Any _Atomic
    targ: Any _Atomic
    name1: Any _Atomic
    tname: Any _Atomic
    list: Any _Atomic
    any: Any _Atomic
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
            let arg1 = atomic_value(&mut self.unwrap_non_null(Private).as_mut().arg1 as *mut _);
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
            let targ = atomic_value(&mut self.unwrap_non_null(Private).as_mut().targ as *mut _);
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
            let name1 = atomic_value(&mut self.unwrap_non_null(Private).as_mut().name1 as *mut _);
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
            let tname = atomic_value(&mut self.unwrap_non_null(Private).as_mut().tname as *mut _);
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
            let linear = atomic_value(&mut self.unwrap_non_null(Private).as_mut().linear as *mut _);
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
            let any = atomic_value(&mut self.unwrap_non_null(Private).as_mut().any as *mut _);
            let ptr = any.load(Ordering::Relaxed);
            ValueRef::wrap(ptr.cast())
        }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> TypeMapLevel<'target> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<TypeMapLevel>(ptr);
            TypeMapLevel::wrap_non_null(ptr, Private)
        }
    }
}

impl_julia_typecheck!(TypeMapLevel<'scope>, jl_typemap_level_type, 'scope);
impl_debug!(TypeMapLevel<'_>);

impl<'scope> WrapperPriv<'scope, '_> for TypeMapLevel<'scope> {
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

impl_root!(TypeMapLevel, 1);
