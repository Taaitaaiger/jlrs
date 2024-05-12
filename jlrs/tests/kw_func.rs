mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn call_no_kw() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<()>>()
                .scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "funcwithkw")?
                        .as_managed();

                    let v = func.call(&mut frame, [a_value]).unwrap().unbox::<isize>()?;

                    assert_eq!(v, 2);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_with_kw() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "funcwithkw")?
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value);
                    let v = func
                        .provide_keywords(kw)?
                        .call1(&mut frame, a_value)
                        .unwrap()
                        .unbox::<isize>()?;

                    assert_eq!(v, 11);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_with_kw_and_no_arg() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let b_value = Value::new(&mut frame, 10isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "funcwithkw")?
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value);
                    let v = func
                        .provide_keywords(kw)?
                        .call0(&mut frame)
                        .unwrap()
                        .unbox::<isize>()?;

                    assert_eq!(v, 12);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_with_kw_and_1_arg() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "funcwithkw")?
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value);
                    let v = func
                        .provide_keywords(kw)?
                        .call1(&mut frame, a_value)
                        .unwrap()
                        .unbox::<isize>()?;

                    assert_eq!(v, 11);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_with_kw_and_1_vararg() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let c_value = Value::new(&mut frame, 5isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "funcwithkw")?
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value);
                    let v = func
                        .provide_keywords(kw)?
                        .call2(&mut frame, a_value, c_value)
                        .unwrap()
                        .unbox::<isize>()?;

                    assert_eq!(v, 16);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_with_kw_and_2_vararg() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let c_value = Value::new(&mut frame, 5isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "funcwithkw")?
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value);
                    let v = func
                        .provide_keywords(kw)?
                        .call2(&mut frame, a_value, c_value)
                        .unwrap()
                        .unbox::<isize>()?;

                    assert_eq!(v, 16);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_with_kw_and_3_vararg() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let c_value = Value::new(&mut frame, 5isize);
                    let d_value = Value::new(&mut frame, 4isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "funcwithkw")?
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value);
                    let v = func
                        .provide_keywords(kw)?
                        .call3(&mut frame, a_value, c_value, d_value)
                        .unwrap()
                        .unbox::<isize>()?;

                    assert_eq!(v, 20);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_with_kw_and_4_vararg() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let c_value = Value::new(&mut frame, 5isize);
                    let d_value = Value::new(&mut frame, 4isize);
                    let e_value = Value::new(&mut frame, 2isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "funcwithkw")?
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value);
                    let v = func
                        .provide_keywords(kw)?
                        .call(&mut frame, [a_value, c_value, d_value, e_value])
                        .unwrap()
                        .unbox::<isize>()?;

                    assert_eq!(v, 22);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_with_abstract_kw_f32() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1f32);
                    let b_value = Value::new(&mut frame, 10f32);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "funcwithabstractkw")?
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value);
                    let v = func
                        .provide_keywords(kw)?
                        .call1(&mut frame, a_value)
                        .unwrap()
                        .unbox::<f32>()?;

                    assert_eq!(v, 11.0f32);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_with_abstract_kw_f64() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1f32);
                    let b_value = Value::new(&mut frame, 10f64);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "funcwithabstractkw")?
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value);
                    let v = func
                        .provide_keywords(kw)?
                        .call1(&mut frame, a_value)
                        .unwrap()
                        .unbox::<f64>()?;

                    assert_eq!(v, 11.0f64);
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn kw_func_test() {
        call_no_kw();
        call_with_kw();
        call_with_kw_and_no_arg();
        call_with_kw_and_1_arg();
        call_with_kw_and_1_vararg();
        call_with_kw_and_2_vararg();
        call_with_kw_and_3_vararg();
        call_with_kw_and_4_vararg();
        call_with_abstract_kw_f32();
        call_with_abstract_kw_f64();
    }
}
