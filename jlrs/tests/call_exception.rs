mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn call0_exception_is_caught() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "throws_exception")?
                        .as_managed();

                    let res = func.call0(&mut frame);
                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call0_kw_exception_is_caught() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let kw = named_tuple!(&mut frame, "a" => arg);

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "throws_exception")?
                        .as_managed()
                        .provide_keywords(kw)?;

                    let res = func.call0(&mut frame);
                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call1_exception_is_caught() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "throws_exception")?
                        .as_managed();

                    let res = func.call1(&mut frame, arg);
                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call1_kw_exception_is_caught() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let kw = named_tuple!(&mut frame, "a" => arg);

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "throws_exception")?
                        .as_managed()
                        .provide_keywords(kw)?;

                    let res = func.call1(&mut frame, arg);
                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call2_exception_is_caught() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "throws_exception")?
                        .as_managed();

                    let res = func.call2(&mut frame, arg, arg);
                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call2_kw_exception_is_caught() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let kw = named_tuple!(&mut frame, "a" => arg);

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "throws_exception")?
                        .as_managed()
                        .provide_keywords(kw)?;

                    let res = func.call2(&mut frame, arg, arg);
                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call3_exception_is_caught() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "throws_exception")?
                        .as_managed();

                    let res = func.call3(&mut frame, arg, arg, arg);
                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call3_kw_exception_is_caught() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let kw = named_tuple!(&mut frame, "a" => arg);

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "throws_exception")?
                        .as_managed()
                        .provide_keywords(kw)?;

                    let res = func.call3(&mut frame, arg, arg, arg);
                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_exception_is_caught() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "throws_exception")?
                        .as_managed();

                    let res = func.call(&mut frame, []);
                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_kw_exception_is_caught() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let kw = named_tuple!(&mut frame, "a" => arg);

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "throws_exception")?
                        .as_managed()
                        .provide_keywords(kw)?;

                    let res = func.call(&mut frame, []);
                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn method_error_is_caught() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let not_a_func = Value::new(&mut frame, 1usize);
                    let res = not_a_func.call0(&mut frame);
                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn call_exception_tests() {
        call0_exception_is_caught();
        call0_kw_exception_is_caught();
        call1_exception_is_caught();
        call1_kw_exception_is_caught();
        call2_exception_is_caught();
        call2_kw_exception_is_caught();
        call3_exception_is_caught();
        call3_kw_exception_is_caught();
        call_exception_is_caught();
        call_kw_exception_is_caught();
        method_error_is_caught();
    }
}
