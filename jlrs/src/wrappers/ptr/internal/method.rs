//! Wrapper for `Method`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L273

use crate::{
    impl_debug, impl_julia_typecheck, impl_valid_layout, memory::output::Output,
    wrappers::ptr::TypedArrayRef,
};
use crate::{
    private::Private,
    wrappers::ptr::{
        private::Wrapper, ArrayRef, MethodInstanceRef, ModuleRef, SimpleVectorRef, SymbolRef,
        ValueRef,
    },
};
use jl_sys::{jl_method_t, jl_method_type};
use std::{marker::PhantomData, ptr::NonNull};

#[cfg(not(feature = "lts"))]
use super::super::atomic_value;
#[cfg(not(feature = "lts"))]
use std::sync::atomic::Ordering;

/// This type describes a single method definition, and stores data shared by the specializations
/// of a function.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Method<'scope>(NonNull<jl_method_t>, PhantomData<&'scope ()>);

impl<'scope> Method<'scope> {
    /*
    for (a, b) in zip(fieldnames(Method), fieldtypes(Method))
        println(a, ": ", b)
    end
    name: Symbol
    module: Module
    file: Symbol
    line: Int32
    primary_world: UInt64
    deleted_world: UInt64
    sig: Type
    specializations: Core.SimpleVector _Atomic
    speckeyset: Array _Atomic
    slot_syms: String
    external_mt: Any
    source: Any
    unspecialized: Core.MethodInstance _Atomic
    generator: Any
    roots: Vector{Any}
    root_blocks: Vector{UInt64}
    nroots_sysimg: Int32
    ccallable: Core.SimpleVector
    invokes: Any _Atomic
    recursion_relation: Any
    nargs: Int32
    called: Int32
    nospecialize: Int32
    nkw: Int32
    isva: Bool
    pure: Bool
    is_for_opaque_closure: Bool
    constprop: UInt8
    purity: UInt8
    */

