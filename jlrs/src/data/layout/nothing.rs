//! Layout type for `Nothing`.

use jl_sys::jl_nothing_type;

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
    fn julia_type<'scope, T>(target: T) -> DataTypeData<'scope, T>
    where
        T: Target<'scope>,
    {
        DataType::nothing_type(&target).root(target)
    }
}

unsafe impl CCallReturn for Nothing {
    type FunctionReturnType = Nothing;
    type CCallReturnType = Nothing;
}

impl_construct_julia_type!(Nothing, jl_nothing_type);
