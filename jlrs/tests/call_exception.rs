mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{named_tuple, prelude::*};

    use super::util::JULIA;

    fn call0_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_managed();

                    let res = func.call(&mut frame, []);
                    assert!(res.is_err());
                })
            });
        });
    }

    fn call0_kw_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let kw = named_tuple!(&mut frame, "a" => arg).unwrap();

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_managed()
                        .provide_keywords(kw);

                    let res = func.call(&mut frame, []);
                    assert!(res.is_err());
                })
            });
        });
    }

    fn call1_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_managed();

                    let res = func.call(&mut frame, [arg]);
                    assert!(res.is_err());
                })
            });
        });
    }

    fn call1_kw_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let kw = named_tuple!(&mut frame, "a" => arg).unwrap();

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_managed()
                        .provide_keywords(kw);

                    let res = func.call(&mut frame, [arg]);
                    assert!(res.is_err());
                })
            });
        });
    }

    fn call2_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_value();

                    let res = func.call(&mut frame, [arg, arg]);
                    assert!(res.is_err());
                })
            });
        });
    }

    fn call2_kw_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let kw = named_tuple!(&mut frame, "a" => arg).unwrap();

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_managed()
                        .provide_keywords(kw);

                    let res = func.call(&mut frame, [arg, arg]);
                    assert!(res.is_err());
                })
            });
        });
    }

    fn call3_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_managed();

                    let res = func.call(&mut frame, [arg, arg, arg]);
                    assert!(res.is_err());
                })
            });
        });
    }

    fn call3_kw_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let kw = named_tuple!(&mut frame, "a" => arg).unwrap();

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_managed()
                        .provide_keywords(kw);

                    let res = func.call(&mut frame, [arg, arg, arg]);
                    assert!(res.is_err());
                })
            });
        });
    }

    fn call_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_managed();

                    let res = func.call(&mut frame, []);
                    assert!(res.is_err());
                })
            });
        });
    }

    fn call_kw_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let arg = Value::new(&mut frame, 1usize);
                    let kw = named_tuple!(&mut frame, "a" => arg).unwrap();

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_managed()
                        .provide_keywords(kw);

                    let res = func.call(&mut frame, []);
                    assert!(res.is_err());
                })
            });
        });
    }

    fn method_error_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let not_a_func = Value::new(&mut frame, 1usize);
                    let res = not_a_func.call(&mut frame, []);
                    assert!(res.is_err());
                })
            });
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
