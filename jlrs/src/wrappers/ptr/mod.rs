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

macro_rules! impl_root {
    ($type:tt, 2) => {
        impl<'target, 'value, 'data> $crate::wrappers::ptr::Root<'target, 'value, 'data>
            for $type<'value, 'data>
        {
            type Output = $type<'target, 'data>;
            unsafe fn root<S>(
                scope: S,
                value: $crate::wrappers::ptr::Ref<'value, 'data, Self>,
            ) -> $crate::error::JlrsResult<Self::Output>
            where
                S: $crate::memory::scope::PartialScope<'target>,
            {
                if let Some(v) = Self::wrapper(value, Private) {
                    let ptr = v.unwrap_non_null(Private);
                    scope.value(ptr, Private)
                } else {
                    Err($crate::error::JlrsError::UndefRef)?
                }
            }
        }
    };
    ($type:tt, 1) => {
        impl<'target, 'value> $crate::wrappers::ptr::Root<'target, 'value, 'static>
            for $type<'value>
        {
            type Output = $type<'target>;
            unsafe fn root<S>(
                scope: S,
                value: $crate::wrappers::ptr::Ref<'value, 'static, Self>,
            ) -> $crate::error::JlrsResult<Self::Output>
            where
                S: $crate::memory::scope::PartialScope<'target>,
            {
                if let Some(v) =
                    <Self as $crate::wrappers::ptr::private::Wrapper>::wrapper(value, Private)
                {
                    let ptr = v.unwrap_non_null(Private);
                    scope.value(ptr, Private)
                } else {
                    Err($crate::error::JlrsError::UndefRef)?
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
#[cfg(not(feature = "lts"))]
pub mod vararg;

#[cfg(not(feature = "lts"))]
use jl_sys::jl_value_t;

use self::{
    array::{Array, TypedArray},
    datatype::DataType,
    function::Function,
    module::Module,
    private::Wrapper as _,
    simple_vector::SimpleVector,
    string::JuliaString,
    symbol::Symbol,
    task::Task,
    type_name::TypeName,
    type_var::TypeVar,
    union::Union,
    union_all::UnionAll,
    value::Value,
};

#[cfg(feature = "internal-types")]
use self::internal::{
    code_instance::CodeInstance, expr::Expr, method::Method, method_instance::MethodInstance,
    method_match::MethodMatch, method_table::MethodTable, typemap_entry::TypeMapEntry,
    typemap_level::TypeMapLevel, weak_ref::WeakRef,
};

#[cfg(all(not(feature = "lts"), feature = "internal-types"))]
use self::internal::opaque_closure::OpaqueClosure;
#[cfg(not(feature = "lts"))]
use self::vararg::Vararg;
use crate::{
    call::Call,
    error::{JlrsError, JlrsResult, CANNOT_DISPLAY_VALUE},
    layout::valid_layout::ValidLayout,
    memory::{global::Global, scope::PartialScope},
    private::Private,
};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::null_mut,
    str::FromStr,
};

#[cfg(not(feature = "lts"))]
use std::sync::atomic::AtomicPtr;

macro_rules! impl_valid_layout {
    ($ref_type:ident, $type:ident) => {
        unsafe impl $crate::layout::valid_layout::ValidLayout for $ref_type<'_> {
            fn valid_layout(v: $crate::wrappers::ptr::value::Value) -> bool {
                if let Ok(dt) = v.cast::<$crate::wrappers::ptr::datatype::DataType>() {
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
            pub unsafe fn root<'target, S>(self, scope: S) -> JlrsResult<$type<'target, 'data>>
            where
                S: PartialScope<'target>,
            {
                <$type as Root>::root(scope, self)
            }
        }
    };
    ($type:tt, $reftype:tt, 1) => {
        impl<'scope> $reftype<'scope> {
            pub unsafe fn root<'target, S>(self, scope: S) -> JlrsResult<$type<'target>>
            where
                S: PartialScope<'target>,
            {
                <$type as Root>::root(scope, self)
            }
        }
    };
}

pub(crate) trait Root<'target, 'value, 'data>: Wrapper<'value, 'data> {
    type Output;
    unsafe fn root<S>(scope: S, value: Ref<'value, 'data, Self>) -> JlrsResult<Self::Output>
    where
        S: PartialScope<'target>;
}

/// Marker trait implemented by `Ref`.
pub trait WrapperRef<'scope, 'data>:
    private::WrapperRef<'scope, 'data> + Copy + Debug + ValidLayout
{
    type Wrapper: Wrapper<'scope, 'data>;
}

impl<'scope, 'data, T> WrapperRef<'scope, 'data> for Ref<'scope, 'data, T>
where
    T: Wrapper<'scope, 'data>,
    Self: ValidLayout,
{
    type Wrapper = T;
}

/// Methods shared by all builtin pointer wrappers.
pub trait Wrapper<'scope, 'data>: private::Wrapper<'scope, 'data> {
    /// The reference type associated with this wrapper.
    type Ref;

    /// Convert the wrapper to a `Ref`.
    fn as_ref(self) -> Self::Ref;

    /// Convert the wrapper to a `Value`.
    fn as_value(self) -> Value<'scope, 'data> {
        unsafe { Value::wrap_non_null(self.unwrap_non_null(Private).cast(), Private) }
    }

    /// Convert the wrapper to its display string, i.e. the string that is shown when calling
    /// `Base.show`.
    fn display_string(self) -> JlrsResult<String> {
        unsafe {
            let global = Global::new();
            let s = Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("valuestring")?
                .wrapper_unchecked()
                .call1_unrooted(global, self.as_value())
                .map_err(|e| JlrsError::Exception {
                    msg: format!(
                        "Jlrs.valuestring failed: {}",
                        e.value_unchecked().error_string_or(CANNOT_DISPLAY_VALUE)
                    ),
                })?
                .value_unchecked()
                .cast::<JuliaString>()?
                .as_str()?;

            let s = String::from_str(s).unwrap();

            Ok(s)
        }
    }

    /// Convert the wrapper to its error string, i.e. the string that is shown when calling
    /// `Base.showerror`. This string can contain ANSI color codes if this is enabled by calling
    /// [`Julia::error_color`], [`AsyncJulia::error_color`], or [`AsyncJulia::try_error_color`], .
    ///
    /// [`Julia::error_color`]: crate::runtime::sync_rt::Julia::error_color
    /// [`AsyncJulia::error_color`]: crate::runtime::async_rt::AsyncJulia::error_color
    /// [`AsyncJulia::try_error_color`]: crate::runtime::async_rt::AsyncJulia::try_error_color
    fn error_string(self) -> JlrsResult<String> {
        unsafe {
            let global = Global::new();
            let s = Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("errorstring")?
                .wrapper_unchecked()
                .call1_unrooted(global, self.as_value())
                .map_err(|e| JlrsError::Exception {
                    msg: format!(
                        "Jlrs.errorstring failed: {}",
                        e.value_unchecked().error_string_or(CANNOT_DISPLAY_VALUE)
                    ),
                })?
                .value_unchecked()
                .cast::<JuliaString>()?
                .as_str()?
                .to_string();

            Ok(s)
        }
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
    W: private::Wrapper<'scope, 'data>,
{
    type Ref = Ref<'scope, 'data, Self>;

    fn as_ref(self) -> Self::Ref {
        unsafe { Self::Ref::wrap(self.unwrap(Private)) }
    }
}

#[macro_export]
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

/// An unrooted reference to Julia data.
///
/// Pointer wrappers are generally guaranteed to wrap valid, rooted data. In some cases this
/// guarantee is too strong. The garbage collector uses the roots as a starting point to
/// determine what values can be reached, as long as you can guarantee a value is reachable it's
/// safe to use. Whenever data is not rooted jlrs returns a `Ref`. Because it's not rooted it's
/// unsafe to use.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Ref<'scope, 'data, T: Wrapper<'scope, 'data>>(
    *mut T::Wraps,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

impl<'scope, 'data, T: Wrapper<'scope, 'data>> Debug for Ref<'scope, 'data, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Ref<{}>", T::NAME)
    }
}

/// A reference to a [`Value`]
pub type ValueRef<'scope, 'data> = Ref<'scope, 'data, Value<'scope, 'data>>;

unsafe impl ValidLayout for ValueRef<'_, '_> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            !dt.is_inline_alloc()
        } else if v.cast::<UnionAll>().is_ok() {
            true
        } else if let Ok(u) = v.cast::<Union>() {
            !u.is_bits_union()
        } else {
            false
        }
    }

    const IS_REF: bool = true;
}

impl_ref_root!(Value, ValueRef, 2);

/// A reference to an [`Function`]
pub type FunctionRef<'scope, 'data> = Ref<'scope, 'data, Function<'scope, 'data>>;

unsafe impl ValidLayout for FunctionRef<'_, '_> {
    fn valid_layout(ty: Value) -> bool {
        let global = unsafe { Global::new() };
        let function_type = DataType::function_type(global);
        ty.subtype(function_type.as_value())
    }

    const IS_REF: bool = true;
}

impl_ref_root!(Function, FunctionRef, 2);

/// A reference to an [`Array`]
pub type ArrayRef<'scope, 'data> = Ref<'scope, 'data, Array<'scope, 'data>>;

unsafe impl ValidLayout for ArrayRef<'_, '_> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<Array>()
        } else if let Ok(ua) = v.cast::<UnionAll>() {
            unsafe { ua.base_type().wrapper_unchecked().is::<Array>() }
        } else {
            false
        }
    }

    const IS_REF: bool = true;
}

