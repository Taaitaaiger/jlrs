//! Wrappers for builtin pointer types.
//!
//! In this module you'll find wrappers for all builtin pointer types. These are types like
//! [`Module`], [`DataType`], and [`Array`]. These types often provide access to some specific
//! functionality from the C API. For example, the [`Module`] wrapper provides access to the
//! contents of Julia modules, and the [`Array`] wrapper access to the contents of n-dimensional
//! Julia arrays.
//!
//! The most common of these wrappers is [`Value`], it represents some arbitrary data that Julia
//! can use. Whenever you call a Julia function its arguments must be of this type, and a new one
//! is returned. All pointer wrappers are valid [`Value`]s.
//!
//! One useful guarantee provided by wrappers is that they point to an existing value and are
//! rooted. If a wrapper is returned that isn't rooted, jlrs will return a [`Ref`]. Unlike a
//! wrapper a ref can be undefined, and since it's not rooted it's not guaranteed to remain valid
//! while it can be used. For more information about rooting see the documentation of the
//! [`memory`] module.
//!
//! [`memory`]: crate::memory
//! [`DataType`]: crate::wrappers::ptr::datatype::DataType
//! [`Array`]: crate::wrappers::ptr::array::Array

macro_rules! impl_valid_layout {
    ($ref_type:ident, $type:ident) => {
        unsafe impl $crate::layout::valid_layout::ValidLayout for $ref_type<'_> {
            fn valid_layout(ty: $crate::wrappers::ptr::value::Value) -> bool {
                if let Ok(dt) = ty.cast::<$crate::wrappers::ptr::datatype::DataType>() {
                    dt.is::<$type>()
                } else {
                    false
                }
            }

            const IS_REF: bool = true;
        }

        unsafe impl $crate::layout::valid_layout::ValidField for Option<$ref_type<'_>> {
            fn valid_field(ty: $crate::wrappers::ptr::value::Value) -> bool {
                if let Ok(dt) = ty.cast::<$crate::wrappers::ptr::datatype::DataType>() {
                    dt.is::<$type>()
                } else {
                    false
                }
            }
        }
    };
}

macro_rules! impl_ref_root {
    ($type:tt, $reftype:tt, 2) => {
        impl<'scope, 'data> $reftype<'scope, 'data> {
            /// Root this data in `scope`.
            ///
            /// Safety: The data pointed to by `self` must not have been freed by the GC yet.
            pub unsafe fn root<'target, T>(self, target: T) -> T::Data<'data, $type<'target, 'data>>
            where
                T: $crate::memory::target::Target<'target>,
            {
                target.data_from_ptr(self.ptr(), $crate::private::Private)
            }
        }
    };
    ($type:tt, $reftype:tt, 1) => {
        impl<'scope> $reftype<'scope> {
            /// Root this data in `scope`.
            ///
            /// Safety: The data pointed to by `self` must not have been freed by the GC yet.
            pub unsafe fn root<'target, T>(self, target: T) -> T::Data<'static, $type<'target>>
            where
                T: $crate::memory::target::Target<'target>,
            {
                target.data_from_ptr(self.ptr(), $crate::private::Private)
            }
        }
    };
}

macro_rules! impl_debug {
    ($type:ty) => {
        impl ::std::fmt::Debug for $type {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match <Self as $crate::wrappers::ptr::Wrapper>::display_string(*self) {
                    Ok(s) => f.write_str(&s),
                    Err(e) => f.write_fmt(format_args!("<Cannot display value: {}>", e)),
                }
            }
        }
    };
}

pub mod array;
pub mod datatype;
pub mod function;
#[cfg(feature = "internal-types")]
pub mod internal;
pub mod module;
pub mod simple_vector;
pub mod string;
pub mod symbol;
pub mod task;
pub mod type_name;
pub mod type_var;
pub mod union;
pub mod union_all;
pub mod value;

