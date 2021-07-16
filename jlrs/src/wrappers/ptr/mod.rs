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
//! while it can be used. For more information about rooting please see the documentation of the
//! [`memory`] module.
//!
//! [`memory`]: crate::memory

pub mod array;
pub mod call;
pub mod code_instance;
pub mod datatype;
pub mod expr;
pub mod function;
pub mod method;
pub mod method_instance;
pub mod method_match;
pub mod method_table;
pub mod module;
pub mod simple_vector;
pub mod string;
pub mod symbol;
pub mod task;
pub mod type_name;
pub mod type_var;
pub mod typemap_entry;
pub mod typemap_level;
pub mod union;
pub mod union_all;
pub mod value;
pub mod weak_ref;

use self::{
    array::{Array, TypedArray},
    call::Call,
    code_instance::CodeInstance,
    datatype::DataType,
    expr::Expr,
    function::Function,
    method::Method,
    method_instance::MethodInstance,
    method_match::MethodMatch,
    method_table::MethodTable,
    module::Module,
    private::Wrapper as _,
    simple_vector::SimpleVector,
    string::JuliaString,
    symbol::Symbol,
    task::Task,
    type_name::TypeName,
    type_var::TypeVar,
    typemap_entry::TypeMapEntry,
    typemap_level::TypeMapLevel,
    union::Union,
    union_all::UnionAll,
    value::Value,
    weak_ref::WeakRef,
};
use crate::{
    error::{JlrsError, JlrsResult},
    layout::valid_layout::ValidLayout,
    memory::{frame::Frame, global::Global, scope::Scope},
    private::Private,
};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::null_mut,
};

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
        }
    };
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

    /// Convert the wrapper to its display string, ie the string that is shown by calling
    /// `Base.display`.
    fn display_string(self) -> JlrsResult<String> {
        unsafe {
            let global = Global::new();
            let s = Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("displaystring")?
                .wrapper_unchecked()
                .call1_unrooted(global, self.as_value())
                .map_err(|e| JlrsError::Exception {
                    msg: format!("Jlrs.displaystring failed: {:?}", e.value_unchecked()),
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
}

/// A reference to an [`Function`]
pub type FunctionRef<'scope, 'data> = Ref<'scope, 'data, Function<'scope, 'data>>;

unsafe impl ValidLayout for FunctionRef<'_, '_> {
    fn valid_layout(ty: Value) -> bool {
        let global = unsafe { Global::new() };
        let function_type = DataType::function_type(global);
        ty.subtype(function_type.as_value())
    }
}

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
}

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
}

/// A reference to a [`Module`]
pub type ModuleRef<'scope> = Ref<'scope, 'static, Module<'scope>>;
impl_valid_layout!(ModuleRef, Module);

/// A reference to a [`DataType`]
pub type DataTypeRef<'scope> = Ref<'scope, 'static, DataType<'scope>>;
impl_valid_layout!(DataTypeRef, DataType);

/// A reference to a [`JuliaString`]
pub type StringRef<'scope> = Ref<'scope, 'static, JuliaString<'scope>>;
impl_valid_layout!(StringRef, String);

/// A reference to a [`CodeInstance`]
pub type CodeInstanceRef<'scope> = Ref<'scope, 'static, CodeInstance<'scope>>;
impl_valid_layout!(CodeInstanceRef, CodeInstance);

/// A reference to an [`Expr`]
pub type ExprRef<'scope> = Ref<'scope, 'static, Expr<'scope>>;
impl_valid_layout!(ExprRef, Expr);

/// A reference to a [`Method`]
pub type MethodRef<'scope> = Ref<'scope, 'static, Method<'scope>>;
impl_valid_layout!(MethodRef, Method);

/// A reference to a [`MethodInstance`]
pub type MethodInstanceRef<'scope> = Ref<'scope, 'static, MethodInstance<'scope>>;
impl_valid_layout!(MethodInstanceRef, MethodInstance);

/// A reference to a [`MethodMatch`]
pub type MethodMatchRef<'scope> = Ref<'scope, 'static, MethodMatch<'scope>>;
impl_valid_layout!(MethodMatchRef, MethodMatch);

/// A reference to a [`MethodTable`]
pub type MethodTableRef<'scope> = Ref<'scope, 'static, MethodTable<'scope>>;
impl_valid_layout!(MethodTableRef, MethodTable);

/// A reference to a [`SimpleVector`]
pub type SimpleVectorRef<'scope, T = Value<'scope, 'static>> =
    Ref<'scope, 'static, SimpleVector<'scope, T>>;

unsafe impl<'scope, T: Wrapper<'scope, 'static>> ValidLayout for SimpleVectorRef<'scope, T> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<SimpleVector>()

            // FIXME: Check if all elements are T
        } else {
            false
        }
    }
}

