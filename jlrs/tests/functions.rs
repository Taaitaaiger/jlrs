mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn return_nothing() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Value::eval_string(
                        &mut frame,
                        "function x(a)::Nothing
                    @assert 3 == a;
                end",
                    )
                    .into_jlrs_result()?;
                    let v = Value::new(&mut frame, 3usize);
                    let v2 = func.call1(&mut frame, v).into_jlrs_result()?;
                    assert!(v2.is::<Nothing>());
                    Ok(())
                })
                .unwrap();
        })
    }

    fn throw_nothing() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Value::eval_string(
                        &mut frame,
                        "function y()::Nothing
                throw(nothing)
                end",
                    )
                    .into_jlrs_result()?;
                    let v = func.call0(&mut frame).unwrap_err();
                    assert!(v.is::<Nothing>());
                    Ok(())
                })
                .unwrap();
        })
    }

    fn call0() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "vect")?.as_managed();
                    func.call0(&mut frame).unwrap();
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call0_unrooted() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| unsafe {
                    Module::base(&frame)
                        .function(&frame, "vect")?
                        .as_managed()
                        .call0(&frame)
                        .unwrap();

                    Ok(())
                })
                .unwrap();
        });
    }

    fn call0_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();
                    frame
                        .returning::<JlrsResult<_>>()
                        .scope(|frame| unsafe {
                            let func = Module::base(&frame).function(&frame, "vect")?.as_managed();

                            Ok(func.call0(output))
                        })?
                        .unwrap();

                    Ok(())
                })
                .unwrap();
        });
    }

    fn call0_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "vect")?.as_managed();
                    func.call0(&mut frame).unwrap();
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call0_dynamic_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();
                    frame
                        .returning::<JlrsResult<_>>()
                        .scope(|frame| unsafe {
                            let func = Module::base(&frame).function(&frame, "vect")?.as_managed();

                            Ok(func.call0(output))
                        })?
                        .unwrap();
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call1() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "cos")?.as_managed();
                    let angle = Value::new(&mut frame, std::f32::consts::PI);
                    let out = func.call1(&mut frame, angle).unwrap();
                    out.unbox::<f32>()
                });

            assert_eq!(out.unwrap(), -1.);
        });
    }

    fn call1_unrooted() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "cos")?.as_managed();
                    let angle = Value::new(&mut frame, std::f32::consts::PI);
                    let out = func.call1(&frame, angle).unwrap();
                    out.as_managed().unbox::<f32>()
                });

            assert_eq!(out.unwrap(), -1.);
        });
    }

    fn call1_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let output = frame.output();
                    let out = frame
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| {
                            let func = Module::base(&frame).function(&frame, "cos")?.as_managed();
                            let angle = Value::new(&mut frame, std::f32::consts::PI);

                            Ok(func.call1(output, angle))
                        })?
                        .unwrap()
                        .unbox::<f32>();
                    assert_eq!(out.unwrap(), -1.);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call1_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();
                    let out = frame
                        .scope(|mut frame| unsafe {
                            let func = Module::base(&frame).function(&frame, "cos")?.as_managed();
                            let angle = Value::new(&mut frame, std::f32::consts::PI);

                            func.call1(output, angle).into_jlrs_result()
                        })?
                        .unbox::<f32>();
                    assert_eq!(out.unwrap(), -1.);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call1_dynamic_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();
                    let out = frame
                        .scope(|mut frame| unsafe {
                            let func = Module::base(&frame).function(&frame, "cos")?.as_managed();
                            let angle = Value::new(&mut frame, std::f32::consts::PI);

                            func.call1(output, angle).into_jlrs_result()
                        })?
                        .unbox::<f32>();
                    assert_eq!(out.unwrap(), -1.);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call2() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                    let arg0 = Value::new(&mut frame, 1u32);
                    let arg1 = Value::new(&mut frame, 2u32);
                    let out = func.call2(&mut frame, arg0, arg1).unwrap();
                    out.unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 3);
        });
    }

    fn call2_unrooted() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                    let arg0 = Value::new(&mut frame, 1u32);
                    let arg1 = Value::new(&mut frame, 2u32);
                    let out = func.call2(&frame, arg0, arg1).unwrap();
                    out.as_managed().unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 3);
        });
    }

    fn call_multiple_scopes() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let arg0 = Value::new(&mut frame, 1u32);

                    let output = frame.output();
                    frame
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| {
                            let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                            let arg1 = Value::new(&mut frame, 2u32);
                            Ok(func.call(output, [arg0, arg1]))
                        })?
                        .into_jlrs_result()?
                        .unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 3);
        });
    }

    fn call2_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();
                    frame
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| unsafe {
                            let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                            let arg0 = Value::new(&mut frame, 1u32);
                            let arg1 = Value::new(&mut frame, 2u32);

                            Ok(func.call2(output, arg0, arg1))
                        })?
                        .unwrap()
                        .unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 3);
        });
    }

    fn call2_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                    let arg0 = Value::new(&mut frame, 1u32);
                    let arg1 = Value::new(&mut frame, 2u32);
                    let out = func.call2(&mut frame, arg0, arg1).unwrap();
                    out.unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 3);
        });
    }

    fn call2_dynamic_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();
                    frame
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| unsafe {
                            let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                            let arg0 = Value::new(&mut frame, 1u32);
                            let arg1 = Value::new(&mut frame, 2u32);

                            Ok(func.call2(output, arg0, arg1))
                        })?
                        .unwrap()
                        .unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 3);
        });
    }

    fn call3() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                    let arg0 = Value::new(&mut frame, 1u32);
                    let arg1 = Value::new(&mut frame, 2u32);
                    let arg2 = Value::new(&mut frame, 3u32);
                    let out = func.call3(&mut frame, arg0, arg1, arg2).unwrap();
                    out.unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 6);
        });
    }

    fn call3_unrooted() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                    let arg0 = Value::new(&mut frame, 1u32);
                    let arg1 = Value::new(&mut frame, 2u32);
                    let arg2 = Value::new(&mut frame, 3u32);
                    let out = func.call3(&frame, arg0, arg1, arg2).unwrap();
                    out.as_managed().unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 6);
        });
    }

    fn call3_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();
                    frame
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| unsafe {
                            let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                            let arg0 = Value::new(&mut frame, 1u32);
                            let arg1 = Value::new(&mut frame, 2u32);
                            let arg2 = Value::new(&mut frame, 3u32);

                            Ok(func.call3(output, arg0, arg1, arg2))
                        })?
                        .unwrap()
                        .unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 6);
        });
    }

    fn call3_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                    let arg0 = Value::new(&mut frame, 1u32);
                    let arg1 = Value::new(&mut frame, 2u32);
                    let arg2 = Value::new(&mut frame, 3u32);
                    let out = func.call3(&mut frame, arg0, arg1, arg2).unwrap();
                    out.unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 6);
        });
    }

    fn call3_dynamic_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();
                    frame
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| unsafe {
                            let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                            let arg0 = Value::new(&mut frame, 1u32);
                            let arg1 = Value::new(&mut frame, 2u32);
                            let arg2 = Value::new(&mut frame, 3u32);

                            Ok(func.call3(output, arg0, arg1, arg2))
                        })?
                        .unwrap()
                        .unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 6);
        });
    }

    fn call() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                    let arg0 = Value::new(&mut frame, 1u32);
                    let arg1 = Value::new(&mut frame, 2u32);
                    let arg2 = Value::new(&mut frame, 3u32);
                    let arg3 = Value::new(&mut frame, 4u32);
                    let out = func.call(&mut frame, [arg0, arg1, arg2, arg3]).unwrap();
                    out.unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 10);
        });
    }

    fn call_unrooted() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                    let arg0 = Value::new(&mut frame, 1u32);
                    let arg1 = Value::new(&mut frame, 2u32);
                    let arg2 = Value::new(&mut frame, 3u32);
                    let arg3 = Value::new(&mut frame, 4u32);
                    let out = func.call(&frame, [arg0, arg1, arg2, arg3]).unwrap();
                    out.as_managed().unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 10);
        });
    }

    fn call_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();
                    frame
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| unsafe {
                            let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                            let arg0 = Value::new(&mut frame, 1u32);
                            let arg1 = Value::new(&mut frame, 2u32);
                            let arg2 = Value::new(&mut frame, 3u32);
                            let arg3 = Value::new(&mut frame, 4u32);

                            Ok(func.call(output, [arg0, arg1, arg2, arg3]))
                        })?
                        .unwrap()
                        .unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 10);
        });
    }

    fn call_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();
                    frame
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| unsafe {
                            let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                            let arg0 = Value::new(&mut frame, 1u32);
                            let arg1 = Value::new(&mut frame, 2u32);
                            let arg2 = Value::new(&mut frame, 3u32);
                            let arg3 = Value::new(&mut frame, 4u32);

                            Ok(func.call(output, [arg0, arg1, arg2, arg3]))
                        })?
                        .unwrap()
                        .unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 10);
        });
    }

    fn call_dynamic_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();
                    frame
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| unsafe {
                            let func = Module::base(&frame).function(&frame, "+")?.as_managed();
                            let arg0 = Value::new(&mut frame, 1u32);
                            let arg1 = Value::new(&mut frame, 2u32);
                            let arg2 = Value::new(&mut frame, 3u32);
                            let arg3 = Value::new(&mut frame, 4u32);

                            Ok(func.clone().call(output, [arg0, arg1, arg2, arg3]))
                        })?
                        .unwrap()
                        .unbox::<u32>()
                });

            assert_eq!(out.unwrap(), 10);
        });
    }

    #[test]
    fn function_tests() {
        return_nothing();
        throw_nothing();
        call0();
        call0_unrooted();
        call0_output();
        call0_dynamic();
        call0_dynamic_output();
        call1();
        call1_unrooted();
        call1_output();
        call1_dynamic();
        call1_dynamic_output();
        call2();
        call2_unrooted();
        call_multiple_scopes();
        call2_output();
        call2_dynamic();
        call2_dynamic_output();
        call3();
        call3_unrooted();
        call3_output();
        call3_dynamic();
        call3_dynamic_output();
        call();
        call_unrooted();
        call_output();
        call_dynamic();
        call_dynamic_output();
    }
}
