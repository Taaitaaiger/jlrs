//! Layout type for `Nothing`.

use jl_sys::jl_nothing_type;

use super::is_bits::IsBits;
use crate::{
    convert::{ccall_types::CCallReturn, into_julia::IntoJulia, unbox::Unbox},
    data::managed::{
        datatype::{DataType, DataTypeData},
        Managed,
    },
    impl_julia_typecheck, impl_valid_layout,
    memory::target::Target,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Nothing;

impl_julia_typecheck!(Nothing, jl_nothing_type);
impl_valid_layout!(Nothing, jl_nothing_type);

unsafe impl Unbox for Nothing {
    type Output = Self;
}

unsafe impl IntoJulia for Nothing {
    #[inline]
    fn julia_type<'scope, Tgt>(target: Tgt) -> DataTypeData<'scope, Tgt>
    where
        Tgt: Target<'scope>,
    {
        DataType::nothing_type(&target).root(target)
    }
}

unsafe impl CCallReturn for Nothing {
    type FunctionReturnType = Nothing;
    type CCallReturnType = Nothing;
    type ReturnAs = Nothing;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}

impl_construct_julia_type!(Nothing, jl_nothing_type);

unsafe impl IsBits for Nothing {}
