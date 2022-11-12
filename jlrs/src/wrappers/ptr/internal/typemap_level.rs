//! Wrapper for `TypeMapLevel`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#525

use crate::{
    impl_julia_typecheck,
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
    pub fn arg1(self) -> Option<ValueRef<'scope, 'static>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let arg1 = self.unwrap_non_null(Private).as_ref().arg1;
                    let arg1 = NonNull::new(arg1.cast())?;
                    Some(ValueRef::wrap(arg1))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let arg1 = self.unwrap_non_null(Private).as_ref().arg1.load(Ordering::Relaxed);
                    let arg1 = NonNull::new(arg1.cast())?;
                    Some(ValueRef::wrap(arg1))
                }
            }
        }
    }

    /// The `targ` field.
    pub fn targ(self) -> Option<ValueRef<'scope, 'static>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let targ = self.unwrap_non_null(Private).as_ref().targ;
                    let targ = NonNull::new(targ.cast())?;
                    Some(ValueRef::wrap(targ))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let targ = self.unwrap_non_null(Private).as_ref().targ.load(Ordering::Relaxed);
                    let targ = NonNull::new(targ.cast())?;
                    Some(ValueRef::wrap(targ))
                }
            }
        }
    }

    /// The `name1` field.
    pub fn name1(self) -> Option<ValueRef<'scope, 'static>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let name1 = self.unwrap_non_null(Private).as_ref().name1;
                    let name1 = NonNull::new(name1.cast())?;
                    Some(ValueRef::wrap(name1))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let name1 = self.unwrap_non_null(Private).as_ref().name1.load(Ordering::Relaxed);
                    let name1 = NonNull::new(name1.cast())?;
                    Some(ValueRef::wrap(name1))
                }
            }
        }
    }

    /// The `tname` field.
    pub fn tname(self) -> Option<ValueRef<'scope, 'static>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let tname = self.unwrap_non_null(Private).as_ref().tname;
                    let tname = NonNull::new(tname.cast())?;
                    Some(ValueRef::wrap(tname))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let tname = self.unwrap_non_null(Private).as_ref().tname.load(Ordering::Relaxed);
                    let tname = NonNull::new(tname.cast())?;
                    Some(ValueRef::wrap(tname))
                }
            }
        }
    }

    /// The `linear` field, which is called `list` in `Core.TypemapLevel`.
    pub fn list(self) -> Option<ValueRef<'scope, 'static>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let linear = self.unwrap_non_null(Private).as_ref().linear;
                    let linear = NonNull::new(linear.cast())?;
                    Some(ValueRef::wrap(linear))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let linear = self.unwrap_non_null(Private).as_ref().linear.load(Ordering::Relaxed);
                    let linear = NonNull::new(linear.cast())?;
                    Some(ValueRef::wrap(linear))
                }
            }
        }
    }

    /// The `any` field.
    pub fn any(self) -> Option<ValueRef<'scope, 'static>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let any = self.unwrap_non_null(Private).as_ref().any;
                    let any = NonNull::new(any.cast())?;
                    Some(ValueRef::wrap(any))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let any = self.unwrap_non_null(Private).as_ref().any.load(Ordering::Relaxed);
                    let any = NonNull::new(any.cast())?;
                    Some(ValueRef::wrap(any))
                }
            }
        }
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
