use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn load_module() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(1, |global, frame| {
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

        jlrs.scope_with_slots(1, |global, frame| {
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

        jlrs.scope_with_slots(4, |global, frame| {
            let func = Module::base(global)
                .require(&mut *frame, "LinearAlgebra")?
                .expect("Cannot load LinearAlgebra")
                .cast::<Module>()?
                .function("dot")?;

            let mut arr1 = vec![1.0f64, 2.0f64];
            let mut arr2 = vec![2.0f64, 3.0f64];

            let arr1_v = Value::borrow_array(&mut *frame, &mut arr1, 2)?;
            let arr2_v = Value::borrow_array(&mut *frame, &mut arr2, 2)?;

            let res = func
                .call2(&mut *frame, arr1_v, arr2_v)?
                .expect("Cannot call LinearAlgebra.dot")
                .cast::<f64>()?;

            assert_eq!(res, 8.0);

            Ok(())
        })
        .unwrap();
    });
}
