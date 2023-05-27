//! Managed type for `TypeMapLevel`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#525

#[julia_version(since = "1.7")]
use std::sync::atomic::Ordering;
use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_typemap_level_t, jl_typemap_level_type};
use jlrs_macros::julia_version;

use crate::{
    data::{managed::{
        private::ManagedPriv,
        value::{ValueData, ValueRef},
        Ref,
    }},
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
};

/// One level in a TypeMap tree
/// Indexed by key if it is a sublevel in an array
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeMapLevel<'scope>(NonNull<jl_typemap_level_t>, PhantomData<&'scope ()>);

impl<'scope> TypeMapLevel<'scope> {
    /*
    inspect(Core.TypeMapLevel):

    arg1: Any (mut) _Atomic
    targ: Any (mut) _Atomic
    name1: Any (mut) _Atomic
    tname: Any (mut) _Atomic
    list: Any (mut) _Atomic
    any: Any (mut) _Atomic
    */

    #[julia_version(until = "1.6")]
    /// The `arg1` field.
    pub fn arg1<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let arg1 = self.unwrap_non_null(Private).as_ref().arg1;
            let arg1 = NonNull::new(arg1.cast())?;
            Some(ValueRef::wrap(arg1).root(target))
        }
    }

    #[julia_version(since = "1.7")]
    /// The `arg1` field.
    pub fn arg1<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let arg1 = self
                .unwrap_non_null(Private)
                .as_ref()
                .arg1
                .load(Ordering::Relaxed);
            let arg1 = NonNull::new(arg1.cast())?;
            Some(ValueRef::wrap(arg1).root(target))
        }
    }

    #[julia_version(until = "1.6")]
    /// The `targ` field.
    pub fn targ<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let targ = self.unwrap_non_null(Private).as_ref().targ;
            let targ = NonNull::new(targ.cast())?;
            Some(ValueRef::wrap(targ).root(target))
        }
    }

    #[julia_version(since = "1.7")]
    /// The `targ` field.
    pub fn targ<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let targ = self
                .unwrap_non_null(Private)
                .as_ref()
                .targ
                .load(Ordering::Relaxed);
            let targ = NonNull::new(targ.cast())?;
            Some(ValueRef::wrap(targ).root(target))
        }
    }

    #[julia_version(until = "1.6")]
    /// The `name1` field.
    pub fn name1<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let name1 = self.unwrap_non_null(Private).as_ref().name1;
            let name1 = NonNull::new(name1.cast())?;
            Some(ValueRef::wrap(name1).root(target))
        }
    }

    #[julia_version(since = "1.7")]
    /// The `name1` field.
    pub fn name1<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let name1 = self
                .unwrap_non_null(Private)
                .as_ref()
                .name1
                .load(Ordering::Relaxed);
            let name1 = NonNull::new(name1.cast())?;
            Some(ValueRef::wrap(name1).root(target))
        }
    }

    #[julia_version(until = "1.6")]
    /// The `tname` field.
    pub fn tname<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let tname = self.unwrap_non_null(Private).as_ref().tname;
            let tname = NonNull::new(tname.cast())?;
            Some(ValueRef::wrap(tname).root(target))
        }
    }

    #[julia_version(since = "1.7")]
    /// The `tname` field.
    pub fn tname<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let tname = self
                .unwrap_non_null(Private)
                .as_ref()
                .tname
                .load(Ordering::Relaxed);
            let tname = NonNull::new(tname.cast())?;
            Some(ValueRef::wrap(tname).root(target))
        }
    }

    #[julia_version(until = "1.6")]
    /// The `linear` field, which is called `list` in `Core.TypemapLevel`.
    pub fn list<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let linear = self.unwrap_non_null(Private).as_ref().linear;
            let linear = NonNull::new(linear.cast())?;
            Some(ValueRef::wrap(linear).root(target))
        }
    }

    #[julia_version(since = "1.7")]
    /// The `linear` field, which is called `list` in `Core.TypemapLevel`.
    pub fn list<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let linear = self
                .unwrap_non_null(Private)
                .as_ref()
                .linear
                .load(Ordering::Relaxed);
            let linear = NonNull::new(linear.cast())?;
            Some(ValueRef::wrap(linear).root(target))
        }
    }

    #[julia_version(until = "1.6")]
    /// The `any` field.
    pub fn any<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let any = self.unwrap_non_null(Private).as_ref().any;
            let any = NonNull::new(any.cast())?;
            Some(ValueRef::wrap(any).root(target))
        }
    }

    #[julia_version(since = "1.7")]
    /// The `any` field.
    pub fn any<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let any = self
                .unwrap_non_null(Private)
                .as_ref()
                .any
                .load(Ordering::Relaxed);
            let any = NonNull::new(any.cast())?;
            Some(ValueRef::wrap(any).root(target))
        }
    }
}

impl_julia_typecheck!(TypeMapLevel<'scope>, jl_typemap_level_type, 'scope);
impl_debug!(TypeMapLevel<'_>);

impl<'scope> ManagedPriv<'scope, '_> for TypeMapLevel<'scope> {
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

impl_construct_type_managed!(TypeMapLevel, 1, jl_typemap_level_type);

/// A reference to a [`TypeMapLevel`] that has not been explicitly rooted.
pub type TypeMapLevelRef<'scope> = Ref<'scope, 'static, TypeMapLevel<'scope>>;

/// A [`TypeMapLevelRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`TypeMapLevel`].
pub type TypeMapLevelRet = Ref<'static, 'static, TypeMapLevel<'static>>;

impl_valid_layout!(TypeMapLevelRef, TypeMapLevel);

use crate::memory::target::target_type::TargetType;

/// `TypeMaLevely` or `TypeMaLevelyRef`, depending on the target type `T`.
pub type TypeMapLevelData<'target, T> =
    <T as TargetType<'target>>::Data<'static, TypeMapLevel<'target>>;

/// `JuliaResult<TypeMaLevely>` or `JuliaResultRef<TypeMapLevelRef>`, depending on the target type
/// `T`.
pub type TypeMapLevelResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, TypeMapLevel<'target>>;

impl_ccall_arg_managed!(TypeMapLevel, 1);
impl_into_typed!(TypeMapLevel);
