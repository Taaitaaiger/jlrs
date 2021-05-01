//! Support for values with the `Core.CodeInstance` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L273

use super::wrapper_ref::{CodeInstanceRef, MethodInstanceRef, ValueRef};
use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_code_instance_t, jl_code_instance_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// A `CodeInstance` represents an executable operation.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct CodeInstance<'frame>(NonNull<jl_code_instance_t>, PhantomData<&'frame ()>);

impl<'frame> CodeInstance<'frame> {
    pub(crate) unsafe fn wrap(code_instance: *mut jl_code_instance_t) -> Self {
        debug_assert!(!code_instance.is_null());
        CodeInstance(NonNull::new_unchecked(code_instance), PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_code_instance_t> {
        self.0
    }

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
    isspecsig: Bool
    precompile: Bool
    invoke: Ptr{Nothing}
    specptr: Ptr{Nothing}
    */

    /// Method this instance is specialized from.
    pub fn def(self) -> MethodInstanceRef<'frame> {
        unsafe { MethodInstanceRef::wrap((&*self.inner().as_ptr()).def) }
    }

    /// Next cache entry.
    pub fn next(self) -> CodeInstanceRef<'frame> {
        unsafe { CodeInstanceRef::wrap((&*self.inner().as_ptr()).next) }
    }

    /// Returns the minimum of the world range for which this object is valid to use.
    pub fn min_world(self) -> usize {
        unsafe { (&*self.inner().as_ptr()).min_world }
    }

    /// Returns the maximum of the world range for which this object is valid to use.
    pub fn max_world(self) -> usize {
        unsafe { (&*self.inner().as_ptr()).max_world }
    }

    /// Return type for fptr.
    pub fn rettype(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).rettype) }
    }

    /// Inferred constant return value, or null
    pub fn rettype_const(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).rettype_const) }
    }

    /// Inferred `CodeInfo`, `Nothing`, or `None`.
    pub fn inferred(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).inferred) }
    }

    /// If `specptr` is a specialized function signature for specTypes->rettype
    pub fn isspecsig(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).isspecsig != 0 }
    }

    /// If `specptr` is a specialized function signature for specTypes->rettype
    pub fn precompile(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).precompile != 0 }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for CodeInstance<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.inner().as_ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for CodeInstance<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotACodeInstance)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.inner().as_ptr().cast())
    }
}

impl<'scope> Debug for CodeInstance<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("CodeInstance").finish()
    }
}

impl_julia_typecheck!(CodeInstance<'frame>, jl_code_instance_type, 'frame);
impl_julia_type!(CodeInstance<'frame>, jl_code_instance_type, 'frame);
impl_valid_layout!(CodeInstance<'frame>, 'frame);
