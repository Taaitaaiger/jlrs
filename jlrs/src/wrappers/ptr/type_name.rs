//! Wrapper for `TypeName`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L380

use super::{private::Wrapper, SymbolRef};
use crate::wrappers::ptr::{MethodTableRef, ModuleRef, SimpleVectorRef, ValueRef};
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout};
use crate::{memory::global::Global, private::Private};
use jl_sys::{
    jl_array_typename, jl_llvmpointer_typename, jl_namedtuple_typename, jl_pointer_typename,
    jl_tuple_typename, jl_type_typename, jl_typename_t, jl_typename_type, jl_vararg_typename,
    jl_vecelement_typename,
};
use std::{marker::PhantomData, ptr::NonNull};

/// Describes the syntactic structure of a type and stores all data common to different
/// instantiations of the type.
#[derive(Copy, Clone)]
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
    wrapper: Type
    cache: Core.SimpleVector
    linearcache: Core.SimpleVector
    hash: Int64
    mt: Core.MethodTable
    partial: Any
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
    pub fn names(self) -> SimpleVectorRef<'scope> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().names) }
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

    /// The `hash` field.
    pub fn hash(self) -> isize {
        unsafe { self.unwrap_non_null(Private).as_ref().hash }
    }

    /// The `mt` field.
    pub fn mt(self) -> MethodTableRef<'scope> {
        unsafe { MethodTableRef::wrap(self.unwrap_non_null(Private).as_ref().mt) }
    }

    /// Incomplete instantiations of this type.
    pub fn partial(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().partial.cast()) }
    }
}

impl<'base> TypeName<'base> {
    /// The typename of the `UnionAll` `VecElement`.
    pub fn vecelement_typename(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_vecelement_typename, Private) }
    }

    /// The typename of the `UnionAll` `Array`.
    pub fn array_typename(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_array_typename, Private) }
    }

    /// The typename of the `UnionAll` `Ptr`.
    pub fn pointer_typename(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_pointer_typename, Private) }
    }

    /// The typename of the `UnionAll` `LLVMPtr`.
    pub fn llvmpointer_typename(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_llvmpointer_typename, Private) }
    }

    /// The typename of the `UnionAll` `NamedTuple`.
    pub fn namedtuple_typename(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_namedtuple_typename, Private) }
    }

    /// The typename of the `UnionAll` `Vararg`.
    pub fn vararg_typename(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_vararg_typename, Private) }
    }

    /// The typename of the `UnionAll` `Type`.
    pub fn type_typename(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_type_typename, Private) }
    }

    /// The typename of the `DataType` `Tuple`.
    pub fn tuple_typename(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_tuple_typename, Private) }
    }
}

impl_julia_typecheck!(TypeName<'scope>, jl_typename_type, 'scope);
impl_debug!(TypeName<'_>);
impl_valid_layout!(TypeName<'scope>, 'scope);

impl<'scope> Wrapper<'scope, '_> for TypeName<'scope> {
    type Internal = jl_typename_t;
    const NAME: &'static str = "TypeName";

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
