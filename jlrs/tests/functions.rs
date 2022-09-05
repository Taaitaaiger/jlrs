mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn return_nothing() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(3, |_global, mut frame| unsafe {
                let func = Value::eval_string(
                    &mut frame,
                    "function x(a)::Nothing
                    @assert 3 == a;
                end",
                )?
                .into_jlrs_result()?;
                let v = Value::new(&mut frame, 3usize)?;
                let v2 = func.call1(&mut frame, v)?.into_jlrs_result()?;
                assert!(v2.is::<Nothing>());
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn throw_nothing() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(3, |_global, mut frame| unsafe {
                let func = Value::eval_string(
                    &mut frame,
                    "function y()::Nothing
                throw(nothing)
                end",
                )?
                .into_jlrs_result()?;
                let v = func.call0(&mut frame)?.unwrap_err();
                assert!(v.is::<Nothing>());
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn call0() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, mut frame| unsafe {
                let func = Module::base(global)
                    .function_ref("vect")?
                    .wrapper_unchecked();
                func.call0(&mut frame)?.unwrap();
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call0_unrooted() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, _| unsafe {
                Module::base(global)
                    .function_ref("vect")?
                    .wrapper_unchecked()
                    .call0_unrooted(global)
                    .unwrap();

                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call0_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, mut frame| {
                let (output, frame) = frame.split()?;
                frame
                    .scope_with_capacity(24, |mut frame| unsafe {
                        let func = Module::base(global)
                            .function_ref("vect")?
                            .wrapper_unchecked();
                        let output = output.into_scope(&mut frame);
                        func.call0(output)
                    })?
                    .unwrap();

                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call0_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let func = Module::base(global)
                    .function_ref("vect")?
                    .wrapper_unchecked();
                func.call0(&mut frame)?.unwrap();
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call0_dynamic_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                let (output, frame) = frame.split()?;
                frame
                    .scope_with_capacity(24, |mut frame| unsafe {
                        let func = Module::base(global)
                            .function_ref("vect")?
                            .wrapper_unchecked();
                        let output = output.into_scope(&mut frame);
                        func.call0(output)
                    })?
                    .unwrap();
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call1() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(2, |global, mut frame| unsafe {
                let func = Module::base(global)
                    .function_ref("cos")?
                    .wrapper_unchecked();
                let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                let out = func.call1(&mut frame, angle)?.unwrap();
                out.unbox::<f32>()
            });

            assert_eq!(out.unwrap(), -1.);
        });
    }

    #[test]
    fn call1_unrooted() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(2, |global, mut frame| unsafe {
                let func = Module::base(global)
                    .function_ref("cos")?
                    .wrapper_unchecked();
                let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                let out = func.call1_unrooted(global, angle).unwrap();
                out.wrapper_unchecked().unbox::<f32>()
            });

            assert_eq!(out.unwrap(), -1.);
        });
    }

    #[test]
    fn call1_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(2, |global, mut frame| unsafe {
                let (output, frame) = frame.split()?;
                let out = frame
                    .scope_with_capacity(24, |mut frame| {
                        let func = Module::base(global)
                            .function_ref("cos")?
                            .wrapper_unchecked();
                        let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                        let output = output.into_scope(&mut frame);
                        func.call1(output, angle)
                    })?
                    .unwrap()
                    .unbox::<f32>();
                assert_eq!(out.unwrap(), -1.);
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call1_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                let (output, frame) = frame.split()?;
                let out = frame
                    .scope_with_capacity(24, |mut frame| unsafe {
                        let func = Module::base(global)
                            .function_ref("cos")?
                            .wrapper_unchecked();
                        let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                        let output = output.into_scope(&mut frame);
                        func.call1(output, angle)
                    })?
                    .unwrap()
                    .unbox::<f32>();
                assert_eq!(out.unwrap(), -1.);
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call1_dynamic_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                let (output, frame) = frame.split()?;
                let out = frame
                    .scope_with_capacity(24, |mut frame| unsafe {
                        let func = Module::base(global)
                            .function_ref("cos")?
                            .wrapper_unchecked();
                        let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                        let output = output.into_scope(&mut frame);
                        func.call1(output, angle)
                    })?
                    .unwrap()
                    .unbox::<f32>();
                assert_eq!(out.unwrap(), -1.);
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call2() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(3, |global, mut frame| unsafe {
                let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                let arg0 = Value::new(&mut frame, 1u32)?;
                let arg1 = Value::new(&mut frame, 2u32)?;
                let out = func.call2(&mut frame, arg0, arg1)?.unwrap();
                out.unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 3);
        });
    }

    #[test]
    fn call2_unrooted() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(3, |global, mut frame| unsafe {
                let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                let arg0 = Value::new(&mut frame, 1u32)?;
                let arg1 = Value::new(&mut frame, 2u32)?;
                let out = func.call2_unrooted(global, arg0, arg1).unwrap();
                out.wrapper_unchecked().unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 3);
        });
    }

    #[test]
    fn call_multiple_scopes() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(3, |global, mut frame| unsafe {
                let arg0 = Value::new(&mut frame, 1u32)?;

                let output = frame.output()?;
                frame
                    .scope(|mut frame| {
                        let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                        let arg1 = Value::new(&mut frame, 2u32)?;
                        func.call(output, [arg0, arg1])
                    })?
                    .into_jlrs_result()?
                    .unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 3);
        });
    }

    #[test]
    fn call2_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(3, |global, mut frame| {
                let (output, frame) = frame.split()?;
                frame
                    .scope_with_capacity(24, |mut frame| unsafe {
                        let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                        let arg0 = Value::new(&mut frame, 1u32)?;
                        let arg1 = Value::new(&mut frame, 2u32)?;
                        let output = output.into_scope(&mut frame);
                        func.call2(output, arg0, arg1)
                    })?
                    .unwrap()
                    .unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 3);
        });
    }

    #[test]
    fn call2_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope(|global, mut frame| unsafe {
                let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                let arg0 = Value::new(&mut frame, 1u32)?;
                let arg1 = Value::new(&mut frame, 2u32)?;
                let out = func.call2(&mut frame, arg0, arg1)?.unwrap();
                out.unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 3);
        });
    }

    #[test]
    fn call2_dynamic_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope(|global, mut frame| {
                let (output, frame) = frame.split()?;
                frame
                    .scope_with_capacity(24, |mut frame| unsafe {
                        let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                        let arg0 = Value::new(&mut frame, 1u32)?;
                        let arg1 = Value::new(&mut frame, 2u32)?;
                        let output = output.into_scope(&mut frame);
                        func.call2(output, arg0, arg1)
                    })?
                    .unwrap()
                    .unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 3);
        });
    }

    #[test]
    fn call3() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(4, |global, mut frame| unsafe {
                let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                let arg0 = Value::new(&mut frame, 1u32)?;
                let arg1 = Value::new(&mut frame, 2u32)?;
                let arg2 = Value::new(&mut frame, 3u32)?;
                let out = func.call3(&mut frame, arg0, arg1, arg2)?.unwrap();
                out.unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 6);
        });
    }

    #[test]
    fn call3_unrooted() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(4, |global, mut frame| unsafe {
                let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                let arg0 = Value::new(&mut frame, 1u32)?;
                let arg1 = Value::new(&mut frame, 2u32)?;
                let arg2 = Value::new(&mut frame, 3u32)?;
                let out = func.call3_unrooted(global, arg0, arg1, arg2).unwrap();
                out.wrapper_unchecked().unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 6);
        });
    }

    #[test]
    fn call3_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(4, |global, mut frame| {
                let (output, frame) = frame.split()?;
                frame
                    .scope_with_capacity(24, |mut frame| unsafe {
                        let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                        let arg0 = Value::new(&mut frame, 1u32)?;
                        let arg1 = Value::new(&mut frame, 2u32)?;
                        let arg2 = Value::new(&mut frame, 3u32)?;
                        let output = output.into_scope(&mut frame);
                        func.call3(output, arg0, arg1, arg2)
                    })?
                    .unwrap()
                    .unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 6);
        });
    }

    #[test]
    fn call3_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope(|global, mut frame| unsafe {
                let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                let arg0 = Value::new(&mut frame, 1u32)?;
                let arg1 = Value::new(&mut frame, 2u32)?;
                let arg2 = Value::new(&mut frame, 3u32)?;
                let out = func.call3(&mut frame, arg0, arg1, arg2)?.unwrap();
                out.unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 6);
        });
    }

    #[test]
    fn call3_dynamic_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope(|global, mut frame| {
                let (output, frame) = frame.split()?;
                frame
                    .scope_with_capacity(24, |mut frame| unsafe {
                        let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                        let arg0 = Value::new(&mut frame, 1u32)?;
                        let arg1 = Value::new(&mut frame, 2u32)?;
                        let arg2 = Value::new(&mut frame, 3u32)?;
                        let output = output.into_scope(&mut frame);
                        func.call3(output, arg0, arg1, arg2)
                    })?
                    .unwrap()
                    .unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 6);
        });
    }

    #[test]
    fn call() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(5, |global, mut frame| unsafe {
                let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                let arg0 = Value::new(&mut frame, 1u32)?;
                let arg1 = Value::new(&mut frame, 2u32)?;
                let arg2 = Value::new(&mut frame, 3u32)?;
                let arg3 = Value::new(&mut frame, 4u32)?;
                let out = func
                    .call(&mut frame, &mut [arg0, arg1, arg2, arg3])?
                    .unwrap();
                out.unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 10);
        });
    }

    #[test]
    fn call_unrooted() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(5, |global, mut frame| unsafe {
                let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                let arg0 = Value::new(&mut frame, 1u32)?;
                let arg1 = Value::new(&mut frame, 2u32)?;
                let arg2 = Value::new(&mut frame, 3u32)?;
                let arg3 = Value::new(&mut frame, 4u32)?;
                let out = func
                    .call_unrooted(global, &mut [arg0, arg1, arg2, arg3])
                    .unwrap();
                out.wrapper_unchecked().unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 10);
        });
    }

    #[test]
    fn call_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(5, |global, mut frame| {
                let (output, frame) = frame.split()?;
                frame
                    .scope_with_capacity(24, |mut frame| unsafe {
                        let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                        let arg0 = Value::new(&mut frame, 1u32)?;
                        let arg1 = Value::new(&mut frame, 2u32)?;
                        let arg2 = Value::new(&mut frame, 3u32)?;
                        let arg3 = Value::new(&mut frame, 4u32)?;
                        let output = output.into_scope(&mut frame);
                        func.call(output, &mut [arg0, arg1, arg2, arg3])
                    })?
                    .unwrap()
                    .unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 10);
        });
    }

    #[test]
    fn call_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope(|global, mut frame| {
                let (output, frame) = frame.split()?;
                frame
                    .scope_with_capacity(24, |mut frame| unsafe {
                        let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                        let arg0 = Value::new(&mut frame, 1u32)?;
                        let arg1 = Value::new(&mut frame, 2u32)?;
                        let arg2 = Value::new(&mut frame, 3u32)?;
                        let arg3 = Value::new(&mut frame, 4u32)?;
                        let output = output.into_scope(&mut frame);
                        func.call(output, &mut [arg0, arg1, arg2, arg3])
                    })?
                    .unwrap()
                    .unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 10);
        });
    }

    #[test]
    fn call_dynamic_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope(|global, mut frame| {
                let (output, frame) = frame.split()?;
                frame
                    .scope_with_capacity(24, |mut frame| unsafe {
                        let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                        let arg0 = Value::new(&mut frame, 1u32)?;
                        let arg1 = Value::new(&mut frame, 2u32)?;
                        let arg2 = Value::new(&mut frame, 3u32)?;
                        let arg3 = Value::new(&mut frame, 4u32)?;
                        let output = output.into_scope(&mut frame);
                        func.clone().call(output, &mut [arg0, arg1, arg2, arg3])
                    })?
                    .unwrap()
                    .unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 10);
        });
    }
}
