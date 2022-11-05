//! Wrapper for `TypeMapLevel`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#525

use crate::{
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{private::WrapperPriv, value::ValueRef, Ref},
};
use cfg_if::cfg_if;
use jl_sys::{jl_typemap_level_t, jl_typemap_level_type};
use std::{marker::PhantomData, ptr::NonNull};

cfg_if! {
    if #[cfg(not(feature = "lts"))] {
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
    pub fn arg1(self) -> ValueRef<'scope, 'static> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().arg1.cast()) }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let arg1 = self.unwrap_non_null(Private).as_ref().arg1.load(Ordering::Relaxed);
                    ValueRef::wrap(arg1.cast())
                }
            }
        }
    }

    /// The `targ` field.
    pub fn targ(self) -> ValueRef<'scope, 'static> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().targ.cast()) }
            } else {
                unsafe {
                    // Safety: the pointer points to valid data
                    let targ = self.unwrap_non_null(Private).as_ref().targ.load(Ordering::Relaxed);
                    ValueRef::wrap(targ.cast())
                }
            }
        }
    }

    /// The `name1` field.
    pub fn name1(self) -> ValueRef<'scope, 'static> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().name1.cast()) }
            } else {
                unsafe {
                    // Safety: the pointer points to valid data
                    let name1 = self.unwrap_non_null(Private).as_ref().name1.load(Ordering::Relaxed);
                    ValueRef::wrap(name1.cast())
                }
            }
        }
    }

    /// The `tname` field.
    pub fn tname(self) -> ValueRef<'scope, 'static> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().tname.cast()) }
            } else {
                unsafe {
                    // Safety: the pointer points to valid data
                    let tname = self.unwrap_non_null(Private).as_ref().tname.load(Ordering::Relaxed);
                    ValueRef::wrap(tname.cast())
                }
            }
        }
    }

    /// The `linear` field, which is called `list` in `Core.TypemapLevel`.
    pub fn list(self) -> ValueRef<'scope, 'static> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().linear.cast()) }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let linear = self.unwrap_non_null(Private).as_ref().linear.load(Ordering::Relaxed);
                    ValueRef::wrap(linear.cast())
                }
            }
        }
    }

    /// The `any` field.
    pub fn any(self) -> ValueRef<'scope, 'static> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().any.cast()) }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let any = self.unwrap_non_null(Private).as_ref().any.load(Ordering::Relaxed);
                    ValueRef::wrap(any.cast())
                }
            }
        }
    }

    /// Use the target to reroot this data.
    pub fn root<'target, T>(self, target: T) -> TypeMapLevelData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the data is valid.
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
    }
}

impl_julia_typecheck!(TypeMapLevel<'scope>, jl_typemap_level_type, 'scope);
impl_debug!(TypeMapLevel<'_>);

impl<'scope> WrapperPriv<'scope, '_> for TypeMapLevel<'scope> {
    type Wraps = jl_typemap_level_t;
    type TypeConstructorPriv<'target, 'da> = TypeMapLevel<'target>;
    const NAME: &'static str = "TypeMapLevel";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(TypeMapLevel, 1);

/// A reference to a [`TypeMapLevel`] that has not been explicitly rooted.
pub type TypeMapLevelRef<'scope> = Ref<'scope, 'static, TypeMapLevel<'scope>>;
impl_valid_layout!(TypeMapLevelRef, TypeMapLevel);
impl_ref_root!(TypeMapLevel, TypeMapLevelRef, 1);

use crate::memory::target::target_type::TargetType;

/// `TypeMaLevely` or `TypeMaLevelyRef`, depending on the target type `T`.
pub type TypeMapLevelData<'target, T> =
    <T as TargetType<'target>>::Data<'static, TypeMapLevel<'target>>;

/// `JuliaResult<TypeMaLevely>` or `JuliaResultRef<TypeMapLevelRef>`, depending on the target type
/// `T`.
pub type TypeMapLevelResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, TypeMapLevel<'target>>;
