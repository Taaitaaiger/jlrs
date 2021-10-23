use jlrs::prelude::*;
mod util;
use util::JULIA;

#[test]
fn call_no_kw() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(2, |global, frame| unsafe {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("funcwithkw")?
                .wrapper_unchecked();

            let v = func
                .call(&mut *frame, &mut [a_value])?
                .unwrap()
                .unbox::<isize>()?;

            assert_eq!(v, 2);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call_with_kw() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(4, |global, frame| unsafe {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let b_value = Value::new(&mut *frame, 10isize)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("funcwithkw")?
                .wrapper_unchecked();

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)?
                .call1(&mut *frame, a_value)?
                .unwrap()
                .unbox::<isize>()?;

            assert_eq!(v, 11);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call_with_kw_and_1_vararg() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(5, |global, frame| unsafe {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let b_value = Value::new(&mut *frame, 10isize)?;
            let c_value = Value::new(&mut *frame, 5isize)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("funcwithkw")?
                .wrapper_unchecked();

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)?
                .call2(&mut *frame, a_value, c_value)?
                .unwrap()
                .unbox::<isize>()?;

            assert_eq!(v, 16);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call_with_kw_and_2_vararg() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(6, |global, frame| unsafe {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let b_value = Value::new(&mut *frame, 10isize)?;
            let c_value = Value::new(&mut *frame, 5isize)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("funcwithkw")?
                .wrapper_unchecked();

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)?
                .call2(&mut *frame, a_value, c_value)?
                .unwrap()
                .unbox::<isize>()?;

            assert_eq!(v, 16);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call_with_kw_and_3_vararg() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(7, |global, frame| unsafe {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let b_value = Value::new(&mut *frame, 10isize)?;
            let c_value = Value::new(&mut *frame, 5isize)?;
            let d_value = Value::new(&mut *frame, 4isize)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("funcwithkw")?
                .wrapper_unchecked();

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)?
                .call3(&mut *frame, a_value, c_value, d_value)?
                .unwrap()
                .unbox::<isize>()?;

            assert_eq!(v, 20);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call_with_kw_and_4_vararg() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(8, |global, frame| unsafe {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let b_value = Value::new(&mut *frame, 10isize)?;
            let c_value = Value::new(&mut *frame, 5isize)?;
            let d_value = Value::new(&mut *frame, 4isize)?;
            let e_value = Value::new(&mut *frame, 2isize)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("funcwithkw")?
                .wrapper_unchecked();

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)?
                .call(&mut *frame, &mut [a_value, c_value, d_value, e_value])?
                .unwrap()
                .unbox::<isize>()?;

            assert_eq!(v, 22);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call_with_abstract_kw_f32() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(4, |global, frame| unsafe {
            let a_value = Value::new(&mut *frame, 1f32)?;
            let b_value = Value::new(&mut *frame, 10f32)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("funcwithabstractkw")?
                .wrapper_unchecked();

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)?
                .call1(&mut *frame, a_value)?
                .unwrap()
                .unbox::<f32>()?;

            assert_eq!(v, 11.0f32);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call_with_abstract_kw_f64() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(4, |global, frame| unsafe {
            let a_value = Value::new(&mut *frame, 1f32)?;
            let b_value = Value::new(&mut *frame, 10f64)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("funcwithabstractkw")?
                .wrapper_unchecked();

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)?
                .call1(&mut *frame, a_value)?
                .unwrap()
                .unbox::<f64>()?;

            assert_eq!(v, 11.0f64);
            Ok(())
        })
        .unwrap();
    });
}
