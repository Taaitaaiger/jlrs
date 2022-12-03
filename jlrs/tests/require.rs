mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn load_module() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    Module::main(&frame)
                        .require(&mut frame, "LinearAlgebra")
                        .expect("Cannot load LinearAlgebra");
                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_load_nonexistent_module() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    Module::main(&frame)
                        .require(&mut frame, "LnearAlgebra")
                        .expect_err("Can load LnearAlgebra");
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_function_from_loaded_module() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame)
                        .require(&mut frame, "LinearAlgebra")
                        .expect("Cannot load LinearAlgebra")
                        .cast::<Module>()?
                        .function(&frame, "dot")?
                        .as_managed();

                    let mut arr1 = vec![1.0f64, 2.0f64];
                    let mut arr2 = vec![2.0f64, 3.0f64];

                    let arr1_v =
                        Array::from_slice_unchecked(frame.as_extended_target(), &mut arr1, 2)?;
                    let arr2_v =
                        Array::from_slice_unchecked(frame.as_extended_target(), &mut arr2, 2)?;

                    let res = func
                        .call2(&mut frame, arr1_v.as_value(), arr2_v.as_value())
                        .expect("Cannot call LinearAlgebra.dot")
                        .unbox::<f64>()?;

                    assert_eq!(res, 8.0);

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn require_tests() {
        load_module();
        cannot_load_nonexistent_module();
        call_function_from_loaded_module();
    }
}
