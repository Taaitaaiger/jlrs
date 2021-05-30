//! Wrapper for `Core.MethodMatch`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/f9720dc2ebd6cd9e3086365f281e62506444ef37/src/julia.h#L585
use super::private::Wrapper;
use crate::{impl_julia_typecheck, impl_valid_layout};
use crate::{
    private::Private,
    wrappers::ptr::{MethodRef, SimpleVectorRef, ValueRef},
};
use jl_sys::{jl_method_match_t, jl_method_match_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct MethodMatch<'frame>(NonNull<jl_method_match_t>, PhantomData<&'frame ()>);

impl<'frame> MethodMatch<'frame> {
    /*
    for (a, b) in zip(fieldnames(Core.MethodMatch), fieldtypes(Core.MethodMatch))
        println(a, ": ", b)
    end
    spec_types: Type
    sparams: Core.SimpleVector
    method: Method
    fully_covers: Bool
    */

    /// The `spec_types` field.
    pub fn spec_types(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().spec_types.cast()) }
    }

    /// The `sparams` field.
    pub fn sparams(self) -> SimpleVectorRef<'frame> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().sparams) }
    }

    /// The `method` field.
    pub fn method(self) -> MethodRef<'frame> {
        unsafe { MethodRef::wrap(self.unwrap_non_null(Private).as_ref().method) }
    }

    /// A bool on the julia side, but can be temporarily 0x2 as a sentinel
    /// during construction.
    pub fn fully_covers(self) -> u8 {
        unsafe { self.unwrap_non_null(Private).as_ref().fully_covers }
    }
}

impl<'scope> Debug for MethodMatch<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("MethodMatch").finish()
    }
}

impl_julia_typecheck!(MethodMatch<'frame>, jl_method_match_type, 'frame);

impl_valid_layout!(MethodMatch<'frame>, 'frame);

impl<'scope> Wrapper<'scope, '_> for MethodMatch<'scope> {
    type Internal = jl_method_match_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
