//! Wrapper for `TypeName`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L380

use crate::{
    impl_julia_typecheck,
    memory::{global::Global, output::Output},
    private::Private,
    wrappers::ptr::{
        module::ModuleRef, private::WrapperPriv, simple_vector::SimpleVectorRef, symbol::SymbolRef,
        value::ValueRef,
    },
};
use cfg_if::cfg_if;
use jl_sys::{
    jl_array_typename, jl_llvmpointer_typename, jl_namedtuple_typename, jl_pointer_typename,
    jl_tuple_typename, jl_type_typename, jl_typename_t, jl_typename_type, jl_vecelement_typename,
};
use std::{marker::PhantomData, ptr::NonNull};

use super::Ref;

cfg_if! {
    if #[cfg(all(feature = "lts", not(feature = "all-features-override")))] {
        use jl_sys::jl_vararg_typename;

    } else {
        use jl_sys::{jl_opaque_closure_typename};
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
        // Safety: the pointer points to valid data
        unsafe { SymbolRef::wrap(self.unwrap_non_null(Private).as_ref().name) }
    }

    /// The `module` field.
    pub fn module(self) -> ModuleRef<'scope> {
        // Safety: the pointer points to valid data
        unsafe { ModuleRef::wrap(self.unwrap_non_null(Private).as_ref().module) }
    }

    /// Field names.
    pub fn names(self) -> SimpleVectorRef<'scope> {
        // Safety: the pointer points to valid data
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().names) }
    }

    /// The `atomicfields` field.
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    pub fn atomicfields(self) -> *const u32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().atomicfields }
    }

    /// The `atomicfields` field.
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    pub fn constfields(self) -> *const u32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().constfields }
    }

    /// Either the only instantiation of the type (if no parameters) or a `UnionAll` accepting
    /// parameters to make an instantiation.
    pub fn wrapper(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().wrapper) }
    }

    /// Sorted array.
    pub fn cache(self) -> SimpleVectorRef<'scope> {
        cfg_if! {
            if #[cfg(all(feature = "lts", not(feature = "all-features-override")))] {
                // Safety: the pointer points to valid data
                unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().cache) }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let cache = self.unwrap_non_null(Private).as_ref().cache.load(Ordering::Relaxed);
                    SimpleVectorRef::wrap(cache)
                }
            }
        }
    }

    /// Unsorted array.
    pub fn linear_cache(self) -> SimpleVectorRef<'scope> {
        cfg_if! {
            if #[cfg(all(feature = "lts", not(feature = "all-features-override")))] {
                // Safety: the pointer points to valid data
                unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().linearcache) }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let cache = self.unwrap_non_null(Private).as_ref().linearcache.load(Ordering::Relaxed);
                    SimpleVectorRef::wrap(cache)
                }
            }
        }
    }

    /// The `mt` field.
    pub fn mt(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().mt.cast()) }
    }

    /// Incomplete instantiations of this type.
    pub fn partial(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().partial.cast()) }
    }

    /// The `hash` field.
    pub fn hash(self) -> isize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().hash }
    }

    /// The `n_uninitialized` field.
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    pub fn n_uninitialized(self) -> i32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().n_uninitialized }
    }

    /// The `abstract` field.
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    pub fn abstract_(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().abstract_() != 0 }
    }

    /// The `mutabl` field.
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    pub fn mutabl(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().mutabl() != 0 }
    }

    /// The `mayinlinealloc` field.
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    pub fn mayinlinealloc(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().mayinlinealloc() != 0 }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> TypeName<'target> {
        // Safety: the pointer points to valid data
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
        // Safety: global constant
        unsafe { Self::wrap(jl_type_typename, Private) }
    }

    /// The typename of the `DataType` `Tuple`.
    pub fn of_tuple(_: Global<'base>) -> Self {
        // Safety: global constant
        unsafe { Self::wrap(jl_tuple_typename, Private) }
    }

    /// The typename of the `UnionAll` `VecElement`.
    pub fn of_vecelement(_: Global<'base>) -> Self {
        // Safety: global constant
        unsafe { Self::wrap(jl_vecelement_typename, Private) }
    }

    /// The typename of the `UnionAll` `Vararg`.
    #[cfg(all(feature = "lts", not(feature = "all-features-override")))]
    pub fn of_vararg(_: Global<'base>) -> Self {
        // Safety: global constant
        unsafe { Self::wrap(jl_vararg_typename, Private) }
    }

    /// The typename of the `UnionAll` `Array`.
    pub fn of_array(_: Global<'base>) -> Self {
        // Safety: global constant
        unsafe { Self::wrap(jl_array_typename, Private) }
    }

    /// The typename of the `UnionAll` `Ptr`.
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    pub fn of_opaque_closure(_: Global<'base>) -> Self {
        // Safety: global constant
        unsafe { Self::wrap(jl_opaque_closure_typename, Private) }
    }

    /// The typename of the `UnionAll` `Ptr`.
    pub fn of_pointer(_: Global<'base>) -> Self {
        // Safety: global constant
        unsafe { Self::wrap(jl_pointer_typename, Private) }
    }

    /// The typename of the `UnionAll` `LLVMPtr`.
    pub fn of_llvmpointer(_: Global<'base>) -> Self {
        // Safety: global constant
        unsafe { Self::wrap(jl_llvmpointer_typename, Private) }
    }

    /// The typename of the `UnionAll` `NamedTuple`.
    pub fn of_namedtuple(_: Global<'base>) -> Self {
        // Safety: global constant
        unsafe { Self::wrap(jl_namedtuple_typename, Private) }
    }
}

impl_julia_typecheck!(TypeName<'scope>, jl_typename_type, 'scope);
impl_debug!(TypeName<'_>);

impl<'scope> WrapperPriv<'scope, '_> for TypeName<'scope> {
    type Wraps = jl_typename_t;
    const NAME: &'static str = "TypeName";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(TypeName, 1);

/// A reference to a [`TypeName`] that has not been explicitly rooted.
pub type TypeNameRef<'scope> = Ref<'scope, 'static, TypeName<'scope>>;
impl_valid_layout!(TypeNameRef, TypeName);
impl_ref_root!(TypeName, TypeNameRef, 1);
