//! Support for values with the `Core.Method` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L273

use super::wrapper_ref::{
    ArrayRef, MethodInstanceRef, ModuleRef, SimpleVectorRef, SymbolRef, ValueRef,
};
use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_method_t, jl_method_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// This type describes a single method definition, and stores data shared by the specializations
/// of a function.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Method<'frame>(NonNull<jl_method_t>, PhantomData<&'frame ()>);

impl<'frame> Method<'frame> {
    pub(crate) unsafe fn wrap(method: *mut jl_method_t) -> Self {
        debug_assert!(!method.is_null());
        Method(NonNull::new_unchecked(method), PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_method_t> {
        self.0
    }

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
    specializations: Core.SimpleVector
    speckeyset: Array
    slot_syms: String
    source: Any
    unspecialized: Core.MethodInstance
    generator: Any
    roots: Vector{Any}
    ccallable: Core.SimpleVector
    invokes: Any
    nargs: Int32
    called: Int32
    nospecialize: Int32
    nkw: Int32
    isva: Bool
    pure: Bool
    */

    /// Method name for error reporting
    pub fn name(self) -> SymbolRef<'frame> {
        unsafe { SymbolRef::wrap((&*self.inner().as_ptr()).name) }
    }

    /// Method module
    pub fn module(self) -> ModuleRef<'frame> {
        unsafe { ModuleRef::wrap((&*self.inner().as_ptr()).module) }
    }

    /// Method file
    pub fn file(self) -> SymbolRef<'frame> {
        unsafe { SymbolRef::wrap((&*self.inner().as_ptr()).file) }
    }

    /// Method line in file
    pub fn line(self) -> i32 {
        unsafe { (&*self.inner().as_ptr()).line }
    }

    /// The `primary_world` field.
    pub fn primary_world(self) -> usize {
        unsafe { (&*self.inner().as_ptr()).primary_world }
    }

    /// The `deleted_world` field.
    pub fn deleted_world(self) -> usize {
        unsafe { (&*self.inner().as_ptr()).deleted_world }
    }

    /// Method's type signature.
    pub fn signature(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).sig) }
    }

    /// Table of all `Method` specializations, allocated as [hashable, ..., NULL, linear, ....]
    pub fn specializations(self) -> SimpleVectorRef<'frame> {
        unsafe { SimpleVectorRef::wrap((&*self.inner().as_ptr()).specializations) }
    }

    /// Index lookup by hash into specializations
    pub fn speckeyset(self) -> ArrayRef<'frame, 'static> {
        unsafe { ArrayRef::wrap((&*self.inner().as_ptr()).speckeyset) }
    }

    /// Compacted list of slot names (String)
    pub fn slot_syms(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).slot_syms) }
    }

    // Original code template (`Core.CodeInfo`, but may be compressed), `None` for builtins.
    pub fn source(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).source) }
    }

    /// Unspecialized executable method instance, or `None`
    pub fn unspecialized(self) -> MethodInstanceRef<'frame> {
        unsafe { MethodInstanceRef::wrap((&*self.inner().as_ptr()).unspecialized) }
    }

    /// Executable code-generating function if available
    pub fn generator(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).generator) }
    }

    /// Pointers in generated code (shared to reduce memory), or `None`
    pub fn roots(self) -> ArrayRef<'frame, 'static> {
        unsafe { ArrayRef::wrap((&*self.inner().as_ptr()).roots) }
    }

    /// `SimpleVector(rettype, sig)` if a ccallable entry point is requested for this
    pub fn ccallable(self) -> SimpleVectorRef<'frame> {
        unsafe { SimpleVectorRef::wrap((&*self.inner().as_ptr()).ccallable) }
    }

    /// Cache of specializations of this method for invoke(), i.e.
    /// cases where this method was called even though it was not necessarily
    /// the most specific for the argument types.
    pub fn invokes(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).invokes) }
    }

    /// The `n_args` field.
    pub fn n_args(self) -> i32 {
        unsafe { (&*self.inner().as_ptr()).nargs }
    }

    /// Bit flags: whether each of the first 8 arguments is called
    pub fn called(self) -> i32 {
        unsafe { (&*self.inner().as_ptr()).called }
    }

    /// Bit flags: which arguments should not be specialized
    pub fn nospecialize(self) -> i32 {
        unsafe { (&*self.inner().as_ptr()).nospecialize }
    }

    /// Number of leading arguments that are actually keyword arguments
    /// of another method.
    pub fn nkw(self) -> i32 {
        unsafe { (&*self.inner().as_ptr()).nkw }
    }

    /// The `is_varargs` field.
    pub fn is_varargs(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).isva != 0 }
    }

    /// The `pure` field.
    pub fn pure(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).pure_ != 0 }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for Method<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Method").finish()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for Method<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap_non_null(self.inner().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Method<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAMethod)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.inner().as_ptr().cast())
    }
}

impl_julia_typecheck!(Method<'frame>, jl_method_type, 'frame);

impl_valid_layout!(Method<'frame>, 'frame);
