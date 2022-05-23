//! Wrapper for `CodeInstance`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L273

use crate::{
    impl_debug, impl_julia_typecheck,
    memory::output::Output,
    private::Private,
    wrappers::ptr::{
        private::Wrapper as WrapperPriv, CodeInstanceRef, MethodInstanceRef, ValueRef,
    },
};
use cfg_if::cfg_if;
use jl_sys::{jl_code_instance_t, jl_code_instance_type};
use std::{ffi::c_void, sync::atomic::AtomicPtr};
use std::{marker::PhantomData, ptr::NonNull, sync::atomic::AtomicU8};

cfg_if! {
    if #[cfg(not(feature = "lts"))] {
        use crate::wrappers::ptr::atomic_value;
        use std::sync::atomic::Ordering;
    }
}

/// A `CodeInstance` represents an executable operation.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct CodeInstance<'scope>(NonNull<jl_code_instance_t>, PhantomData<&'scope ()>);

impl<'scope> CodeInstance<'scope> {
    /*
    for (a, b) in zip(fieldnames(Core.CodeInstance), fieldtypes(Core.CodeInstance))
        println(a, ": ", b)
    end
    def: Core.MethodInstance
    next: Core.CodeInstance
    min_world: UInt64
    max_world: UInt64
    rettype: Any
    rettype_const: Any
    inferred: Any
    ipo_purity_bits: UInt32
    purity_bits: UInt32
    argescapes: Any
    isspecsig: Bool
    precompile: Bool _Atomic
    invoke: Ptr{Nothing} _Atomic
    specptr: Ptr{Nothing} _Atomic
    relocatability: UInt8
    */

    /// Method this instance is specialized from.
    pub fn def(self) -> MethodInstanceRef<'scope> {
        unsafe { MethodInstanceRef::wrap(self.unwrap_non_null(Private).as_ref().def) }
    }

    /// Next cache entry.
    #[cfg(feature = "lts")]
    pub fn next(self) -> CodeInstanceRef<'scope> {
        unsafe { CodeInstanceRef::wrap(self.unwrap_non_null(Private).as_ref().next) }
    }

    /// Next cache entry.
    #[cfg(not(feature = "lts"))]
    pub fn next(self) -> CodeInstanceRef<'scope> {
        unsafe {
            let next = atomic_value(&mut self.unwrap_non_null(Private).as_mut().next as *mut _);
            let ptr = next.load(Ordering::Relaxed);
            CodeInstanceRef::wrap(ptr.cast())
        }
    }

    /// Returns the minimum of the world range for which this object is valid to use.
    pub fn min_world(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().min_world }
    }

    /// Returns the maximum of the world range for which this object is valid to use.
    pub fn max_world(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().max_world }
    }

    /// Return type for fptr.
    pub fn rettype(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().rettype) }
    }

    /// Inferred constant return value, or null
    pub fn rettype_const(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().rettype_const) }
    }

    /// Inferred `CodeInfo`, `Nothing`, or `None`.
    pub fn inferred(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().inferred) }
    }

    /// The `ipo_purity_bits` field of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn ipo_purity_bits(self) -> u32 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_1
                .ipo_purity_bits
        }
    }

    /// The `ipo_consistent` flag of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn ipo_consistent(self) -> u8 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_1
                .ipo_purity_flags
                .ipo_consistent()
        }
    }

    /// The `ipo_effect_free` flag of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn ipo_effect_free(self) -> u8 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_1
                .ipo_purity_flags
                .ipo_effect_free()
        }
    }

    /// The `ipo_nothrow` flag of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn ipo_nothrow(self) -> u8 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_1
                .ipo_purity_flags
                .ipo_nothrow()
        }
    }

    /// The `ipo_terminates` flag of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn ipo_terminates(self) -> u8 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_1
                .ipo_purity_flags
                .ipo_terminates()
        }
    }

    /// The `ipo_overlayed` flag of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn ipo_overlayed(self) -> u8 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_1
                .ipo_purity_flags
                .ipo_overlayed()
        }
    }

    /// The `purity_bits` field of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn purity_bits(self) -> u32 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_2
                .purity_bits
        }
    }

    /// The `consistent` flag of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn consistent(self) -> u8 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_2
                .purity_flags
                .consistent()
        }
    }

    /// The `effect_free` flag of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn effect_free(self) -> u8 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_2
                .purity_flags
                .effect_free()
        }
    }

    /// The `nothrow` flag of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn nothrow(self) -> u8 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_2
                .purity_flags
                .nothrow()
        }
    }

    /// The `terminates` flag of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn terminates(self) -> u8 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_2
                .purity_flags
                .terminates()
        }
    }

    /// The `overlayed` flag of this `CodeInstance`.
    #[cfg(not(feature = "lts"))]
    pub fn overlayed(self) -> u8 {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .__bindgen_anon_2
                .purity_flags
                .overlayed()
        }
    }

    /// Method this instance is specialized from.
    pub fn argescapes(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().argescapes) }
    }

    /// If `specptr` is a specialized function signature for specTypes->rettype
    pub fn is_specsig(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().isspecsig != 0 }
    }

    /// If `specptr` is a specialized function signature for specTypes->rettype
    #[cfg(not(feature = "lts"))]
    pub fn precompile(self) -> bool {
        unsafe {
            let ptr =
                &self.unwrap_non_null(Private).as_ref().precompile as *const u8 as *const AtomicU8;
            let field_ref = &*ptr;
            field_ref.load(Ordering::SeqCst) != 0
        }
    }

    /// If `specptr` is a specialized function signature for specTypes->rettype
    #[cfg(feature = "lts")]
    pub fn precompile(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().precompile != 0 }
    }

    /// jlcall entry point
    #[cfg(not(feature = "lts"))]
    pub fn invoke(self) -> *mut c_void {
        unsafe {
            let ptr = &self.unwrap_non_null(Private).as_ref().invoke as *const _
                as *const AtomicPtr<c_void>;
            (&*ptr).load(Ordering::Relaxed)
        }
    }

    /// jlcall entry point
    #[cfg(feature = "lts")]
    pub fn invoke(self) -> *mut c_void {
        unsafe { &self.unwrap_non_null(Private).as_ref().invoke as *mut c_void }
    }

    /// private data for `jlcall entry point
    #[cfg(not(feature = "lts"))]
    pub fn specptr(self) -> *mut c_void {
        unsafe {
            let ptr = &self.unwrap_non_null(Private).as_ref().specptr as *const _
                as *const AtomicPtr<c_void>;
            (&*ptr).load(Ordering::Relaxed)
        }
    }

    /// private data for `jlcall entry point
    #[cfg(feature = "lts")]
    pub fn specptr(self) -> *mut c_void {
        unsafe { &self.unwrap_non_null(Private).as_ref().specptr as *mut c_void }
    }

    /// nonzero if all roots are built into sysimg or tagged by module key
    #[cfg(not(feature = "lts"))]
    pub fn relocatability(self) -> u8 {
        unsafe { self.unwrap_non_null(Private).as_ref().relocatability }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> CodeInstance<'target> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<CodeInstance>(ptr);
            CodeInstance::wrap_non_null(ptr, Private)
        }
    }
}

impl_julia_typecheck!(CodeInstance<'scope>, jl_code_instance_type, 'scope);
impl_debug!(CodeInstance<'_>);

impl<'scope> WrapperPriv<'scope, '_> for CodeInstance<'scope> {
    type Wraps = jl_code_instance_t;
    const NAME: &'static str = "CodeInstance";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, ::std::marker::PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(CodeInstance, 1);
