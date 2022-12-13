//! Layout type for `Float16`.
//!
//! This module is only available if the `f16` feature has been enabled.

use half::f16;
use jl_sys::jl_float16_type;

use crate::{
    convert::{into_julia::IntoJulia, unbox::Unbox},
    data::managed::{
        datatype::{DataType, DataTypeData},
        private::ManagedPriv,
    },
    impl_julia_typecheck, impl_valid_layout,
    memory::target::Target,
    private::Private,
};

impl_julia_typecheck!(f16, jl_float16_type);
impl_valid_layout!(f16);

unsafe impl Unbox for f16 {
    type Output = Self;
}

unsafe impl IntoJulia for f16 {
    fn julia_type<'scope, T>(target: T) -> DataTypeData<'scope, T>
    where
        T: Target<'scope>,
    {
        let dt = DataType::float16_type(&target);
        unsafe { target.data_from_ptr(dt.unwrap_non_null(Private), Private) }
    }
}
