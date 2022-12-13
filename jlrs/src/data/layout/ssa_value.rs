//! Layout type for `SSAVAlue`.

use std::fmt::{Debug, Formatter, Result as FmtResult};

use jl_sys::jl_ssavalue_type;

use crate::{convert::unbox::Unbox, impl_julia_typecheck, impl_valid_layout};

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
}
