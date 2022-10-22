//! Wrapper for `Nothing`.

use crate::{
    convert::{into_julia::IntoJulia, unbox::Unbox},
    impl_julia_typecheck, impl_valid_layout,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{datatype::DataType, private::WrapperPriv},
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
    fn julia_type<'scope, T>(target: T) -> T::Data
    where
        T: Target<'scope, 'static, DataType<'scope>>,
    {
        let dt = DataType::nothing_type(&target);
        unsafe { target.data_from_ptr(dt.unwrap_non_null(Private), Private) }
    }
}
