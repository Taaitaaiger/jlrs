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

macro_rules! impl_root {
    ($type:tt, 2) => {
        impl<'target, 'value, 'data> $crate::wrappers::ptr::Root<'target, 'value, 'data>
            for $type<'value, 'data>
        {
            type Output = $type<'target, 'data>;
            unsafe fn root<T>(
                target: T,
                value: $crate::wrappers::ptr::Ref<'value, 'data, Self>,
            ) -> $crate::error::JlrsResult<T::Data>
            where
                T: $crate::memory::target::Target<'target, 'data, Self::Output>,
            {
                if let Some(v) = Self::wrapper(value, Private) {
                    let ptr = v.unwrap_non_null(Private);
                    Ok(target.data_from_ptr(ptr, Private))
                } else {
                    Err($crate::error::AccessError::UndefRef)?
                }
            }
        }
    };
    ($type:tt, 1) => {
        impl<'target, 'value> $crate::wrappers::ptr::Root<'target, 'value, 'static>
            for $type<'value>
        {
            type Output = $type<'target>;
            unsafe fn root<T>(
                target: T,
                value: $crate::wrappers::ptr::Ref<'value, 'static, Self>,
            ) -> $crate::error::JlrsResult<T::Data>
            where
                T: $crate::memory::target::Target<'target, 'static, Self::Output>,
            {
                if let Some(v) =
                    <Self as $crate::wrappers::ptr::private::WrapperPriv>::wrapper(value, Private)
                {
                    let ptr = v.unwrap_non_null(Private);
                    Ok(target.data_from_ptr(ptr, Private))
                } else {
                    Err($crate::error::AccessError::UndefRef)?
                }
            }
        }
    };
}

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
    };
}

