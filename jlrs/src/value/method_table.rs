//! Support for values with the `Core.MethodTable` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L535

use super::{
    wrapper_ref::{ArrayRef, ModuleRef, SymbolRef, ValueRef},
    Value,
};
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_methtable_t, jl_methtable_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// contains the TypeMap for one Type
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct MethodTable<'frame>(NonNull<jl_methtable_t>, PhantomData<&'frame ()>);

impl<'frame> MethodTable<'frame> {
    pub(crate) unsafe fn wrap(method_table: *mut jl_methtable_t) -> Self {
        debug_assert!(!method_table.is_null());
        MethodTable(NonNull::new_unchecked(method_table), PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_methtable_t> {
        self.0
    }

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
        unsafe { SymbolRef::wrap((&*self.inner().as_ptr()).name) }
    }

    /// The `defs` field.
    pub fn defs(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).defs) }
    }

    /// The `leafcache` field.
    pub fn leafcache(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).leafcache.cast()) }
    }

    /// The `cache` field.
    pub fn cache(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).cache) }
    }

    /// Max # of non-vararg arguments in a signature
    pub fn max_args(self) -> isize {
        unsafe { (&*self.inner().as_ptr()).max_args }
    }

    /// Keyword argument sorter function
    pub fn kwsorter(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).kwsorter) }
    }

    /// Used for incremental serialization to locate original binding
    pub fn module(self) -> ModuleRef<'frame> {
        unsafe { ModuleRef::wrap((&*self.inner().as_ptr()).module) }
    }

    /// The `backedges` field.
    pub fn backedges(self) -> ArrayRef<'frame, 'static> {
        unsafe { ArrayRef::wrap((&*self.inner().as_ptr()).backedges) }
    }

    /// 0, or 1 to skip splitting typemap on first (function) argument
    pub fn offs(self) -> u8 {
        unsafe { (&*self.inner().as_ptr()).offs }
    }

    /// Whether this accepts adding new methods
    pub fn frozen(self) -> u8 {
        unsafe { (&*self.inner().as_ptr()).frozen }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for MethodTable<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("MethodTable").finish()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for MethodTable<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap_non_null(self.inner().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for MethodTable<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAMethTable)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.inner().as_ptr().cast())
    }
}

impl_julia_typecheck!(MethodTable<'frame>, jl_methtable_type, 'frame);

impl_valid_layout!(MethodTable<'frame>, 'frame);
