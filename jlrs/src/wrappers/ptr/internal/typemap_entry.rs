//! Wrapper for `TypeMapEntry`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#505

use super::super::private::Wrapper;
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout, memory::output::Output};
use crate::{private::Private, wrappers::ptr::ValueRef};
use jl_sys::{jl_typemap_entry_t, jl_typemap_entry_type};
use std::{marker::PhantomData, ptr::NonNull};

#[cfg(not(feature = "lts"))]
use super::super::atomic_value;
#[cfg(not(feature = "lts"))]
use std::sync::atomic::Ordering;

/// One Type-to-Value entry
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeMapEntry<'scope>(NonNull<jl_typemap_entry_t>, PhantomData<&'scope ()>);

impl<'scope> TypeMapEntry<'scope> {
    /*
    for (a,b) in zip(fieldnames(Core.TypeMapEntry), fieldtypes(Core.TypeMapEntry))
         println(a,": ", b)
    end
    next: Any _Atomic
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
    #[cfg(feature = "lts")]
    pub fn next(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().next.cast()) }
    }

    /// Invasive linked list
    #[cfg(not(feature = "lts"))]
    pub fn next(self) -> ValueRef<'scope, 'static> {
        unsafe {
            let next = atomic_value(self.unwrap_non_null(Private).as_ref().next);
            let ptr = next.load(Ordering::Relaxed);
            ValueRef::wrap(ptr.cast())
        }
    }

    /// The type sig for this entry
    pub fn sig(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().sig.cast()) }
    }

    /// A simple signature for fast rejection
    pub fn simple_sig(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().simplesig.cast()) }
    }

    /// The `guardsigs` field.
    pub fn guard_sigs(self) -> ValueRef<'scope, 'static> {
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
    pub fn func(self) -> ValueRef<'scope, 'static> {
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

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> TypeMapEntry<'target> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<TypeMapEntry>(ptr);
            TypeMapEntry::wrap_non_null(ptr, Private)
        }
    }
}

impl_julia_typecheck!(TypeMapEntry<'scope>, jl_typemap_entry_type, 'scope);
impl_debug!(TypeMapEntry<'_>);
impl_valid_layout!(TypeMapEntry<'scope>, 'scope);

impl<'scope> Wrapper<'scope, '_> for TypeMapEntry<'scope> {
    type Wraps = jl_typemap_entry_t;
    const NAME: &'static str = "TypeMapEntry";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(TypeMapEntry, 1);
