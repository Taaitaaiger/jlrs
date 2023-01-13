//! Layout type for `Nothing`.

use jl_sys::jl_nothing_type;

use crate::{
    convert::{construct_type::ConstructType, into_julia::IntoJulia, unbox::Unbox},
    data::managed::{
        datatype::{DataType, DataTypeData},
        Managed,
    },
    impl_julia_typecheck, impl_valid_layout,
    memory::target::{ExtendedTarget, Target},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Nothing;

impl_julia_typecheck!(Nothing, jl_nothing_type);
impl_valid_layout!(Nothing);

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

unsafe impl ConstructType for Nothing {
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
        <Nothing as crate::convert::into_julia::IntoJulia>::julia_type(target)
    }
}
