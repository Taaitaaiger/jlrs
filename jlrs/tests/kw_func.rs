use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn call_no_kw() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(2, |global, frame| {
            let a_value = Value::new(frame, 1isize)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithkw")?;

            let v = func.call(frame, &mut [a_value])?.unwrap().cast::<isize>()?;

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
            let a_value = Value::new(frame, 1isize)?;
            let b_value = Value::new(frame, 10isize)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithkw")?;

            let kw = named_tuple!(frame, "b" => b_value)?;
            let v = func
                .call_keywords(frame, &mut [kw, func, a_value])?
                .unwrap()
                .cast::<isize>()?;

            assert_eq!(v, 11);
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
            let a_value = Value::new(frame, 1f32)?;
            let b_value = Value::new(frame, 10f32)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithabstractkw")?;

            let kw = named_tuple!(frame, "b" => b_value)?;
            let v = func
                .call_keywords(frame, &mut [kw, func, a_value])?
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
            let a_value = Value::new(frame, 1f32)?;
            let b_value = Value::new(frame, 10f64)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("funcwithabstractkw")?;

            let kw = named_tuple!(frame, "b" => b_value)?;
            let v = func
                .call_keywords(frame, &mut [kw, func, a_value])?
                .unwrap()
                .cast::<f64>()?;

            assert_eq!(v, 11.0f64);
            Ok(())
        })
        .unwrap();
    });
}