impl_ref_root!(Array, ArrayRef, 2);

/// A reference to an [`TypedArray`]
pub type TypedArrayRef<'scope, 'data, T> = Ref<'scope, 'data, TypedArray<'scope, 'data, T>>;

unsafe impl<T: Clone + ValidLayout + Debug> ValidLayout for TypedArrayRef<'_, '_, T> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<TypedArray<T>>()
        } else if let Ok(ua) = v.cast::<UnionAll>() {
            unsafe { ua.base_type().wrapper_unchecked().is::<TypedArray<T>>() }
        } else {
            false
        }
    }

    const IS_REF: bool = true;
}

impl<'scope, 'data, T> TypedArrayRef<'scope, 'data, T>
where
    T: Clone + ValidLayout + Debug,
{
    pub unsafe fn root<'target, S>(self, scope: S) -> JlrsResult<TypedArray<'target, 'data, T>>
    where
        S: PartialScope<'target>,
    {
        <TypedArray<T> as Root>::root(scope, self)
    }
}

/// A reference to a [`Module`]
pub type ModuleRef<'scope> = Ref<'scope, 'static, Module<'scope>>;
impl_valid_layout!(ModuleRef, Module);
impl_ref_root!(Module, ModuleRef, 1);

/// A reference to a [`DataType`]
pub type DataTypeRef<'scope> = Ref<'scope, 'static, DataType<'scope>>;
impl_valid_layout!(DataTypeRef, DataType);
impl_ref_root!(DataType, DataTypeRef, 1);

