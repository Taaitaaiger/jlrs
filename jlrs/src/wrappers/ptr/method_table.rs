//! Wrapper for `MethodTable`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L535

use super::private::Wrapper;
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout};
use crate::{
    private::Private,
    wrappers::ptr::{ArrayRef, ModuleRef, SymbolRef, ValueRef},
};
use jl_sys::{jl_methtable_t, jl_methtable_type};
use std::{marker::PhantomData, ptr::NonNull};

#[cfg(not(feature = "lts"))]
use super::atomic_value;
#[cfg(not(feature = "lts"))]
use std::sync::atomic::Ordering;

/// contains the TypeMap for one Type
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct MethodTable<'scope>(NonNull<jl_methtable_t>, PhantomData<&'scope ()>);

impl<'scope> MethodTable<'scope> {
    /*
    for (a, b) in zip(fieldnames(Core.MethodTable), fieldtypes(Core.MethodTable))
        println(a, ": ", b)
    end
    name: Symbol
    defs: Any
    leafcache: Any
    cache: Any
    max_args: Int64
    kwsorter: Any
    module: Module
    backedges: Vector{Any}
    : Int64
    : Int64
    offs: UInt8
    : UInt8
    */

    /// Sometimes a hack used by serialization to handle kwsorter
    pub fn name(self) -> SymbolRef<'scope> {
        unsafe { SymbolRef::wrap(self.unwrap_non_null(Private).as_ref().name) }
    }

    /// The `defs` field.
    #[cfg(feature = "lts")]
    pub fn defs(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().defs) }
    }

    /// The `defs` field.
    #[cfg(not(feature = "lts"))]
    pub fn defs(self) -> ValueRef<'scope, 'static> {
        unsafe {
            let defs = atomic_value(self.unwrap_non_null(Private).as_ref().cache);
            let ptr = defs.load(Ordering::Relaxed);
            ValueRef::wrap(ptr)
        }
    }

    /// The `leafcache` field.
    #[cfg(feature = "lts")]
    pub fn leafcache(self) -> ArrayRef<'scope, 'static> {
        unsafe { ArrayRef::wrap(self.unwrap_non_null(Private).as_ref().leafcache) }
    }

    /// The `leafcache` field.
    #[cfg(not(feature = "lts"))]
    pub fn leafcache(self) -> ArrayRef<'scope, 'static> {
        unsafe {
            let leafcache = atomic_value(self.unwrap_non_null(Private).as_ref().cache);
            let ptr = leafcache.load(Ordering::Relaxed);
            ArrayRef::wrap(ptr.cast())
        }
    }

    /// The `cache` field.
    #[cfg(feature = "lts")]
    pub fn cache(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().cache) }
    }

    /// The `cache` field.
    #[cfg(not(feature = "lts"))]
    pub fn cache(self) -> ValueRef<'scope, 'static> {
        unsafe {
            let cache = atomic_value(self.unwrap_non_null(Private).as_ref().cache);
            let ptr = cache.load(Ordering::Relaxed);
            ValueRef::wrap(ptr)
        }
    }

    /// Max # of non-vararg arguments in a signature
    pub fn max_args(self) -> isize {
        unsafe { self.unwrap_non_null(Private).as_ref().max_args }
    }

    /// Keyword argument sorter function
    pub fn kw_sorter(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().kwsorter) }
    }

    /// Used for incremental serialization to locate original binding
    pub fn module(self) -> ModuleRef<'scope> {
        unsafe { ModuleRef::wrap(self.unwrap_non_null(Private).as_ref().module) }
    }

    /// The `backedges` field.
    pub fn backedges(self) -> ArrayRef<'scope, 'static> {
        unsafe { ArrayRef::wrap(self.unwrap_non_null(Private).as_ref().backedges) }
    }

    /// 0, or 1 to skip splitting typemap on first (function) argument
    pub fn offs(self) -> u8 {
        unsafe { self.unwrap_non_null(Private).as_ref().offs }
    }

    /// Whether this accepts adding new methods
    pub fn frozen(self) -> u8 {
        unsafe { self.unwrap_non_null(Private).as_ref().frozen }
    }
}

impl_julia_typecheck!(MethodTable<'scope>, jl_methtable_type, 'scope);
impl_debug!(MethodTable<'_>);
impl_valid_layout!(MethodTable<'scope>, 'scope);

impl<'scope> Wrapper<'scope, '_> for MethodTable<'scope> {
    type Wraps = jl_methtable_t;
    const NAME: &'static str = "<MethodTable";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}
