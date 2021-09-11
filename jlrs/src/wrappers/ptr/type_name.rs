//! Wrapper for `TypeName`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L380

use super::{private::Wrapper, SymbolRef};
use crate::prelude::Symbol;
use crate::wrappers::ptr::{MethodTableRef, ModuleRef, SimpleVectorRef, ValueRef};
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout};
use crate::{memory::global::Global, private::Private};
use jl_sys::{
    jl_array_typename, jl_llvmpointer_typename, jl_namedtuple_typename, jl_opaque_closure_typename,
    jl_pointer_typename, jl_tuple_typename, jl_type_typename, jl_typename_t, jl_typename_type,
    jl_vecelement_typename,
};
use std::{marker::PhantomData, ptr::NonNull};

/// Describes the syntactic structure of a type and stores all data common to different
/// instantiations of the type.
#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct TypeName<'scope>(NonNull<jl_typename_t>, PhantomData<&'scope ()>);

impl<'scope> TypeName<'scope> {
    /*
    for (a, b) in zip(fieldnames(Core.TypeName), fieldtypes(Core.TypeName))
        println(a, ": ", b)
    end
    name: Symbol
    module: Module
    names: Core.SimpleVector
    atomicfields: Ptr{Nothing}
    wrapper: Type
    cache: Core.SimpleVector
    linearcache: Core.SimpleVector
    mt: Core.MethodTable
    partial: Any
    hash: Int64
    n_uninitialized: Int32
    flags: UInt8
    */

    /// The `name` field.
    pub fn name(self) -> SymbolRef<'scope> {
        unsafe { SymbolRef::wrap(self.unwrap_non_null(Private).as_ref().name) }
    }

    /// The `module` field.
    pub fn module(self) -> ModuleRef<'scope> {
        unsafe { ModuleRef::wrap(self.unwrap_non_null(Private).as_ref().module) }
    }

    /// Field names.
    pub fn names(self) -> SimpleVectorRef<'scope, Symbol<'scope>> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().names) }
    }

    /// The `atomicfields` field.
    pub fn atomicfields(self) -> *const u32 {
        unsafe { self.unwrap_non_null(Private).as_ref().atomicfields }
    }

    /// Either the only instantiation of the type (if no parameters) or a `UnionAll` accepting
    /// parameters to make an instantiation.
    pub fn wrapper(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().wrapper) }
    }

    /// Sorted array.
    pub fn cache(self) -> SimpleVectorRef<'scope> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().cache) }
    }

    /// Unsorted array.
    pub fn linear_cache(self) -> SimpleVectorRef<'scope> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().linearcache) }
    }

    /// The `mt` field.
    pub fn mt(self) -> MethodTableRef<'scope> {
        unsafe { MethodTableRef::wrap(self.unwrap_non_null(Private).as_ref().mt) }
    }

    /// Incomplete instantiations of this type.
    pub fn partial(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().partial.cast()) }
    }

    /// The `hash` field.
    pub fn hash(self) -> isize {
        unsafe { self.unwrap_non_null(Private).as_ref().hash }
    }

    /// The `n_uninitialized` field.
    pub fn n_uninitialized(self) -> i32 {
        unsafe { self.unwrap_non_null(Private).as_ref().n_uninitialized }
    }

    /// The `abstract` field.
    pub fn abstract_(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().abstract_() != 0 }
    }

    /// The `mutabl` field.
    pub fn mutabl(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().mutabl() != 0 }
    }

    /// The `mayinlinealloc` field.
    pub fn mayinlinealloc(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().mayinlinealloc() != 0 }
    }
}

impl<'base> TypeName<'base> {
    /// The typename of the `UnionAll` `Type`.
    pub fn of_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_type_typename, Private) }
    }

    /// The typename of the `DataType` `Tuple`.
    pub fn of_tuple(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_tuple_typename, Private) }
    }

    /// The typename of the `UnionAll` `VecElement`.
    pub fn of_vecelement(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_vecelement_typename, Private) }
    }

    /// The typename of the `UnionAll` `Array`.
    pub fn of_array(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_array_typename, Private) }
    }

    /// The typename of the `UnionAll` `Ptr`.
    pub fn of_opaque_closure(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_opaque_closure_typename, Private) }
    }

    /// The typename of the `UnionAll` `Ptr`.
    pub fn of_pointer(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_pointer_typename, Private) }
    }

    /// The typename of the `UnionAll` `LLVMPtr`.
    pub fn of_llvmpointer(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_llvmpointer_typename, Private) }
    }

    /// The typename of the `UnionAll` `NamedTuple`.
    pub fn of_namedtuple(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_namedtuple_typename, Private) }
    }
}

impl_julia_typecheck!(TypeName<'scope>, jl_typename_type, 'scope);
impl_debug!(TypeName<'_>);
impl_valid_layout!(TypeName<'scope>, 'scope);

impl<'scope> Wrapper<'scope, '_> for TypeName<'scope> {
    type Wraps = jl_typename_t;
    const NAME: &'static str = "TypeName";

    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}