/// A reference to a [`JuliaString`]
pub type StringRef<'scope> = Ref<'scope, 'static, JuliaString<'scope>>;
impl_valid_layout!(StringRef, String);
impl_ref_root!(JuliaString, StringRef, 1);

/// A reference to a [`CodeInstance`]
#[cfg(feature = "internal-types")]
pub type CodeInstanceRef<'scope> = Ref<'scope, 'static, CodeInstance<'scope>>;
#[cfg(feature = "internal-types")]
impl_valid_layout!(CodeInstanceRef, CodeInstance);
#[cfg(feature = "internal-types")]
impl_ref_root!(CodeInstance, CodeInstanceRef, 1);

/// A reference to an [`Expr`]
#[cfg(feature = "internal-types")]
pub type ExprRef<'scope> = Ref<'scope, 'static, Expr<'scope>>;
#[cfg(feature = "internal-types")]
impl_valid_layout!(ExprRef, Expr);
#[cfg(feature = "internal-types")]
impl_ref_root!(Expr, ExprRef, 1);

/// A reference to a [`Method`]
#[cfg(feature = "internal-types")]
pub type MethodRef<'scope> = Ref<'scope, 'static, Method<'scope>>;
#[cfg(feature = "internal-types")]
impl_valid_layout!(MethodRef, Method);
#[cfg(feature = "internal-types")]
impl_ref_root!(Method, MethodRef, 1);