    /// Method name for error reporting
    pub fn name(self) -> SymbolRef<'scope> {
        unsafe { SymbolRef::wrap(self.unwrap_non_null(Private).as_ref().name) }
    }

    /// Method module
    pub fn module(self) -> ModuleRef<'scope> {
        unsafe { ModuleRef::wrap(self.unwrap_non_null(Private).as_ref().module) }
    }

    /// Method file
    pub fn file(self) -> SymbolRef<'scope> {
        unsafe { SymbolRef::wrap(self.unwrap_non_null(Private).as_ref().file) }
    }

    /// Method line in file
    pub fn line(self) -> i32 {
        unsafe { self.unwrap_non_null(Private).as_ref().line }
    }

    /// The `primary_world` field.
    pub fn primary_world(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().primary_world }
    }

    /// The `deleted_world` field.
    pub fn deleted_world(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().deleted_world }
    }

    /// Method's type signature.
    pub fn signature(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().sig) }
    }

    /// Table of all `Method` specializations, allocated as [hashable, ..., NULL, linear, ....]
    #[cfg(feature = "lts")]
    pub fn specializations(self) -> SimpleVectorRef<'scope> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().specializations) }
    }

    /// Table of all `Method` specializations, allocated as [hashable, ..., NULL, linear, ....]
    #[cfg(not(feature = "lts"))]
    pub fn specializations(self) -> SimpleVectorRef<'scope> {
        unsafe {
            let specializations =
                atomic_value(self.unwrap_non_null(Private).as_ref().specializations);
            let ptr = specializations.load(Ordering::Relaxed);
            SimpleVectorRef::wrap(ptr.cast())
        }
    }

    /// Index lookup by hash into specializations
    #[cfg(feature = "lts")]
    pub fn spec_key_set(self) -> ArrayRef<'scope, 'static> {
        unsafe { ArrayRef::wrap(self.unwrap_non_null(Private).as_ref().speckeyset) }
    }

    /// Index lookup by hash into specializations
    #[cfg(not(feature = "lts"))]
    pub fn spec_key_set(self) -> ArrayRef<'scope, 'static> {
        unsafe {
            let speckeyset = atomic_value(self.unwrap_non_null(Private).as_ref().speckeyset);
            let ptr = speckeyset.load(Ordering::Relaxed);
            ArrayRef::wrap(ptr.cast())
        }
    }

    /// Compacted list of slot names (String)
    pub fn slot_syms(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().slot_syms) }
    }

    /// reference to the method table this method is part of, null if part of the internal table
    #[cfg(not(feature = "lts"))]
    pub fn external_mt(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().external_mt) }
    }

    // Original code template (`Core.CodeInfo`, but may be compressed), `None` for builtins.
    pub fn source(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().source) }
    }

    /// Unspecialized executable method instance, or `None`
    #[cfg(feature = "lts")]
    pub fn unspecialized(self) -> MethodInstanceRef<'scope> {
        unsafe { MethodInstanceRef::wrap(self.unwrap_non_null(Private).as_ref().unspecialized) }
    }

    /// Unspecialized executable method instance, or `None`
    #[cfg(not(feature = "lts"))]
    pub fn unspecialized(self) -> MethodInstanceRef<'scope> {
        unsafe {
            let unspecialized = atomic_value(self.unwrap_non_null(Private).as_ref().unspecialized);
            let ptr = unspecialized.load(Ordering::Relaxed);
            MethodInstanceRef::wrap(ptr.cast())
        }
    }

    /// Executable code-generating function if available
    pub fn generator(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().generator) }
    }

    /// Pointers in generated code (shared to reduce memory), or `None`
    pub fn roots(self) -> ArrayRef<'scope, 'static> {
        unsafe { ArrayRef::wrap(self.unwrap_non_null(Private).as_ref().roots) }
    }

    /// RLE (build_id, offset) pairs (even/odd indexing)
    #[cfg(not(feature = "lts"))]
    pub fn root_blocks(self) -> TypedArrayRef<'scope, 'static, u64> {
        unsafe { TypedArrayRef::wrap(self.unwrap_non_null(Private).as_ref().root_blocks) }
    }

    /// # of roots stored in the system image
    #[cfg(not(feature = "lts"))]
    pub fn nroots_sysimg(self) -> i32 {
        unsafe { self.unwrap_non_null(Private).as_ref().nroots_sysimg }
    }

    /// `SimpleVector(rettype, sig)` if a ccallable entry point is requested for this
    pub fn ccallable(self) -> SimpleVectorRef<'scope> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().ccallable) }
    }

    /// Cache of specializations of this method for invoke(), i.e.
    /// cases where this method was called even though it was not necessarily
    /// the most specific for the argument types.
    #[cfg(feature = "lts")]
    pub fn invokes(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().invokes) }
    }

    /// Cache of specializations of this method for invoke(), i.e.
    /// cases where this method was called even though it was not necessarily
    /// the most specific for the argument types.
    #[cfg(not(feature = "lts"))]
    pub fn invokes(self) -> ValueRef<'scope, 'static> {
        unsafe {
            let invokes = atomic_value(self.unwrap_non_null(Private).as_ref().invokes);
            let ptr = invokes.load(Ordering::Relaxed);
            ValueRef::wrap(ptr.cast())
        }
    }

    /// The `n_args` field.
    pub fn n_args(self) -> u32 {
        unsafe { self.unwrap_non_null(Private).as_ref().nargs }
    }

    /// Bit flags: whether each of the first 8 arguments is called
    pub fn called(self) -> u32 {
        unsafe { self.unwrap_non_null(Private).as_ref().called }
    }

    /// Bit flags: which arguments should not be specialized
    pub fn no_specialize(self) -> u32 {
        unsafe { self.unwrap_non_null(Private).as_ref().nospecialize }
    }

    /// Number of leading arguments that are actually keyword arguments
    /// of another method.
    pub fn nkw(self) -> u32 {
        unsafe { self.unwrap_non_null(Private).as_ref().nkw }
    }

    /// The `isva` field.
    pub fn is_varargs(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().isva != 0 }
    }

    /// The `pure` field.
    pub fn pure(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().pure_ != 0 }
    }

    /// The `is_for_opaque_closure` field of this `Method`
    #[cfg(not(feature = "lts"))]
    pub fn is_for_opaque_closure(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().is_for_opaque_closure != 0 }
    }

    /// 0x00 = use heuristic; 0x01 = aggressive; 0x02 = none
    #[cfg(not(feature = "lts"))]
    pub fn constprop(self) -> u8 {
        unsafe { self.unwrap_non_null(Private).as_ref().constprop }
    }

    /// Override the conclusions of inter-procedural effect analysis,
    /// forcing the conclusion to always true.
    #[cfg(not(feature = "lts"))]
    pub fn purity(self) -> u8 {
        unsafe { self.unwrap_non_null(Private).as_ref().purity.bits }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> Method<'target> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<Method>(ptr);
            Method::wrap_non_null(ptr, Private)
        }
    }
}

impl_julia_typecheck!(Method<'scope>, jl_method_type, 'scope);
impl_valid_layout!(Method<'scope>, 'scope);
impl_debug!(Method<'_>);

impl<'scope> Wrapper<'scope, '_> for Method<'scope> {
    type Wraps = jl_method_t;
    const NAME: &'static str = "Method";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, ::std::marker::PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(Method, 1);