macro_rules! impl_ref_root {
    ($type:tt, $reftype:tt, 2) => {
        impl<'scope, 'data> $reftype<'scope, 'data> {
            /// Root this data in `scope`.
            ///
            /// Safety: The data pointed to by `self` must not have been freed by the GC yet.
            pub unsafe fn root<'target, T>(self, target: T) -> $crate::error::JlrsResult<T::Data>
            where
                T: $crate::memory::target::Target<'target, 'data, $type<'target, 'data>>,
            {
                <$type as $crate::wrappers::ptr::Root>::root(target, self)
            }
        }
    };
    ($type:tt, $reftype:tt, 1) => {
        impl<'scope> $reftype<'scope> {
            /// Root this data in `scope`.
            ///
            /// Safety: The data pointed to by `self` must not have been freed by the GC yet.
            pub unsafe fn root<'target, T>(self, target: T) -> $crate::error::JlrsResult<T::Data>
            where
                T: $crate::memory::target::Target<'target, 'static, $type<'target>>,
            {
                <$type as $crate::wrappers::ptr::Root>::root(target, self)
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
    layout::valid_layout::ValidLayout,
    memory::{target::global::Global, target::Target},
    private::Private,
    wrappers::ptr::{module::Module, private::WrapperPriv as _, string::JuliaString, value::Value},
};
use std::{
    ffi::c_void,
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::null_mut,
};

pub(crate) trait Root<'target, 'value, 'data>: Wrapper<'value, 'data> {
    type Output: Wrapper<'target, 'data>;
    // Safety: `value` must point to valid Julia data.
    unsafe fn root<T>(target: T, value: Ref<'value, 'data, Self>) -> JlrsResult<T::Data>
    where
        T: Target<'target, 'data, Self::Output>;
}

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
    Self: ValidLayout + Copy,
{
    type Wrapper = T;
}

/// Trait implemented by all pointer wrapper types.
pub trait Wrapper<'scope, 'data>: private::WrapperPriv<'scope, 'data> {
    /// `Self`, but with the `'scope` lifetime replaced with the `'static` lifetime.
    type Static: Wrapper<'static, 'data>;

    /// Convert the wrapper to a `Ref`.
    fn as_ref(self) -> Ref<'scope, 'data, Self> {
        Ref::wrap(self.unwrap(Private))
    }

    /// Convert the wrapper to a `Value`.
    fn as_value(self) -> Value<'scope, 'data> {
        // Safety: Pointer wrappers can always be converted to a Value
        unsafe { Value::wrap_non_null(self.unwrap_non_null(Private).cast(), Private) }
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
                .wrapper_unchecked()
                .function(&global, "valuestring")?
                .wrapper_unchecked()
                .call1(&global, self.as_value())
                .map_err(|e| e.value_unchecked().error_string_or(CANNOT_DISPLAY_VALUE))
                .map_err(|e| JlrsError::exception(format!("Jlrs.valuestring failed: {}", e)))?
                .value_unchecked()
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
                .wrapper_unchecked()
                .function(&global, "errorstring")?
                .wrapper_unchecked()
                .call1(&global, self.as_value())
                .map_err(|e| e.value_unchecked().error_string_or(CANNOT_DISPLAY_VALUE))
                .map_err(|e| JlrsError::exception(format!("Jlrs.errorstring failed: {}", e)))?
                .value_unchecked()
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
    type Static = Self::StaticPriv;
}

/// An reference to Julia data that is not guaranteed to be rooted.
///
/// Pointer wrappers are generally guaranteed to wrap valid, rooted data. In some cases this
/// guarantee is too strong. The garbage collector uses the roots as a starting point to
/// determine what values can be reached, as long as you can guarantee a value is reachable it's
/// safe to use. Whenever data is not rooted jlrs returns a `Ref`. Because it's not rooted it's
/// unsafe to use.
#[repr(transparent)]
pub struct Ref<'scope, 'data, T: Wrapper<'scope, 'data>>(
    *mut T::Wraps,
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
    pub(crate) fn wrap(ptr: *mut T::Wraps) -> Self {
        Ref(ptr, PhantomData, PhantomData)
    }

    /// An undefined reference, i.e. a null pointer.
    pub fn undefined_ref() -> Ref<'scope, 'data, T> {
        Ref(null_mut(), PhantomData, PhantomData)
    }

    /// Returns `true` if the reference is undefined.
    pub fn is_undefined(self) -> bool {
        self.0.is_null()
    }

    /// Assume the reference still points to valid Julia data and convert it to its wrapper type.
    /// Returns `None` if the reference is undefined.
    ///
    /// Safety: a reference is only guaranteed to be valid as long as it's reachable from some
    /// GC root. If the reference is unreachable, the GC can free it. The GC can run whenever a
    /// safepoint is reached, this is generally the case when new Julia data is allocated.
    pub unsafe fn wrapper(self) -> Option<T> {
        T::wrapper(self, Private)
    }

    /// Assume the reference still points to valid Julia data and convert it to its wrapper type.
    ///
    /// Safety: this method doesn't check if the reference is undefined. A reference is only
    /// guaranteed to be valid as long as it's reachable from some GC root. If the reference is
    /// unreachable, the GC can free it. The GC can run whenever a safepoint is reached, this is
    /// generally the case when new Julia data is allocated.
    pub unsafe fn wrapper_unchecked(self) -> T {
        T::wrapper_unchecked(self, Private)
    }

    /// Assume the reference still points to valid Julia data and convert it to a `Value`. Returns
    /// `None` if the reference is undefined.
    ///
    /// Safety: a reference is only guaranteed to be valid as long as it's reachable from some
    /// GC root. If the reference is unreachable, the GC can free it. The GC can run whenever a
    /// safepoint is reached, this is generally the case when new Julia data is allocated.
    pub unsafe fn value(self) -> Option<Value<'scope, 'data>> {
        T::value(self, Private)
    }

    /// Assume the reference still points to valid Julia data and convert it to its wrapper type.
    ///
    /// Safety: this method doesn't check if the reference is undefined. A reference is only
    /// guaranteed to be valid as long as it's reachable from some GC root. If the reference is
    /// unreachable, the GC can free it. The GC can run whenever a safepoint is reached, this is
    /// generally the case when new Julia data is allocated.
    pub unsafe fn value_unchecked(self) -> Value<'scope, 'data> {
        T::value_unchecked(self, Private)
    }

    /// Leaks `self` with a `'static` lifetime. This method is only available when the `ccall`
    /// feature is enabled.
    ///
    /// This method erases the `'scope` lifetime, the `'data` lifetime is not erased.
    ///
    /// Safety: this must only be used to return freshly allocated Julia data from Rust to Julia
    /// from a `ccall`ed function.
    #[cfg(feature = "ccall")]
    pub unsafe fn leak(self) -> Ref<'static, 'data, T::Static> {
        Ref::wrap(self.ptr().cast())
    }

    /// Returns a pointer to the data,
    pub fn data_ptr(self) -> *mut c_void {
        self.0.cast()
    }

    pub(crate) fn ptr(self) -> *mut T::Wraps {
        self.0
    }
}

pub(crate) mod private {
    use crate::private::Private;
    use crate::wrappers::ptr::{value::Value, Ref};
    use std::{fmt::Debug, ptr::NonNull};

    pub trait WrapperPriv<'scope, 'data>: Copy + Debug {
        type Wraps;
        type StaticPriv: WrapperPriv<'static, 'data>;
        const NAME: &'static str;

        // Safety: `inner` must point to valid data. If it is not
        // rooted, it must never be used after becoming unreachable.
        unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self;

        // Safety: `ptr` must point to valid data. If it is not
        // rooted, it must never be used after becoming unreachable.
        unsafe fn wrap(ptr: *mut Self::Wraps, _: Private) -> Self {
            debug_assert!(!ptr.is_null());
            Self::wrap_non_null(NonNull::new_unchecked(ptr), Private)
        }

        fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps>;

        #[inline(always)]
        fn unwrap(self, _: Private) -> *mut Self::Wraps {
            self.unwrap_non_null(Private).as_ptr()
        }

        #[inline(always)]
        // Safety: value_ref must not have been freed yet and not be undefined, the wrapper can't
        // be used after the data becomes unreachable.
        unsafe fn wrapper_unchecked(value_ref: Ref<'scope, 'data, Self>, _: Private) -> Self
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            Self::wrap(value_ref.ptr(), Private)
        }

        #[inline(always)]
        // Safety: `Self` must be the correct type for `value`, and must not be undefined.
        unsafe fn cast(value: Value<'scope, 'data>, _: Private) -> Self {
            Self::wrap_non_null(value.unwrap_non_null(Private).cast(), Private)
        }

        #[inline(always)]
        // Safety: value_ref must not have been freed yet, the wrapper can't
        // be used after the data becomes unreachable.
        unsafe fn wrapper(value_ref: Ref<'scope, 'data, Self>, _: Private) -> Option<Self>
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            let ptr = value_ref.ptr();
            if ptr.is_null() {
                return None;
            }

            Some(Self::wrap(ptr, Private))
        }

        #[inline(always)]
        // Safety: value_ref must not have been freed yet and not be undefined, the wrapper can't
        // be used after the data becomes unreachable.
        unsafe fn value_unchecked(
            value_ref: Ref<'scope, 'data, Self>,
            _: Private,
        ) -> Value<'scope, 'data>
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            Value::wrap(value_ref.ptr().cast(), Private)
        }

        #[inline(always)]
        // Safety: value_ref must not have been freed yet, the wrapper can't
        // be used after the data becomes unreachable.
        unsafe fn value(
            value_ref: Ref<'scope, 'data, Self>,
            _: Private,
        ) -> Option<Value<'scope, 'data>>
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            let ptr = value_ref.ptr();
            if ptr.is_null() {
                return None;
            }

            Some(Value::wrap(ptr.cast(), Private))
        }
    }

    pub trait WrapperRef<'scope, 'data> {}

    impl<'scope, 'data, T> WrapperRef<'scope, 'data> for Ref<'scope, 'data, T> where
        T: WrapperPriv<'scope, 'data>
    {
    }
}