/// A reference to a [`MethodInstance`]
#[cfg(feature = "internal-types")]
pub type MethodInstanceRef<'scope> = Ref<'scope, 'static, MethodInstance<'scope>>;
#[cfg(feature = "internal-types")]
impl_valid_layout!(MethodInstanceRef, MethodInstance);
#[cfg(feature = "internal-types")]
impl_ref_root!(MethodInstance, MethodInstanceRef, 1);

/// A reference to a [`MethodMatch`]
#[cfg(feature = "internal-types")]
pub type MethodMatchRef<'scope> = Ref<'scope, 'static, MethodMatch<'scope>>;
#[cfg(feature = "internal-types")]
impl_valid_layout!(MethodMatchRef, MethodMatch);
#[cfg(feature = "internal-types")]
impl_ref_root!(MethodMatch, MethodMatchRef, 1);

/// A reference to a [`MethodTable`]
#[cfg(feature = "internal-types")]
pub type MethodTableRef<'scope> = Ref<'scope, 'static, MethodTable<'scope>>;
#[cfg(feature = "internal-types")]
impl_valid_layout!(MethodTableRef, MethodTable);
#[cfg(feature = "internal-types")]
impl_ref_root!(MethodTable, MethodTableRef, 1);

/// A reference to an [`OpaqueClosure`]
#[cfg(all(not(feature = "lts"), feature = "internal-types"))]
pub type OpaqueClosureRef<'scope> = Ref<'scope, 'static, OpaqueClosure<'scope>>;
#[cfg(all(not(feature = "lts"), feature = "internal-types"))]
impl_valid_layout!(OpaqueClosureRef, OpaqueClosure);
#[cfg(feature = "internal-types")]
impl_ref_root!(OpaqueClosure, OpaqueClosureRef, 1);

/// A reference to a [`SimpleVector`]
pub type SimpleVectorRef<'scope> = Ref<'scope, 'static, SimpleVector<'scope>>;

unsafe impl<'scope> ValidLayout for SimpleVectorRef<'scope> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<SimpleVector>()
        } else {
            false
        }
    }

    const IS_REF: bool = true;
}

impl<'scope> SimpleVectorRef<'scope> {
    pub unsafe fn root<'target, S>(self, scope: S) -> JlrsResult<SimpleVector<'target>>
    where
        S: PartialScope<'target>,
    {
        <SimpleVector as Root>::root(scope, self)
    }
}

/// A reference to a [`Symbol`]
pub type SymbolRef<'scope> = Ref<'scope, 'static, Symbol<'scope>>;
impl_valid_layout!(SymbolRef, Symbol);
impl_ref_root!(Symbol, SymbolRef, 1);

/// A reference to a [`Task`]
pub type TaskRef<'scope> = Ref<'scope, 'static, Task<'scope>>;
impl_valid_layout!(TaskRef, Task);
impl_ref_root!(Task, TaskRef, 1);

/// A reference to a [`TypeName`]
pub type TypeNameRef<'scope> = Ref<'scope, 'static, TypeName<'scope>>;
impl_valid_layout!(TypeNameRef, TypeName);
impl_ref_root!(TypeName, TypeNameRef, 1);

/// A reference to a [`TypeVar`]
pub type TypeVarRef<'scope> = Ref<'scope, 'static, TypeVar<'scope>>;
impl_valid_layout!(TypeVarRef, TypeVar);
impl_ref_root!(TypeVar, TypeVarRef, 1);

/// A reference to a [`TypeMapEntry`]
#[cfg(feature = "internal-types")]
pub type TypeMapEntryRef<'scope> = Ref<'scope, 'static, TypeMapEntry<'scope>>;
#[cfg(feature = "internal-types")]
impl_valid_layout!(TypeMapEntryRef, TypeMapEntry);
#[cfg(feature = "internal-types")]
impl_ref_root!(TypeMapEntry, TypeMapEntryRef, 1);

/// A reference to a [`TypeMapLevel`]
#[cfg(feature = "internal-types")]
pub type TypeMapLevelRef<'scope> = Ref<'scope, 'static, TypeMapLevel<'scope>>;
#[cfg(feature = "internal-types")]
impl_valid_layout!(TypeMapLevelRef, TypeMapLevel);
#[cfg(feature = "internal-types")]
impl_ref_root!(TypeMapLevel, TypeMapLevelRef, 1);

