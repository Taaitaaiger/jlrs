//! A wrapper around a `Core.SSAValue` in Julia.

use crate::{convert::unbox::Unbox, impl_julia_typecheck, impl_valid_layout};
use crate::{
    private::Private,
    wrappers::ptr::{private::Wrapper, value::Value},
};
use jl_sys::jl_ssavalue_type;
use std::fmt::{Debug, Formatter, Result as FmtResult};

/// A Julia `SSAValue`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SSAValue(isize);

impl SSAValue {
    /// Returns the id of the `SSAValue`.
    pub fn id(self) -> isize {
        self.0
    }
}

impl<'scope> Debug for SSAValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_fmt(format_args!("SSAValue<{}>", self.0))
    }
}

impl_julia_typecheck!(SSAValue, jl_ssavalue_type);
impl_valid_layout!(SSAValue);

unsafe impl Unbox for SSAValue {
    type Output = Self;
    unsafe fn unbox(value: Value) -> SSAValue {
        SSAValue(value.unwrap(Private).cast::<isize>().read())
    }
}
