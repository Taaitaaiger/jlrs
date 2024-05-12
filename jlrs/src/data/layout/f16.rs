//! Layout type for `Float16`.
//!
//! This module is only available if the `f16` feature has been enabled.

use half::f16;
use jl_sys::jl_float16_type;

use super::is_bits::IsBits;
use crate::{
    convert::{ccall_types::CCallReturn, into_julia::IntoJulia, unbox::Unbox},
    data::managed::{
        datatype::{DataType, DataTypeData},
        private::ManagedPriv,
    },
    impl_julia_typecheck, impl_valid_layout,
    memory::target::Target,
    private::Private,
};

impl_julia_typecheck!(f16, jl_float16_type);
impl_valid_layout!(f16, jl_float16_type);

unsafe impl Unbox for f16 {
    type Output = Self;
}

unsafe impl IntoJulia for f16 {
    #[inline]
    fn julia_type<'scope, Tgt>(target: Tgt) -> DataTypeData<'scope, Tgt>
    where
        Tgt: Target<'scope>,
    {
        let dt = DataType::float16_type(&target);
        unsafe { target.data_from_ptr(dt.unwrap_non_null(Private), Private) }
    }
}

impl_ccall_arg!(f16);

unsafe impl CCallReturn for f16 {
    type FunctionReturnType = Self;
    type CCallReturnType = Self;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}

impl_construct_julia_type!(f16, jl_float16_type);

unsafe impl IsBits for f16 {}
