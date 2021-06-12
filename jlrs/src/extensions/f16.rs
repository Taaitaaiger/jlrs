//! Wrapper for half-precision floating point numbers.

use crate::{
    convert::{into_julia::IntoJulia, unbox::Unbox},
    impl_julia_typecheck, impl_valid_layout,
    memory::global::Global,
    wrappers::ptr::{datatype::DataType, DataTypeRef, Wrapper},
};
use half::f16;
use jl_sys::jl_float16_type;

impl_julia_typecheck!(f16, jl_float16_type);
impl_valid_layout!(f16);

unsafe impl Unbox for f16 {
    type Output = Self;
}

unsafe impl IntoJulia for f16 {
    fn julia_type<'scope>(global: Global<'scope>) -> DataTypeRef<'scope> {
        DataType::float16_type(global).as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::util::JULIA;
    use half::f16;

    #[test]
    fn one_minus_one_equals_zero() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| {
                    let one = Value::new(&mut *frame, f16::ONE)?;
                    let func = Module::base(global).function(&mut *frame, "-")?;
                    let res = func
                        .call2(&mut *frame, one, one)?
                        .into_jlrs_result()?
                        .unbox::<f16>()?;

                    assert_eq!(res, f16::ZERO);
                    Ok(())
                })
                .unwrap();
        });
    }
}
