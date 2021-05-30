//! Wrapper for `Core.TypeMapEntry`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#505

use super::private::Wrapper;
use crate::{impl_julia_typecheck, impl_valid_layout};
use crate::{private::Private, wrappers::ptr::ValueRef};
use jl_sys::{jl_typemap_entry_t, jl_typemap_entry_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// One Type-to-Value entry
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeMapEntry<'frame>(NonNull<jl_typemap_entry_t>, PhantomData<&'frame ()>);

impl<'frame> TypeMapEntry<'frame> {
    /*
    for (a,b) in zip(fieldnames(Core.TypeMapEntry), fieldtypes(Core.TypeMapEntry))
         println(a,": ", b)
    end
    next: Any
    sig: Type
    simplesig: Any
    guardsigs: Any
    min_world: UInt64
    max_world: UInt64
    func: Any
    isleafsig: Bool
    issimplesig: Bool
    va: Bool
    */

    /// Invasive linked list
    pub fn next(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().next.cast()) }
    }

    /// The type sig for this entry
    pub fn sig(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().sig.cast()) }
    }

    /// A simple signature for fast rejection
    pub fn simple_sig(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().simplesig.cast()) }
    }

    /// The `guardsigs` field.
    pub fn guard_sigs(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().guardsigs.cast()) }
    }

    /// The `min_world` field.
    pub fn min_world(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().min_world }
    }

    /// The `max_world` field.
    pub fn max_world(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().max_world }
    }

    /// The `func` field.
    pub fn func(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().func.value) }
    }

    /// `isleaftype(sig) & !any(isType, sig)` : unsorted and very fast
    pub fn is_leaf_signature(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().isleafsig != 0 }
    }

    /// `all(isleaftype | isAny | isType | isVararg, sig)` : sorted and fast
    pub fn is_simple_signature(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().issimplesig != 0 }
    }

    /// `isVararg(sig)`
    pub fn is_vararg(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().va != 0 }
    }
}

impl<'scope> Debug for TypeMapEntry<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("TypeMapEntry").finish()
    }
}

impl_julia_typecheck!(TypeMapEntry<'frame>, jl_typemap_entry_type, 'frame);

impl_valid_layout!(TypeMapEntry<'frame>, 'frame);

impl<'scope> Wrapper<'scope, '_> for TypeMapEntry<'scope> {
    type Internal = jl_typemap_entry_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