/// A reference to a [`Symbol`]
pub type SymbolRef<'scope> = Ref<'scope, 'static, Symbol<'scope>>;
impl_valid_layout!(SymbolRef, Symbol);

/// A reference to a [`Task`]
pub type TaskRef<'scope> = Ref<'scope, 'static, Task<'scope>>;
impl_valid_layout!(TaskRef, Task);

/// A reference to a [`TypeName`]
pub type TypeNameRef<'scope> = Ref<'scope, 'static, TypeName<'scope>>;
impl_valid_layout!(TypeNameRef, TypeName);

/// A reference to a [`TypeVar`]
pub type TypeVarRef<'scope> = Ref<'scope, 'static, TypeVar<'scope>>;
impl_valid_layout!(TypeVarRef, TypeVar);

/// A reference to a [`TypeMapEntry`]
pub type TypeMapEntryRef<'scope> = Ref<'scope, 'static, TypeMapEntry<'scope>>;
impl_valid_layout!(TypeMapEntryRef, TypeMapEntry);

/// A reference to a [`TypeMapLevel`]
pub type TypeMapLevelRef<'scope> = Ref<'scope, 'static, TypeMapLevel<'scope>>;
impl_valid_layout!(TypeMapLevelRef, TypeMapLevel);

/// A reference to a [`Union`]
pub type UnionRef<'scope> = Ref<'scope, 'static, Union<'scope>>;
impl_valid_layout!(UnionRef, Union);

/// A reference to a [`UnionAll`]
pub type UnionAllRef<'scope> = Ref<'scope, 'static, UnionAll<'scope>>;
impl_valid_layout!(UnionAllRef, UnionAll);

/// A reference to a [`WeakRef`]
pub type WeakRefRef<'scope> = Ref<'scope, 'static, WeakRef<'scope>>;
impl_valid_layout!(WeakRefRef, WeakRef);

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

    /// Assume the reference still points to valid Julia data and root it in `scope`. Returns an
    /// error if the reference is undefined.
    ///
    /// Safety: a reference is only valid as long as it's reachable through some rooted value.
    pub unsafe fn root<'target, 'current, S, F>(self, scope: S) -> JlrsResult<S::Value>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        if let Some(v) = T::value(self, Private) {
            let ptr = v.unwrap_non_null(Private);
            scope.value(ptr, Private)
        } else {
            Err(JlrsError::UndefRef)?
        }
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

        unsafe fn wrap(ptr: *mut Self::Wraps, _: Private) -> Self {
            debug_assert!(!ptr.is_null());
            Self::wrap_non_null(NonNull::new_unchecked(ptr), Private)
        }

        fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps>;

        fn unwrap(self, _: Private) -> *mut Self::Wraps {
            self.unwrap_non_null(Private).as_ptr()
        }

        unsafe fn wrapper_unchecked(value_ref: Ref<'scope, 'data, Self>, _: Private) -> Self
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            Self::wrap(value_ref.ptr(), Private)
        }

        unsafe fn cast(value: Value<'scope, 'data>, _: Private) -> Self {
            Self::wrap_non_null(value.unwrap_non_null(Private).cast(), Private)
        }

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

        unsafe fn value_unchecked(
            value_ref: Ref<'scope, 'data, Self>,
            _: Private,
        ) -> Value<'scope, 'data>
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            Value::wrap(value_ref.ptr().cast(), Private)
        }

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
}
