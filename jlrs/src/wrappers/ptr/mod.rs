//! Wrappers for builtin pointer types.
//!
//! Some of the most important types in Julia are builtin pointer types, these are types that are
//! defined in C rather than pure Julia. When such types are used as the field types of a struct
//! they're stored as a pointer rather than inline. Rather than dealing with raw pointers to data
//! to such types, jlrs provides wrappers with lifetimes that ensure they can only be used while
//! they're guaranteed to be reachable by the garbage collector. While there are more than twenty
//! of these wrapper types, most of them are available for completeness sake and can be ignored.
//!
//! All wrappers implement the [`Wrapper`] trait. The most important wrappers are
//! [`Value`], [`Module`] and [`Array`]. The first of these represents generic Julia
//! data. For example, when you call a Julia function its arguments must be `Value`s and it will
//! return one, too. A significant part of the Julia C API is available through methods
//! implemented by this type, including methods are available to create new values, access their
//! fields and convert them to other types. See the [`value`] module for more information.
//!
//! In order to even call a Julia function, though, we need to have one first. All Julia functions
//! are defined in some module, you will need to use [`Module`] which provides access to their
//! contents. Anything that implements [`Call`] or [`UnsafeCall`] can be called as a Julia
//! function.
//!
//! The [`Array`] wrapper lets you work with Julia's n-dimensional array type. This wrapper is
//! rarely returned directly. Rather, if a [`Value`] is an array, it can be cast to `Array` by
//! calling [`Value::cast`]. This method can generally be used to convert a `Value` to some
//! other wrapper if the value is of that type. Similarly, all these types can be converted back
//! to a `Value` by calling [`Wrapper::as_value`].
//!
//! Other wrappers you are likely to use are [`Function`], a wrapper for Julia functions;
//! [`DataType`], a wrapper for a value's type information; and [`JuliaString`], Julia's internal
//! string-type.
//!
//! Whenever jlrs returns a wrapper directly, it's guaranteed that the wrapper is rooted while it
//! can be used. Rooting data you're using isn't always necessary, though. For example, a function
//! defined in some module doesn't need to be rooted as long as you can guarantee that it's never
//! used after the module replaced. If you never replace the module, the function can safely be
//! used without rooting it. It's also possible that you want to call a function but don't care
//! about its return value (the function might always return `nothing`). Finally, any pointer
//! field of a value that can be reached through some root is itself reachable, so it doesn't need
//! to be rooted as long as you can guarantee the value won't become unreachable due to mutation.
//! The [`Ref`] struct and its aliases defined in this module are available for these
//! purposes.

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
pub mod traits;
pub mod type_name;
pub mod type_var;
pub mod typemap_entry;
pub mod typemap_level;
pub mod union;
pub mod union_all;
pub mod value;
pub mod weak_ref;

use self::{array::{Array, TypedArray}, call::UnsafeCall, code_instance::CodeInstance, datatype::DataType, expr::Expr, function::Function, method::Method, method_instance::MethodInstance, method_match::MethodMatch, method_table::MethodTable, module::Module, private::Wrapper as _, simple_vector::SimpleVector, string::JuliaString, symbol::Symbol, task::Task, type_name::TypeName, type_var::TypeVar, typemap_entry::TypeMapEntry, typemap_level::TypeMapLevel, union::Union, union_all::UnionAll, value::Value, weak_ref::WeakRef};
use crate::{
    error::{JlrsError, JlrsResult},
    layout::valid_layout::ValidLayout,
    memory::{frame::Frame, global::Global, scope::Scope},
    private::Private,
};
use std::{fmt::{Debug}, marker::PhantomData, ptr::null_mut};

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

/// Generic behavior shared by all wrappers.
pub trait Wrapper<'scope, 'data>: private::Wrapper<'scope, 'data>  {
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
                .unsafe_call1_unrooted(global, self.as_value())
                .map_err(|_| JlrsError::Exception("Jlrs.displaystring failed".into()))?
                .value_unchecked()
                .cast::<JuliaString>()?
                .as_str()?
                .to_string();

