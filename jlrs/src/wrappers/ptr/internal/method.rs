//! Wrapper for `Method`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L273

use crate::{
    impl_julia_typecheck,
    prelude::Symbol,
    private::Private,
    wrappers::ptr::{
        array::ArrayRef, internal::method_instance::MethodInstanceRef, module::ModuleRef,
        private::WrapperPriv, simple_vector::SimpleVectorRef, value::ValueRef, Ref,
    },
};
use cfg_if::cfg_if;
use jl_sys::{jl_method_t, jl_method_type};
use std::{marker::PhantomData, ptr::NonNull};

cfg_if! {
    if #[cfg(not(feature = "lts"))] {
        use std::sync::atomic::Ordering;
    }
}

#[cfg(not(feature = "lts"))]
use crate::wrappers::ptr::array::TypedArrayRef;

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
    pub fn name(self) -> Option<Symbol<'scope>> {
        // Safety: the pointer points to valid data
        unsafe {
            let name = self.unwrap_non_null(Private).as_ref().name;
            let name = NonNull::new(name)?;
            Some(Symbol::wrap_non_null(name, Private))
        }
    }

    /// Method module
    pub fn module(self) -> Option<ModuleRef<'scope>> {
        // Safety: the pointer points to valid data
        unsafe {
            let module = self.unwrap_non_null(Private).as_ref().module;
            let module = NonNull::new(module)?;
            Some(ModuleRef::wrap(module))
        }
    }

    /// Method file
    pub fn file(self) -> Option<Symbol<'scope>> {
        // Safety: the pointer points to valid data
        unsafe {
            let file = self.unwrap_non_null(Private).as_ref().file;
            let file = NonNull::new(file)?;
            Some(Symbol::wrap_non_null(file, Private))
        }
    }

    /// Method line in file
    pub fn line(self) -> i32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().line }
    }

    /// The `primary_world` field.
    pub fn primary_world(self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().primary_world }
    }

    /// The `deleted_world` field.
    pub fn deleted_world(self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().deleted_world }
    }

    /// Method's type signature.
    pub fn signature(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let sig = self.unwrap_non_null(Private).as_ref().sig;
            let sig = NonNull::new(sig)?;
            Some(ValueRef::wrap(sig))
        }
    }

    /// Table of all `Method` specializations, allocated as [hashable, ..., NULL, linear, ....]
    pub fn specializations(self) -> Option<SimpleVectorRef<'scope>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let specializations = self.unwrap_non_null(Private).as_ref().specializations;
                    let specializations = NonNull::new(specializations)?;
                    Some(SimpleVectorRef::wrap(specializations))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let specializations = self.unwrap_non_null(Private).as_ref().specializations.load(Ordering::Relaxed);
                    let specializations = NonNull::new(specializations)?;
                    Some(SimpleVectorRef::wrap(specializations))
                }
            }
        }
    }

    /// Index lookup by hash into specializations
    pub fn spec_key_set(self) -> Option<ArrayRef<'scope, 'static>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let speckeyset = self.unwrap_non_null(Private).as_ref().speckeyset;
                    let speckeyset = NonNull::new(speckeyset)?;
                    Some(ArrayRef::wrap(speckeyset))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let speckeyset = self.unwrap_non_null(Private).as_ref().speckeyset.load(Ordering::Relaxed);
                    let speckeyset = NonNull::new(speckeyset)?;
                    Some(ArrayRef::wrap(speckeyset))
                }
            }
        }
    }

    /// Compacted list of slot names (String)
    pub fn slot_syms(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let slot_syms = self.unwrap_non_null(Private).as_ref().slot_syms;
            let slot_syms = NonNull::new(slot_syms)?;
            Some(ValueRef::wrap(slot_syms))
        }
    }

    /// reference to the method table this method is part of, null if part of the internal table
    #[cfg(not(feature = "lts"))]
    pub fn external_mt(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().external_mt;
            let data = NonNull::new(data)?;
            Some(ValueRef::wrap(data))
        }
    }

    // Original code template (`Core.CodeInfo`, but may be compressed), `None` for builtins.
    pub fn source(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().source;
            let data = NonNull::new(data)?;
            Some(ValueRef::wrap(data))
        }
    }

    /// Unspecialized executable method instance, or `None`
    pub fn unspecialized(self) -> Option<MethodInstanceRef<'scope>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let unspecialized = self.unwrap_non_null(Private).as_ref().unspecialized;
                    let unspecialized = NonNull::new(unspecialized)?;
                    Some(MethodInstanceRef::wrap(unspecialized))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let unspecialized = self.unwrap_non_null(Private).as_ref().unspecialized.load(Ordering::Relaxed);
                    let unspecialized = NonNull::new(unspecialized)?;
                    Some(MethodInstanceRef::wrap(unspecialized))
                }
            }
        }
    }

    /// Executable code-generating function if available
    pub fn generator(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().generator;
            let data = NonNull::new(data)?;
            Some(ValueRef::wrap(data))
        }
    }

    /// Pointers in generated code (shared to reduce memory), or `None`
    pub fn roots(self) -> Option<ArrayRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().roots;
            let data = NonNull::new(data)?;
            Some(ArrayRef::wrap(data))
        }
    }

    /// RLE (build_id, offset) pairs (even/odd indexing)
    #[cfg(not(feature = "lts"))]
    pub fn root_blocks(self) -> Option<TypedArrayRef<'scope, 'static, u64>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().root_blocks;
            let data = NonNull::new(data)?;
            Some(TypedArrayRef::wrap(data))
        }
    }

    /// # of roots stored in the system image
    #[cfg(not(feature = "lts"))]
    pub fn nroots_sysimg(self) -> i32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().nroots_sysimg }
    }

    /// `SimpleVector(rettype, sig)` if a ccallable entry point is requested for this
    pub fn ccallable(self) -> Option<SimpleVectorRef<'scope>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().ccallable;
            let data = NonNull::new(data)?;
            Some(SimpleVectorRef::wrap(data))
        }
    }

    /// Cache of specializations of this method for invoke(), i.e.
    /// cases where this method was called even though it was not necessarily
    /// the most specific for the argument types.
    pub fn invokes(self) -> Option<ValueRef<'scope, 'static>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let invokes = self.unwrap_non_null(Private).as_ref().invokes;
                    let invokes = NonNull::new(invokes)?;
                    Some(ValueRef::wrap(invokes))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let invokes = self.unwrap_non_null(Private).as_ref().invokes.load(Ordering::Relaxed);
                    let invokes = NonNull::new(invokes)?;
                    Some(ValueRef::wrap(invokes))
                }
            }
        }
    }

    /// The `n_args` field.
    pub fn n_args(self) -> u32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().nargs as u32 }
    }

    /// Bit flags: whether each of the first 8 arguments is called
    pub fn called(self) -> u32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().called as u32 }
    }

    /// Bit flags: which arguments should not be specialized
    pub fn no_specialize(self) -> u32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().nospecialize as u32 }
    }

    /// Number of leading arguments that are actually keyword arguments
    /// of another method.
    pub fn nkw(self) -> u32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().nkw as u32 }
    }

    /// The `isva` field.
    pub fn is_varargs(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().isva != 0 }
    }

    /// The `pure` field.
    pub fn pure(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().pure_ != 0 }
    }

    /// The `is_for_opaque_closure` field of this `Method`
    #[cfg(not(feature = "lts"))]
    pub fn is_for_opaque_closure(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().is_for_opaque_closure != 0 }
    }

    /// 0x00 = use heuristic; 0x01 = aggressive; 0x02 = none
    #[cfg(not(feature = "lts"))]
    pub fn constprop(self) -> u8 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().constprop }
    }

    /// Override the conclusions of inter-procedural effect analysis,
    /// forcing the conclusion to always true.
    #[cfg(not(feature = "lts"))]
    pub fn purity(self) -> u8 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().purity.bits }
    }
}

impl_julia_typecheck!(Method<'scope>, jl_method_type, 'scope);
impl_debug!(Method<'_>);

impl<'scope> WrapperPriv<'scope, '_> for Method<'scope> {
    type Wraps = jl_method_t;
    type TypeConstructorPriv<'target, 'da> = Method<'target>;
    const NAME: &'static str = "Method";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, ::std::marker::PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// A reference to a [`Method`] that has not been explicitly rooted.
pub type MethodRef<'scope> = Ref<'scope, 'static, Method<'scope>>;
impl_valid_layout!(MethodRef, Method);
impl_ref_root!(Method, MethodRef, 1);

use crate::memory::target::target_type::TargetType;

/// `Method` or `MethodRef`, depending on the target type `T`.
pub type MethodData<'target, T> = <T as TargetType<'target>>::Data<'static, Method<'target>>;

/// `JuliaResult<Method>` or `JuliaResultRef<MethodRef>`, depending on the target type `T`.
pub type MethodResult<'target, T> = <T as TargetType<'target>>::Result<'static, Method<'target>>;
