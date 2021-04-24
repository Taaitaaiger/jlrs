//! Support for values with the `Core.Method` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L273

use super::array::Array;
use super::method_instance::MethodInstance;
use super::module::Module;
use super::simple_vector::SimpleVector;
use super::symbol::Symbol;
use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_method_t, jl_method_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
};

/// This type describes a single method definition, and stores data shared by the specializations
/// of a function.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Method<'frame>(*mut jl_method_t, PhantomData<&'frame ()>);

impl<'frame> Method<'frame> {
    pub(crate) unsafe fn wrap(method: *mut jl_method_t) -> Self {
        Method(method, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_method_t {
        self.0
    }

    /// Method name for error reporting
    pub fn name(self) -> Symbol<'frame> {
        unsafe { Symbol::wrap((&*self.ptr()).name) }
    }

    /// Method module
    pub fn module(self) -> Module<'frame> {
        unsafe { Module::wrap((&*self.ptr()).module) }
    }

    /// Method file
    pub fn file(self) -> Symbol<'frame> {
        unsafe { Symbol::wrap((&*self.ptr()).file) }
    }

    /// Method line in file
    pub fn line(self) -> i32 {
        unsafe { (&*self.ptr()).line }
    }

    /// The `primary_world` field.
    pub fn primary_world(self) -> usize {
        unsafe { (&*self.ptr()).primary_world }
    }

    /// The `deleted_world` field.
    pub fn deleted_world(self) -> usize {
        unsafe { (&*self.ptr()).deleted_world }
    }

    /// Method's type signature.
    pub fn signature(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).sig) }
    }

    /// Table of all `Method` specializations, allocated as [hashable, ..., NULL, linear, ....]
    pub fn specializations(self) -> SimpleVector<'frame> {
        unsafe { SimpleVector::wrap((&*self.ptr()).specializations) }
    }

    /// Index lookup by hash into specializations
    pub fn speckeyset(self) -> Array<'frame, 'static> {
        unsafe { Array::wrap((&*self.ptr()).speckeyset) }
    }

    /// Compacted list of slot names (String)
    pub fn slot_syms(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).slot_syms) }
    }

    // Original code template (`Core.CodeInfo`, but may be compressed), `None` for builtins.
    pub fn source(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let source = (&*self.ptr()).source;
            if source.is_null() {
                None
            } else {
                Some(Value::wrap(source))
            }
        }
    }

    /// Unspecialized executable method instance, or `None`
    pub fn unspecialized(self) -> Option<MethodInstance<'frame>> {
        unsafe {
            let unspecialized = (&*self.ptr()).unspecialized;
            if unspecialized.is_null() {
                None
            } else {
                Some(MethodInstance::wrap(unspecialized))
            }
        }
    }

    /// Executable code-generating function if available
    pub fn generator(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let generator = (&*self.ptr()).generator;
            if generator.is_null() {
                None
            } else {
                Some(Value::wrap(generator))
            }
        }
    }

    /// Pointers in generated code (shared to reduce memory), or `None`
    pub fn roots(self) -> Option<Array<'frame, 'static>> {
        unsafe {
            let roots = (&*self.ptr()).roots;
            if roots.is_null() {
                None
            } else {
                Some(Array::wrap(roots))
            }
        }
    }

    /// `SimpleVector(rettype, sig)` if a ccallable entry point is requested for this
    pub fn ccallable(self) -> Option<SimpleVector<'frame>> {
        unsafe {
            let ccallable = (&*self.ptr()).ccallable;
            if ccallable.is_null() {
                None
            } else {
                Some(SimpleVector::wrap(ccallable))
            }
        }
    }

    /// Cache of specializations of this method for invoke(), i.e.
    /// cases where this method was called even though it was not necessarily
    /// the most specific for the argument types.
    pub fn invokes(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).invokes) }
    }

    /// The `n_args` field.
    pub fn n_args(self) -> i32 {
        unsafe { (&*self.ptr()).nargs }
    }

    /// Bit flags: whether each of the first 8 arguments is called
    pub fn called(self) -> i32 {
        unsafe { (&*self.ptr()).called }
    }

    /// Bit flags: which arguments should not be specialized
    pub fn nospecialize(self) -> i32 {
        unsafe { (&*self.ptr()).nospecialize }
    }

    /// Number of leading arguments that are actually keyword arguments
    /// of another method.
    pub fn nkw(self) -> i32 {
        unsafe { (&*self.ptr()).nkw }
    }

    /// The `is_varargs` field.
    pub fn is_varargs(self) -> bool {
        unsafe { (&*self.ptr()).isva != 0 }
    }

    /// The `pure` field.
    pub fn pure(self) -> bool {
        unsafe { (&*self.ptr()).pure_ != 0 }
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
        unsafe { Value::wrap(self.ptr().cast()) }
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
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(Method<'frame>, jl_method_type, 'frame);
impl_julia_type!(Method<'frame>, jl_method_type, 'frame);
impl_valid_layout!(Method<'frame>, 'frame);
