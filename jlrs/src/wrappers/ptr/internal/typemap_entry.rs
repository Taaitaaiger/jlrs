//! Wrapper for `TypeMapEntry`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#505

use crate::{
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{private::WrapperPriv, value::ValueRef, Ref},
};
use cfg_if::cfg_if;
use jl_sys::{jl_typemap_entry_t, jl_typemap_entry_type};
use std::{marker::PhantomData, ptr::NonNull};

cfg_if! {
    if #[cfg(not(feature = "lts"))] {
        use std::sync::atomic::Ordering;
    }
}

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
    // TODO: check types
    pub fn next(self) -> ValueRef<'scope, 'static> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().next.cast()) }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let next = self.unwrap_non_null(Private).as_ref().next.load(Ordering::Relaxed);
                    ValueRef::wrap(next.cast())
                }
            }
        }
    }

    /// The type sig for this entry
    pub fn sig(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().sig.cast()) }
    }

    /// A simple signature for fast rejection
    pub fn simple_sig(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().simplesig.cast()) }
    }

    /// The `guardsigs` field.
    pub fn guard_sigs(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().guardsigs.cast()) }
    }

    /// The `min_world` field.
    pub fn min_world(self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().min_world }
    }

    /// The `max_world` field.
    pub fn max_world(self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().max_world }
    }

    /// The `func` field.
    pub fn func(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().func.value) }
    }

    /// `isleaftype(sig) & !any(isType, sig)` : unsorted and very fast
    pub fn is_leaf_signature(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().isleafsig != 0 }
    }

    /// `all(isleaftype | isAny | isType | isVararg, sig)` : sorted and fast
    pub fn is_simple_signature(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().issimplesig != 0 }
    }

    /// `isVararg(sig)`
    pub fn is_vararg(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().va != 0 }
    }

    /// Use the target to reroot this data.
    pub fn root<'target, T>(self, target: T) -> TypeMapEntryData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the data is valid.
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
    }
}

impl_julia_typecheck!(TypeMapEntry<'scope>, jl_typemap_entry_type, 'scope);
impl_debug!(TypeMapEntry<'_>);

impl<'scope> WrapperPriv<'scope, '_> for TypeMapEntry<'scope> {
    type Wraps = jl_typemap_entry_t;
    type TypeConstructorPriv<'target, 'da> = TypeMapEntry<'target>;
    const NAME: &'static str = "TypeMapEntry";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(TypeMapEntry, 1);

/// A reference to a [`TypeMapEntry`] that has not been explicitly rooted.
pub type TypeMapEntryRef<'scope> = Ref<'scope, 'static, TypeMapEntry<'scope>>;
impl_valid_layout!(TypeMapEntryRef, TypeMapEntry);
impl_ref_root!(TypeMapEntry, TypeMapEntryRef, 1);

use crate::memory::target::target_type::TargetType;
pub type TypeMapEntryData<'target, T> =
    <T as TargetType<'target>>::Data<'static, TypeMapEntry<'target>>;
pub type TypeMapEntryResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, TypeMapEntry<'target>>;
