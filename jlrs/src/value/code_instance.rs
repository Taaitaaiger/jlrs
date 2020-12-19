//! Support for values with the `Core.CodeInstance` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L273

use super::method_instance::MethodInstance;
use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_code_instance_t, jl_code_instance_type};
use std::marker::PhantomData;

/// A `CodeInstance` represents an executable operation.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct CodeInstance<'frame>(*mut jl_code_instance_t, PhantomData<&'frame ()>);

impl<'frame> CodeInstance<'frame> {
    pub(crate) unsafe fn wrap(code_instance: *mut jl_code_instance_t) -> Self {
        CodeInstance(code_instance, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_code_instance_t {
        self.0
    }

    /// Method this instance is specialized from.
    pub fn def(self) -> MethodInstance<'frame> {
        unsafe { MethodInstance::wrap((&*self.ptr()).def) }
    }

    /// Next cache entry.
    pub fn next(self) -> Option<Self> {
        unsafe {
            let next = (&*self.ptr()).next;
            if next.is_null() {
                None
            } else {
                Some(CodeInstance::wrap(next))
            }
        }
    }

    /// Returns the minimum of the world range for which this object is valid to use.
    pub fn min_world(self) -> usize {
        unsafe { (&*self.ptr()).min_world }
    }

    /// Returns the maximum of the world range for which this object is valid to use.
    pub fn max_world(self) -> usize {
        unsafe { (&*self.ptr()).max_world }
    }

    /// Return type for fptr.
    pub fn rettype(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).rettype) }
    }

    /// Inferred constant return value, or null
    pub fn rettype_const(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let rettype_const = (&*self.ptr()).rettype_const;
            if rettype_const.is_null() {
                None
            } else {
                Some(Value::wrap(rettype_const))
            }
        }
    }

    /// Inferred `CodeInfo`, `Nothing`, or `None`.
    pub fn inferred(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let inferred = (&*self.ptr()).inferred;
            if inferred.is_null() {
                None
            } else {
                Some(Value::wrap(inferred))
            }
        }
    }

    /// If `specptr` is a specialized function signature for specTypes->rettype
    pub fn isspecsig(self) -> bool {
        unsafe { (&*self.ptr()).isspecsig != 0 }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for CodeInstance<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
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
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(CodeInstance<'frame>, jl_code_instance_type, 'frame);
impl_julia_type!(CodeInstance<'frame>, jl_code_instance_type, 'frame);
impl_valid_layout!(CodeInstance<'frame>, 'frame);
