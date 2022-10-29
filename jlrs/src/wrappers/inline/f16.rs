//! Wrapper for `Float16`.

use crate::{
    convert::{into_julia::IntoJulia, unbox::Unbox},
    impl_julia_typecheck, impl_valid_layout,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{
        datatype::{DataType, DataTypeData},
        private::WrapperPriv,
    },
};
use half::f16;
use jl_sys::jl_float16_type;

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

#[cfg(test)]
#[cfg(feature = "sync-rt")]
mod tests {
    use crate::memory::stack_frame::StackFrame;
    use crate::prelude::*;
    use crate::util::test::JULIA;
    use half::f16;

    #[test]
    fn one_minus_one_equals_zero() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let one = Value::new(&mut frame, f16::ONE);
                    let func = Module::base(&frame).function(&mut frame, "-")?;
                    let res = func
                        .call2(&mut frame, one, one)
                        .into_jlrs_result()?
                        .unbox::<f16>()?;

                    assert_eq!(res, f16::ZERO);
                    Ok(())
                })
                .unwrap();
        });
    }
}
