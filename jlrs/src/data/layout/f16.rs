//! Layout type for `Float16`.
//!
//! This module is only available if the `f16` feature has been enabled.

use half::f16;
use jl_sys::jl_float16_type;

use crate::{
    convert::{construct_type::ConstructType, into_julia::IntoJulia, unbox::Unbox},
    data::managed::{
        datatype::{DataType, DataTypeData},
        private::ManagedPriv,
    },
    impl_julia_typecheck, impl_valid_layout,
    memory::target::{ExtendedTarget, Target},
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

unsafe impl ConstructType for f16 {
    fn base_type<'target, T>(target: &T) -> crate::data::managed::value::Value<'target, 'static>
    where
        T: Target<'target>,
    {
        unsafe { <Self as crate::convert::into_julia::IntoJulia>::julia_type(target).as_value() }
    }

    fn construct_type<'target, 'current, 'borrow, T>(
        target: ExtendedTarget<'target, 'current, 'borrow, T>,
    ) -> DataTypeData<'target, T>
    where
        T: Target<'target>,
    {
        let (target, _) = target.split();
        <f16 as crate::convert::into_julia::IntoJulia>::julia_type(target)
    }
}

impl_ccall_arg!(f16);
