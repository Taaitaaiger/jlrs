mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{named_tuple, prelude::*};

    use super::util::JULIA;

    fn call_no_kw() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "funcwithkw")
                        .unwrap()
                        .as_managed();

                    let v = func
                        .call(&mut frame, [a_value])
                        .unwrap()
                        .unbox::<isize>()
                        .unwrap();

                    assert_eq!(v, 2);
                })
            });
        });
    }

    fn call_with_kw() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "funcwithkw")
                        .unwrap()
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value).unwrap();
                    let v = func
                        .call_kw(&mut frame, [a_value], kw)
                        .unwrap()
                        .unbox::<isize>()
                        .unwrap();

                    assert_eq!(v, 11);
                })
            });
        });
    }

    fn call_with_kw_and_no_arg() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let b_value = Value::new(&mut frame, 10isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "funcwithkw")
                        .unwrap()
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value).unwrap();
                    let v = func
                        .call_kw(&mut frame, [], kw)
                        .unwrap()
                        .unbox::<isize>()
                        .unwrap();

                    assert_eq!(v, 12);
                })
            });
        });
    }

    fn call_with_kw_and_1_arg() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "funcwithkw")
                        .unwrap()
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value).unwrap();
                    let v = func
                        .call_kw(&mut frame, [a_value], kw)
                        .unwrap()
                        .unbox::<isize>()
                        .unwrap();

                    assert_eq!(v, 11);
                })
            });
        });
    }

    fn call_with_kw_and_1_vararg() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let c_value = Value::new(&mut frame, 5isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "funcwithkw")
                        .unwrap()
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value).unwrap();
                    let v = func
                        .call_kw(&mut frame, [a_value, c_value], kw)
                        .unwrap()
                        .unbox::<isize>()
                        .unwrap();

                    assert_eq!(v, 16);
                })
            });
        });
    }

    fn call_with_kw_and_2_vararg() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let c_value = Value::new(&mut frame, 5isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "funcwithkw")
                        .unwrap()
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value).unwrap();
                    let v = func
                        .call_kw(&mut frame, [a_value, c_value], kw)
                        .unwrap()
                        .unbox::<isize>()
                        .unwrap();

                    assert_eq!(v, 16);
                })
            });
        });
    }

    fn call_with_kw_and_3_vararg() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let c_value = Value::new(&mut frame, 5isize);
                    let d_value = Value::new(&mut frame, 4isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "funcwithkw")
                        .unwrap()
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value).unwrap();
                    let v = func
                        .call_kw(&mut frame, [a_value, c_value, d_value], kw)
                        .unwrap()
                        .unbox::<isize>()
                        .unwrap();

                    assert_eq!(v, 20);
                })
            });
        });
    }

    fn call_with_kw_and_4_vararg() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1isize);
                    let b_value = Value::new(&mut frame, 10isize);
                    let c_value = Value::new(&mut frame, 5isize);
                    let d_value = Value::new(&mut frame, 4isize);
                    let e_value = Value::new(&mut frame, 2isize);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "funcwithkw")
                        .unwrap()
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value).unwrap();
                    let v = func
                        .call_kw(&mut frame, [a_value, c_value, d_value, e_value], kw)
                        .unwrap()
                        .unbox::<isize>()
                        .unwrap();

                    assert_eq!(v, 22);
                })
            });
        });
    }

    fn call_with_abstract_kw_f32() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1f32);
                    let b_value = Value::new(&mut frame, 10f32);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "funcwithabstractkw")
                        .unwrap()
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value).unwrap();
                    let v = func
                        .call_kw(&mut frame, [a_value], kw)
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();

                    assert_eq!(v, 11.0f32);
                })
            });
        });
    }

    fn call_with_abstract_kw_f64() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let a_value = Value::new(&mut frame, 1f32);
                    let b_value = Value::new(&mut frame, 10f64);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "funcwithabstractkw")
                        .unwrap()
                        .as_managed();

                    let kw = named_tuple!(&mut frame, "b" => b_value).unwrap();
                    let v = func
                        .call_kw(&mut frame, [a_value], kw)
                        .unwrap()
                        .unbox::<f64>()
                        .unwrap();

                    assert_eq!(v, 11.0f64);
                })
            });
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