use crate::{
    call::Call,
    error::{JlrsError, JlrsResult, CANNOT_DISPLAY_VALUE},
    layout::valid_layout::{ValidField, ValidLayout},
    memory::target::global::Global,
    prelude::Target,
    private::Private,
    wrappers::ptr::{module::Module, private::WrapperPriv as _, string::JuliaString, value::Value},
};
use std::{
    ffi::c_void,
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// Trait implemented by `Ref`.
pub trait WrapperRef<'scope, 'data>:
    private::WrapperRef<'scope, 'data> + Copy + Debug + ValidLayout
{
    /// The pointer wrapper type associated with this `Ref`.
    type Wrapper: Wrapper<'scope, 'data>;
}

impl<'scope, 'data, T> WrapperRef<'scope, 'data> for Ref<'scope, 'data, T>
where
    T: Wrapper<'scope, 'data>,
    Self: Copy + ValidLayout,
    Option<Self>: ValidField,
{
    type Wrapper = T;
}

/// Trait implemented by all pointer wrapper types.
pub trait Wrapper<'scope, 'data>: private::WrapperPriv<'scope, 'data> {
    /// `Self`, but with arbitrary lifetimes. Used to construct the appropriate type in generic
    /// contexts.
    type TypeConstructor<'target, 'da>: Wrapper<'target, 'da>;

    /// Convert the wrapper to a `Ref`.
    fn as_ref(self) -> Ref<'scope, 'data, Self> {
        Ref::wrap(self.unwrap_non_null(Private))
    }

    /// Convert the wrapper to a `Value`.
    fn as_value(self) -> Value<'scope, 'data> {
        // Safety: Pointer wrappers can always be converted to a Value
        unsafe { Value::wrap_non_null(self.unwrap_non_null(Private).cast(), Private) }
    }

    /// Use the target to reroot this data.
    fn root<'target, T>(self, target: T) -> T::Data<'data, Self::TypeConstructor<'target, 'data>>
    where
        T: Target<'target>,
    {
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private).cast(), Private) }
    }

    /// Convert the wrapper to its display string, i.e. the string that is shown when calling
    /// `Base.show`.
    fn display_string(self) -> JlrsResult<String> {
        // Safety: all Julia data that is accessed is globally rooted, the result is converted
        // to a String before the GC can free it.
        let s = unsafe {
            let global = Global::new();
            Module::main(&global)
                .submodule(&global, "Jlrs")?
                .wrapper()
                .function(&global, "valuestring")?
                .wrapper()
                .call1(&global, self.as_value())
                .map_err(|e| e.value().error_string_or(CANNOT_DISPLAY_VALUE))
                .map_err(|e| JlrsError::exception(format!("Jlrs.valuestring failed: {}", e)))?
                .value()
                .cast::<JuliaString>()?
                .as_str()?
                .to_string()
        };

        Ok(s)
    }

    /// Convert the wrapper to its error string, i.e. the string that is shown when calling
    /// `Base.showerror`. This string can contain ANSI color codes if this is enabled by calling
    /// [`Julia::error_color`], [`AsyncJulia::error_color`], or [`AsyncJulia::try_error_color`], .
    ///
    /// [`Julia::error_color`]: crate::runtime::sync_rt::Julia::error_color
    /// [`AsyncJulia::error_color`]: crate::runtime::async_rt::AsyncJulia::error_color
    /// [`AsyncJulia::try_error_color`]: crate::runtime::async_rt::AsyncJulia::try_error_color
    fn error_string(self) -> JlrsResult<String> {
        // Safety: all Julia data that is accessed is globally rooted, the result is converted
        // to a String before the GC can free it.
        let s = unsafe {
            let global = Global::new();
            Module::main(&global)
                .submodule(&global, "Jlrs")?
                .wrapper()
                .function(&global, "errorstring")?
                .wrapper()
                .call1(&global, self.as_value())
                .map_err(|e| e.value().error_string_or(CANNOT_DISPLAY_VALUE))
                .map_err(|e| JlrsError::exception(format!("Jlrs.errorstring failed: {}", e)))?
                .value()
                .cast::<JuliaString>()?
                .as_str()?
                .to_string()
        };

        Ok(s)
    }

    /// Convert the wrapper to its display string, i.e. the string that is shown by calling
    /// `Base.display`, or some default value.
    fn display_string_or<S: Into<String>>(self, default: S) -> String {
        self.display_string().unwrap_or(default.into())
    }

    /// Convert the wrapper to its error string, i.e. the string that is shown when this value is
    /// thrown as an exception, or some default value.
    fn error_string_or<S: Into<String>>(self, default: S) -> String {
        self.error_string().unwrap_or(default.into())
    }
}

impl<'scope, 'data, W> Wrapper<'scope, 'data> for W
where
    W: private::WrapperPriv<'scope, 'data>,
{
    type TypeConstructor<'target, 'da> = Self::TypeConstructorPriv<'target, 'da>;
}

