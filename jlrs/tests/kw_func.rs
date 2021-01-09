use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn call_no_kw() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(2, |global, frame| {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithkw")?;

            let v = func
                .call(&mut *frame, &mut [a_value])?
                .unwrap()
                .cast::<isize>()?;

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

        jlrs.frame(4, |global, frame| {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let b_value = Value::new(&mut *frame, 10isize)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithkw")?;

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)
                .call1(&mut *frame, a_value)?
                .unwrap()
                .cast::<isize>()?;

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

        jlrs.frame(5, |global, frame| {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let b_value = Value::new(&mut *frame, 10isize)?;
            let c_value = Value::new(&mut *frame, 5isize)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithkw")?;

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)
                .call2(&mut *frame, a_value, c_value)?
                .unwrap()
                .cast::<isize>()?;

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

        jlrs.frame(6, |global, frame| {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let b_value = Value::new(&mut *frame, 10isize)?;
            let c_value = Value::new(&mut *frame, 5isize)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithkw")?;

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)
                .call2(&mut *frame, a_value, c_value)?
                .unwrap()
                .cast::<isize>()?;

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

        jlrs.frame(7, |global, frame| {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let b_value = Value::new(&mut *frame, 10isize)?;
            let c_value = Value::new(&mut *frame, 5isize)?;
            let d_value = Value::new(&mut *frame, 4isize)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithkw")?;

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)
                .call3(&mut *frame, a_value, c_value, d_value)?
                .unwrap()
                .cast::<isize>()?;

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

        jlrs.frame(8, |global, frame| {
            let a_value = Value::new(&mut *frame, 1isize)?;
            let b_value = Value::new(&mut *frame, 10isize)?;
            let c_value = Value::new(&mut *frame, 5isize)?;
            let d_value = Value::new(&mut *frame, 4isize)?;
            let e_value = Value::new(&mut *frame, 2isize)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithkw")?;

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)
                .call(&mut *frame, &mut [a_value, c_value, d_value, e_value])?
                .unwrap()
                .cast::<isize>()?;

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

        jlrs.frame(4, |global, frame| {
            let a_value = Value::new(&mut *frame, 1f32)?;
            let b_value = Value::new(&mut *frame, 10f32)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithabstractkw")?;

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)
                .call1(&mut *frame, a_value)?
                .unwrap()
                .cast::<f32>()?;

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

        jlrs.frame(4, |global, frame| {
            let a_value = Value::new(&mut *frame, 1f32)?;
            let b_value = Value::new(&mut *frame, 10f64)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithabstractkw")?;

            let kw = named_tuple!(&mut *frame, "b" => b_value)?;
            let v = func
                .with_keywords(kw)
                .call1(&mut *frame, a_value)?
                .unwrap()
                .cast::<f64>()?;

            assert_eq!(v, 11.0f64);
            Ok(())
        })
        .unwrap();
    });
}
