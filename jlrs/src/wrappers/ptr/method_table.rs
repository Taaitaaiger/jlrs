//! Wrapper for `Core.MethodTable`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L535

use super::private::Wrapper;
use crate::{impl_julia_typecheck, impl_valid_layout};
use crate::{
    private::Private,
    wrappers::ptr::{ArrayRef, ModuleRef, SymbolRef, ValueRef},
};
use jl_sys::{jl_methtable_t, jl_methtable_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// contains the TypeMap for one Type
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct MethodTable<'frame>(NonNull<jl_methtable_t>, PhantomData<&'frame ()>);

impl<'frame> MethodTable<'frame> {
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
    pub fn name(self) -> SymbolRef<'frame> {
        unsafe { SymbolRef::wrap(self.unwrap_non_null(Private).as_ref().name) }
    }

    /// The `defs` field.
    pub fn defs(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().defs) }
    }

    /// The `leafcache` field.
    pub fn leaf_cache(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().leafcache.cast()) }
    }

    /// The `cache` field.
    pub fn cache(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().cache) }
    }

    /// Max # of non-vararg arguments in a signature
    pub fn max_args(self) -> isize {
        unsafe { self.unwrap_non_null(Private).as_ref().max_args }
    }

    /// Keyword argument sorter function
    pub fn kw_sorter(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().kwsorter) }
    }

    /// Used for incremental serialization to locate original binding
    pub fn module(self) -> ModuleRef<'frame> {
        unsafe { ModuleRef::wrap(self.unwrap_non_null(Private).as_ref().module) }
    }

    /// The `backedges` field.
    pub fn backedges(self) -> ArrayRef<'frame, 'static> {
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

impl<'scope> Debug for MethodTable<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("MethodTable").finish()
    }
}

impl_julia_typecheck!(MethodTable<'frame>, jl_methtable_type, 'frame);

impl_valid_layout!(MethodTable<'frame>, 'frame);

impl<'scope> Wrapper<'scope, '_> for MethodTable<'scope> {
    type Internal = jl_methtable_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
