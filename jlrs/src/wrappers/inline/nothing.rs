//! Wrapper for `Nothing`.

use crate::{
    convert::{into_julia::IntoJulia, unbox::Unbox},
    impl_julia_typecheck, impl_valid_layout,
    memory::global::Global,
    wrappers::ptr::{datatype::DataType, DataTypeRef, Wrapper},
};
use jl_sys::jl_nothing_type;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Nothing;

impl_julia_typecheck!(Nothing, jl_nothing_type);
impl_valid_layout!(Nothing);

unsafe impl Unbox for Nothing {
    type Output = Self;
}

unsafe impl IntoJulia for Nothing {
    fn julia_type<'scope>(global: Global<'scope>) -> DataTypeRef<'scope> {
        DataType::nothing_type(global).as_ref()
    }
}
