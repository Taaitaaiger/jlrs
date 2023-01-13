//! Layout type for `SSAVAlue`.

use std::fmt::{Debug, Formatter, Result as FmtResult};

use jl_sys::jl_ssavalue_type;

use crate::{
    convert::{construct_type::ConstructType, unbox::Unbox},
    data::managed::datatype::DataTypeData,
    impl_julia_typecheck, impl_valid_layout,
    memory::target::ExtendedTarget,
    prelude::{DataType, Managed, Target},
};

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

unsafe impl ConstructType for SSAValue {
    fn base_type<'target, T>(target: &T) -> crate::data::managed::value::Value<'target, 'static>
    where
        T: Target<'target>,
    {
        DataType::ssavalue_type(target).as_value()
    }

    fn construct_type<'target, 'current, 'borrow, T>(
        target: ExtendedTarget<'target, 'current, 'borrow, T>,
    ) -> DataTypeData<'target, T>
    where
        T: Target<'target>,
    {
        let (target, _) = target.split();
        DataType::ssavalue_type(&target).root(target)
    }
}

impl_ccall_arg!(SSAValue);
