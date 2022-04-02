mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn load_module() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, frame| unsafe {
                Module::main(global)
                    .require(&mut *frame, "LinearAlgebra")?
                    .expect("Cannot load LinearAlgebra");
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn cannot_load_nonexistent_module() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, frame| unsafe {
                Module::main(global)
                    .require(&mut *frame, "LnearAlgebra")?
                    .expect_err("Can load LnearAlgebra");
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call_function_from_loaded_module() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(4, |global, frame| unsafe {
                let func = Module::base(global)
                    .require(&mut *frame, "LinearAlgebra")?
                    .expect("Cannot load LinearAlgebra")
                    .cast::<Module>()?
                    .function_ref("dot")?
                    .wrapper_unchecked();

                let mut arr1 = vec![1.0f64, 2.0f64];
                let mut arr2 = vec![2.0f64, 3.0f64];

                let arr1_v = Array::from_slice(&mut *frame, &mut arr1, 2)?;
                let arr2_v = Array::from_slice(&mut *frame, &mut arr2, 2)?;

                let res = func
                    .call2(&mut *frame, arr1_v.as_value(), arr2_v.as_value())?
                    .expect("Cannot call LinearAlgebra.dot")
                    .unbox::<f64>()?;

                assert_eq!(res, 8.0);

                Ok(())
            })
            .unwrap();
        });
    }
}
