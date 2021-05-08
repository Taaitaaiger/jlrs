use super::{string::JuliaString, traits::wrapper::Wrapper};
use crate::value::{
    array::Array, code_instance::CodeInstance, datatype::DataType, expr::Expr, method::Method,
    method_instance::MethodInstance, method_match::MethodMatch, method_table::MethodTable,
    module::Module, simple_vector::SimpleVector, symbol::Symbol, task::Task, type_name::TypeName,
    type_var::TypeVar, typemap_entry::TypeMapEntry, typemap_level::TypeMapLevel, union::Union,
    union_all::UnionAll, weak_ref::WeakRef, Value,
};
use crate::{layout::valid_layout::ValidLayout, private::Private};
use std::{marker::PhantomData, ptr::null_mut};

/// A (possibly undefined or dangling) reference to Julia data.
///
/// When dealing with Julia data from Rust care must be taken that the garbage collector doesn't
/// free data that is still in use. Normally when you create Julia data with jlrs, the result is
/// returned as a `Value` which is explicitly rooted in a frame. These values can have fields,
/// the contents of each field are either stored inline or as a pointer. For example, a field that
/// contains a `UInt8` is stored inline, while a field that's untyped is stored as a pointer to
/// its contents. All wrapper types, such as `Array` and `Module` are generally stored as pointers
/// when they're used as fields. More information about the layout of Julia data can be found
/// [here].
///
/// When a field is stored as a pointer, the data that is pointed to is either valid Julia data
/// itself, or an undefined reference. An easy way to see this in action is through the `instance`
/// field of a `DataType`:
///
/// ```julia
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
/// its root as long as it's not an undefined reference, but there's one important issue that must
/// be taken into account: Julia data can be mutable and this can cause a `WrapperRef` that is in
/// use to become unreachable from any root. In general, a reference that was just acquired
/// through a root can be assumed to not have become unreachable and converted to its wrapper type
/// with [`WrapperRef::assume_reachable`] and to a `Value` with [`WrapperRef::assume_reachable_value`].
///
/// [here]: crate::value::traits::julia_struct
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct WrapperRef<'scope, 'data, T: Wrapper<'scope, 'data>>(
    *mut T::Internal,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

unsafe impl<'scope, 'data, T: Wrapper<'scope, 'data>> ValidLayout for WrapperRef<'scope, 'data, T> {
    unsafe fn valid_layout(ty: Value) -> bool {
        T::valid_layout(ty)
    }
}

/// A reference to a [`Value`]
pub type ValueRef<'scope, 'data> = WrapperRef<'scope, 'data, Value<'scope, 'data>>;

/// A reference to an [`Array`]
pub type ArrayRef<'scope, 'data> = WrapperRef<'scope, 'data, Array<'scope, 'data>>;

/// A reference to a [`CodeInstance`]
pub type CodeInstanceRef<'scope> = WrapperRef<'scope, 'static, CodeInstance<'scope>>;

/// A reference to a [`DataType`]
pub type DataTypeRef<'scope> = WrapperRef<'scope, 'static, DataType<'scope>>;

/// A reference to an [`Expr`]
pub type ExprRef<'scope> = WrapperRef<'scope, 'static, Expr<'scope>>;

/// A reference to a [`Method`]
pub type MethodRef<'scope> = WrapperRef<'scope, 'static, Method<'scope>>;

/// A reference to a [`MethodInstance`]
pub type MethodInstanceRef<'scope> = WrapperRef<'scope, 'static, MethodInstance<'scope>>;

/// A reference to a [`MethodMatch`]
pub type MethodMatchRef<'scope> = WrapperRef<'scope, 'static, MethodMatch<'scope>>;

/// A reference to a [`MethodTable`]
pub type MethodTableRef<'scope> = WrapperRef<'scope, 'static, MethodTable<'scope>>;

/// A reference to a [`Module`]
pub type ModuleRef<'scope> = WrapperRef<'scope, 'static, Module<'scope>>;

/// A reference to a [`SimpleVector`]
pub type SimpleVectorRef<'scope> = WrapperRef<'scope, 'static, SimpleVector<'scope>>;

/// A reference to a [`JuliaString`]
pub type StringRef<'scope> = WrapperRef<'scope, 'static, JuliaString<'scope>>;

/// A reference to a [`Symbol`]
pub type SymbolRef<'scope> = WrapperRef<'scope, 'static, Symbol<'scope>>;

/// A reference to a [`Task`]
pub type TaskRef<'scope> = WrapperRef<'scope, 'static, Task<'scope>>;

/// A reference to a [`TypeName`]
pub type TypeNameRef<'scope> = WrapperRef<'scope, 'static, TypeName<'scope>>;

/// A reference to a [`TypeVar`]
pub type TypeVarRef<'scope> = WrapperRef<'scope, 'static, TypeVar<'scope>>;

/// A reference to a [`TypeMapEntry`]
pub type TypeMapEntryRef<'scope> = WrapperRef<'scope, 'static, TypeMapEntry<'scope>>;

/// A reference to a [`TypeMapLevel`]
pub type TypeMapLevelRef<'scope> = WrapperRef<'scope, 'static, TypeMapLevel<'scope>>;

/// A reference to a [`Union`]
pub type UnionRef<'scope> = WrapperRef<'scope, 'static, Union<'scope>>;

/// A reference to a [`UnionAll`]
pub type UnionAllRef<'scope> = WrapperRef<'scope, 'static, UnionAll<'scope>>;

/// A reference to a [`WeakRef`]
pub type WeakRefRef<'scope> = WrapperRef<'scope, 'static, WeakRef<'scope>>;

impl<'scope, 'data, T: Wrapper<'scope, 'data>> WrapperRef<'scope, 'data, T> {
    pub(crate) unsafe fn wrap(ptr: *mut T::Internal) -> Self {
        WrapperRef(ptr, PhantomData, PhantomData)
    }

    /// An undefined reference.
    pub fn undefined_ref() -> WrapperRef<'scope, 'data, T> {
        WrapperRef(null_mut(), PhantomData, PhantomData)
    }

    /// Assume the reference still points to valid Julia data and convert it to the appropariate
    /// pointer type. Returns `None` if the reference is undefined.
    ///
    /// Safety: a reference is only valid as long as it's reachable through some rooted value.
    pub unsafe fn assume_reachable(self) -> Option<T> {
        if self.ptr().is_null() {
            return None;
        }

        T::assume_reachable(self, Private)
    }

    /// Assume the reference still points to valid Julia data and convert it to the appropariate
    /// pointer type. Returns `None` if the reference is undefined.
    ///
    /// Safety: this method does not check if the reference is undefined, a reference is only
    /// valid as long as it's reachable through some rooted value.
    pub unsafe fn assume_reachable_unchecked(self) -> T {
        T::assume_reachable_unchecked(self, Private)
    }

    /// Assume the reference still points to valid Julia data and convert it to a `Value`. Returns
    /// `None` if the reference is undefined.
    ///
    /// Safety: a reference is only valid as long as it's reachable through some rooted value.
    pub unsafe fn assume_reachable_value(self) -> Option<Value<'scope, 'data>> {
        if self.ptr().is_null() {
            return None;
        }

        T::assume_reachable_value(self, Private)
    }

    /// Assume the reference still points to valid Julia data and convert it to a `Value`.
    ///
    /// Safety: this method does not check if the reference is undefined, a reference is only
    /// valid as long as it's reachable through some rooted value.
    pub unsafe fn assume_reachable_value_unchecked(self) -> Value<'scope, 'data> {
        T::assume_reachable_value_unchecked(self, Private)
    }

    pub(crate) fn ptr(self) -> *mut T::Internal {
        self.0
    }
}
