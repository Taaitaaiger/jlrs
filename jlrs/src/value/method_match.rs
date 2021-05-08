//! Support for values with the `Core.MethodMatch` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/f9720dc2ebd6cd9e3086365f281e62506444ef37/src/julia.h#L585
use super::wrapper_ref::{MethodRef, SimpleVectorRef, ValueRef};
use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_method_match_t, jl_method_match_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct MethodMatch<'frame>(NonNull<jl_method_match_t>, PhantomData<&'frame ()>);

impl<'frame> MethodMatch<'frame> {
    pub(crate) unsafe fn wrap(method_match: *mut jl_method_match_t) -> Self {
        debug_assert!(!method_match.is_null());
        MethodMatch(NonNull::new_unchecked(method_match), PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_method_match_t> {
        self.0
    }

    /*
    for (a, b) in zip(fieldnames(Core.MethodMatch), fieldtypes(Core.MethodMatch))
        println(a, ": ", b)
    end
    spec_types: Type
    sparams: Core.SimpleVector
    method: Method
    fully_covers: Bool
    */

    pub fn spec_types(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).spec_types.cast()) }
    }

    pub fn sparams(self) -> SimpleVectorRef<'frame> {
        unsafe { SimpleVectorRef::wrap((&*self.inner().as_ptr()).sparams) }
    }

    pub fn method(self) -> MethodRef<'frame> {
        unsafe { MethodRef::wrap((&*self.inner().as_ptr()).method) }
    }

    /// A bool on the julia side, but can be temporarily 0x2 as a sentinel
    /// during construction.
    pub fn fully_covers(self) -> u8 {
        unsafe { (&*self.inner().as_ptr()).fully_covers }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for MethodMatch<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("MethodMatch").finish()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for MethodMatch<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap_non_null(self.inner().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for MethodMatch<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAMethodMatch)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.inner().as_ptr().cast())
    }
}

impl_julia_typecheck!(MethodMatch<'frame>, jl_method_match_type, 'frame);

impl_valid_layout!(MethodMatch<'frame>, 'frame);