/// A reference to Julia data that is not guaranteed to be rooted.
///
/// Pointer wrappers are generally guaranteed to wrap valid, rooted data. In some cases this
/// guarantee is too strong. The garbage collector uses the roots as a starting point to
/// determine what values can be reached, as long as you can guarantee a value is reachable it's
/// safe to use. Whenever data is not rooted jlrs returns a `Ref`. Because it's not rooted it's
/// unsafe to use.
#[repr(transparent)]
pub struct Ref<'scope, 'data, T: Wrapper<'scope, 'data>>(
    NonNull<T::Wraps>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

impl<'scope, 'data, T> Clone for Ref<'scope, 'data, T>
where
    T: Wrapper<'scope, 'data>,
{
    fn clone(&self) -> Self {
        Ref(self.0, PhantomData, PhantomData)
    }
}

impl<'scope, 'data, T> Copy for Ref<'scope, 'data, T> where T: Wrapper<'scope, 'data> {}

impl<'scope, 'data, T: Wrapper<'scope, 'data>> Debug for Ref<'scope, 'data, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Ref<{}>", T::NAME)
    }
}

impl<'scope, 'data, T: Wrapper<'scope, 'data>> Ref<'scope, 'data, T> {
    pub(crate) fn wrap(ptr: NonNull<T::Wraps>) -> Self {
        Ref(ptr, PhantomData, PhantomData)
    }

    /// Assume the reference still points to valid Julia data and convert it to its wrapper type.
    ///
    /// Safety: a reference is only guaranteed to be valid as long as it's reachable from some
    /// GC root. If the reference is unreachable, the GC can free it. The GC can run whenever a
    /// safepoint is reached, this is typically the case when new Julia data is allocated.
    pub unsafe fn wrapper(self) -> T {
        T::wrapper(self, Private)
    }

    /// Assume the reference still points to valid Julia data and convert it to a `Value`.
    ///
    /// Safety: a reference is only guaranteed to be valid as long as it's reachable from some
    /// GC root. If the reference is unreachable, the GC can free it. The GC can run whenever a
    /// safepoint is reached, this is typically the case when new Julia data is allocated.
    pub unsafe fn value(self) -> Value<'scope, 'data> {
        T::value(self, Private)
    }

    /// Leaks `self` with a `'static` lifetime. This method is only available when the `ccall`
    /// feature is enabled.
    ///
    /// This method erases the `'scope` lifetime, the `'data` lifetime is not erased.
    ///
    /// Safety: this must only be used to return freshly allocated Julia data from Rust to Julia
    /// from a `ccall`ed function.
    #[cfg(feature = "ccall")]
    pub unsafe fn leak(self) -> Ref<'static, 'data, T::TypeConstructor<'static, 'data>> {
        Ref::wrap(self.ptr().cast())
    }

    /// Returns a pointer to the data,
    pub fn data_ptr(self) -> NonNull<c_void> {
        self.ptr().cast()
    }

    pub(crate) fn ptr(self) -> NonNull<T::Wraps> {
        self.0
    }
}

pub(crate) mod private {
    use crate::private::Private;
    use crate::wrappers::ptr::{value::Value, Ref};
    use std::{fmt::Debug, ptr::NonNull};

    pub trait WrapperPriv<'scope, 'data>: Copy + Debug {
        type Wraps;
        type TypeConstructorPriv<'target, 'da>: WrapperPriv<'target, 'da>;
        const NAME: &'static str;

        // Safety: `inner` must point to valid data. If it is not
        // rooted, it must never be used after becoming unreachable.
        unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self;

        #[inline(always)]
        // Safety: `Self` must be the correct type for `value`.
        unsafe fn from_value_unchecked(value: Value<'scope, 'data>, _: Private) -> Self {
            Self::wrap_non_null(value.unwrap_non_null(Private).cast(), Private)
        }

        fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps>;

        #[inline(always)]
        fn unwrap(self, _: Private) -> *mut Self::Wraps {
            self.unwrap_non_null(Private).as_ptr()
        }

        #[inline(always)]
        // Safety: value_ref must not have been freed yet and not be undefined, the wrapper can't
        // be used after the data becomes unreachable.
        unsafe fn wrapper(value_ref: Ref<'scope, 'data, Self>, _: Private) -> Self
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            Self::wrap_non_null(value_ref.ptr(), Private)
        }

        #[inline(always)]
        // Safety: value_ref must not have been freed yet, the wrapper can't
        // be used after the data becomes unreachable.
        unsafe fn value(value_ref: Ref<'scope, 'data, Self>, _: Private) -> Value<'scope, 'data>
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            Value::wrap_non_null(value_ref.ptr().cast(), Private)
        }
    }

    pub trait WrapperRef<'scope, 'data> {}

    impl<'scope, 'data, T> WrapperRef<'scope, 'data> for Ref<'scope, 'data, T> where
        T: WrapperPriv<'scope, 'data>
    {
    }
}