            Ok(s)
        }
    }
}

impl<'scope, 'data, W> Wrapper<'scope, 'data> for W where
    W: private::Wrapper<'scope, 'data>
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
                    Err(e) => f.write_fmt(format_args!("<Cannot display value: {}>", e))
                }
            }
        }
    };
}


/// A possibly undefined or dangling, unrooted reference to Julia data.
///
/// When dealing with Julia data from Rust care must be taken that the garbage collector doesn't
/// free data that is still in use. Normally when you create Julia data with jlrs, the result is
/// returned as a `Value` which is explicitly rooted in a frame. These values can have fields,
/// the contents of each field are either stored inline or as a pointer. For example, a field that
/// contains a `UInt8` is stored inline, while a field that's untyped is stored as a pointer to
/// its contents. All wrapper types, such as `Array` and `Module` are generally stored as pointers
/// when they're used as fields.
///
/// When a field is stored as a pointer, the data that is pointed to is either valid Julia data
/// itself, or an undefined reference. An easy way to see this in action is through the `instance`
/// field of a `DataType`:
///
/// ```ignore
/// julia> println(Nothing.instance)
/// nothing
///
/// julia> println(DataType.instance)
/// ERROR: UndefRefError: access to undefined reference
/// Stacktrace:
///  [1] getproperty(x::Type, f::Symbol)
///    @ Base ./Base.jl:28
///  [2] top-level scope
///    @ REPL[13]:1
/// ```
///
/// An undefined reference is stored as a null pointer, and as you can see in the example above
/// trying to use an undefined reference causes an error. If it does point to valid Julia data
/// and the parent is rooted, the garbage collector wil not free that data until the parent is
/// no longer rooted. Similarly, if the data it points to contains pointer fields itself those
/// will also be protected.
///
/// This means that any pointer field can normally be used as a `Value` with the same lifetimes as
/// its parent as long as it's not an undefined reference, but there's one important restriction
/// that must be taken into account: Julia data can be mutable and this can cause a `Ref` that is
/// in use to become unreachable from any root.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Ref<'scope, 'data, T: Wrapper<'scope, 'data>>(
    *mut T::Internal,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

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
    pub(crate) unsafe fn wrap(ptr: *mut T::Internal) -> Self {
        Ref(ptr, PhantomData, PhantomData)
    }

    /// An undefined reference.
    pub fn undefined_ref() -> Ref<'scope, 'data, T> {
        Ref(null_mut(), PhantomData, PhantomData)
    }

    /// Returns `true` if the reference is undefined.
    pub fn is_undefined(self) -> bool {
        self.0.is_null()
    }

    /// Assume the reference still points to valid Julia data and convert it to the appropariate
    /// pointer type. Returns `None` if the reference is undefined.
    ///
    /// Safety: a reference is only valid as long as it's reachable through some rooted value.
    /// It's the caller's responsibility to ensure the result is never used after it becomes
    /// unreachable.
    pub unsafe fn wrapper(self) -> Option<T> {
        T::wrapper(self, Private)
    }

    /// Assume the reference still points to valid Julia data and convert it to the appropariate
    /// pointer type. Returns `None` if the reference is undefined.
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

    pub(crate) fn ptr(self) -> *mut T::Internal {
        self.0
    }
}

pub(crate) mod private {
    use crate::private::Private;
    use crate::wrappers::ptr::{value::Value, Ref};
    use std::{fmt::Debug, ptr::NonNull};

    pub trait Wrapper<'scope, 'data>: Sized + Copy + Debug {
        type Internal: Copy;

        unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self;

        unsafe fn wrap(ptr: *mut Self::Internal, _: Private) -> Self {
            debug_assert!(!ptr.is_null());
            Self::wrap_non_null(NonNull::new_unchecked(ptr), Private)
        }

        unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal>;

        unsafe fn unwrap(self, _: Private) -> *mut Self::Internal {
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
