//! Wrapper for `TypeName`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L380

use crate::{
    impl_debug, impl_julia_typecheck,
    memory::{global::Global, output::Output},
    private::Private,
    wrappers::ptr::{
        private::Wrapper as WrapperPriv, ModuleRef, SimpleVectorRef, SymbolRef, ValueRef,
    },
};
use cfg_if::cfg_if;
use jl_sys::{
    jl_array_typename, jl_llvmpointer_typename, jl_namedtuple_typename, jl_pointer_typename,
    jl_tuple_typename, jl_type_typename, jl_typename_t, jl_typename_type, jl_vecelement_typename,
};
use std::{marker::PhantomData, ptr::NonNull};

cfg_if! {
    if #[cfg(feature = "lts")] {
        use jl_sys::jl_vararg_typename;

    } else {
        use jl_sys::jl_opaque_closure_typename;
        use super::atomic_value;
        use std::sync::atomic::Ordering;
    }
}

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
    cache: Core.SimpleVector _Atomic
    linearcache: Core.SimpleVector _Atomic
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
    pub fn names(self) -> SimpleVectorRef<'scope> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().names) }
    }

    /// The `atomicfields` field.
    #[cfg(not(feature = "lts"))]
    pub fn atomicfields(self) -> *const u32 {
        unsafe { self.unwrap_non_null(Private).as_ref().atomicfields }
    }

    /// The `atomicfields` field.
    #[cfg(not(feature = "lts"))]
    pub fn constfields(self) -> *const u32 {
        unsafe { self.unwrap_non_null(Private).as_ref().constfields }
    }

    /// Either the only instantiation of the type (if no parameters) or a `UnionAll` accepting
    /// parameters to make an instantiation.
    pub fn wrapper(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().wrapper) }
    }

    /// Sorted array.
    #[cfg(feature = "lts")]
    pub fn cache(self) -> SimpleVectorRef<'scope> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().cache) }
    }

    /// Sorted array.
    #[cfg(not(feature = "lts"))]
    pub fn cache(self) -> SimpleVectorRef<'scope> {
        unsafe {
            let cache = atomic_value(&mut self.unwrap_non_null(Private).as_mut().cache as *mut _);
            let ptr = cache.load(Ordering::Relaxed);
            SimpleVectorRef::wrap(ptr.cast())
        }
    }

    /// Unsorted array.
    #[cfg(feature = "lts")]
    pub fn linear_cache(self) -> SimpleVectorRef<'scope> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().linearcache) }
    }

    /// Unsorted array.
    #[cfg(not(feature = "lts"))]
    pub fn linear_cache(self) -> SimpleVectorRef<'scope> {
        unsafe {
            let linearcache =
                atomic_value(&mut self.unwrap_non_null(Private).as_mut().linearcache as *mut _);
            let ptr = linearcache.load(Ordering::Relaxed);
            SimpleVectorRef::wrap(ptr.cast())
        }
    }

    /// The `mt` field.
    pub fn mt(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().mt.cast()) }
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
    #[cfg(not(feature = "lts"))]
    pub fn n_uninitialized(self) -> i32 {
        unsafe { self.unwrap_non_null(Private).as_ref().n_uninitialized }
    }

    /// The `abstract` field.
    #[cfg(not(feature = "lts"))]
    pub fn abstract_(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().abstract_() != 0 }
    }

    /// The `mutabl` field.
    #[cfg(not(feature = "lts"))]
    pub fn mutabl(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().mutabl() != 0 }
    }

    /// The `mayinlinealloc` field.
    #[cfg(not(feature = "lts"))]
    pub fn mayinlinealloc(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().mayinlinealloc() != 0 }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> TypeName<'target> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<TypeName>(ptr);
            TypeName::wrap_non_null(ptr, Private)
        }
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

    /// The typename of the `UnionAll` `Vararg`.
    #[cfg(feature = "lts")]
    pub fn of_vararg(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_vararg_typename, Private) }
    }

    /// The typename of the `UnionAll` `Array`.
    pub fn of_array(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_array_typename, Private) }
    }

    /// The typename of the `UnionAll` `Ptr`.
    #[cfg(not(feature = "lts"))]
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

impl<'scope> WrapperPriv<'scope, '_> for TypeName<'scope> {
    type Wraps = jl_typename_t;
    const NAME: &'static str = "TypeName";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(TypeName, 1);