/// A reference to a [`Union`]
pub type UnionRef<'scope> = Ref<'scope, 'static, Union<'scope>>;
impl_valid_layout!(UnionRef, Union);
impl_ref_root!(Union, UnionRef, 1);

/// A reference to a [`UnionAll`]
pub type UnionAllRef<'scope> = Ref<'scope, 'static, UnionAll<'scope>>;
impl_valid_layout!(UnionAllRef, UnionAll);
impl_ref_root!(UnionAll, UnionAllRef, 1);

/// A reference to a [`Vararg`]
#[cfg(not(feature = "lts"))]
pub type VarargRef<'scope> = Ref<'scope, 'static, Vararg<'scope>>;
#[cfg(not(feature = "lts"))]
impl_valid_layout!(VarargRef, Vararg);
#[cfg(not(feature = "lts"))]
impl_ref_root!(Vararg, VarargRef, 1);

/// A reference to a [`WeakRef`]
#[cfg(feature = "internal-types")]
pub type WeakRefRef<'scope> = Ref<'scope, 'static, WeakRef<'scope>>;
#[cfg(feature = "internal-types")]
impl_valid_layout!(WeakRefRef, WeakRef);
#[cfg(feature = "internal-types")]
impl_ref_root!(WeakRef, WeakRefRef, 1);

impl<'scope, 'data, T: Wrapper<'scope, 'data>> Ref<'scope, 'data, T> {
    pub(crate) unsafe fn wrap(ptr: *mut T::Wraps) -> Self {
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
    /// Safety: a reference is only valid as long as it's reachable through some rooted value.
    /// It's the caller's responsibility to ensure the result is never used after it becomes
    /// unreachable.
    pub unsafe fn wrapper(self) -> Option<T> {
        T::wrapper(self, Private)
    }

    /// Assume the reference still points to valid Julia data and convert it to its wrapper type.
    ///
    /// Safety: this method doesn't  check if the reference is undefined, a reference is only
    /// valid as long as it's reachable through some rooted value.  It's the caller's
    /// responsibility to ensure the result is never used after it becomes unreachable.
    pub unsafe fn wrapper_unchecked(self) -> T {
        T::wrapper_unchecked(self, Private)
    }

    /// Assume the reference still points to valid Julia data and convert it to a `Value`. Returns
    /// `None` if the reference is undefined.
    ///
    /// Safety: a reference is only valid as long as it's reachable through some rooted value.
    /// It's the caller's responsibility to ensure the result is never used after it becomes
    /// unreachable.
    pub unsafe fn value(self) -> Option<Value<'scope, 'data>> {
        T::value(self, Private)
    }

    /// Assume the reference still points to valid Julia data and convert it to a `Value`.
    ///
    /// Safety: this method doesn't  check if the reference is undefined, a reference is only
    /// valid as long as it's reachable through some rooted value. It's the caller's
    /// responsibility to ensure the result is never used after it becomes unreachable.
    pub unsafe fn value_unchecked(self) -> Value<'scope, 'data> {
        T::value_unchecked(self, Private)
    }

    pub(crate) fn ptr(self) -> *mut T::Wraps {
        self.0
    }
}

pub(crate) mod private {
    use crate::private::Private;
    use crate::wrappers::ptr::{value::Value, Ref};
    use std::{fmt::Debug, ptr::NonNull};

    pub trait Wrapper<'scope, 'data>: Sized + Copy + Debug {
        type Wraps: Copy;
        const NAME: &'static str;

        unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self;

        #[inline(always)]
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
        unsafe fn wrapper_unchecked(value_ref: Ref<'scope, 'data, Self>, _: Private) -> Self
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            Self::wrap(value_ref.ptr(), Private)
        }

        #[inline(always)]
        unsafe fn cast(value: Value<'scope, 'data>, _: Private) -> Self {
            Self::wrap_non_null(value.unwrap_non_null(Private).cast(), Private)
        }

        #[inline(always)]
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
        T: Wrapper<'scope, 'data>
    {
    }
}

#[cfg(not(feature = "lts"))]
pub(crate) fn atomic_value(addr: u64) -> AtomicPtr<jl_value_t> {
    AtomicPtr::new(addr as usize as *mut _)
}
