mod util;
#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn call0() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, mut frame| unsafe {
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();
                func.call0(&mut frame)?.into_jlrs_result()?;
                Ok(())
            })
            .unwrap_err();
        });
    }

    #[test]
    fn call0_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();
                func.call0(&mut frame)?.into_jlrs_result()?;
                Ok(())
            })
            .unwrap_err();
        });
    }

    /*#[test]
    fn call0_nested_as_unrooted() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                frame
                    .result_scope(|output, mut frame| unsafe {
                        let func = Module::main(global)
                            .submodule_ref("JlrsTests")?
                            .wrapper_unchecked()
                            .function_ref("throws_exception")?
                            .wrapper_unchecked();
                        let res = func.call0(&mut frame)?;

                        let os = output.into_scope(frame);
                        Ok(res.as_unrooted(os))
                    })?
                    .unwrap_err();

                Ok(())
            })
            .unwrap();
        });
    }*/

    #[test]
    fn call1() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(2, |global, mut frame| unsafe {
                let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();
                func.call1(&mut frame, angle)?.into_jlrs_result()?;
                Ok(())
            })
            .unwrap_err();
        });
    }

    #[test]
    fn call1_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();
                func.call1(&mut frame, angle)?.into_jlrs_result()?;
                Ok(())
            })
            .unwrap_err();
        });
    }

    #[test]
    fn call2() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(3, |global, mut frame| unsafe {
                let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();
                func.call2(&mut frame, angle, angle)?.into_jlrs_result()?;
                Ok(())
            })
            .unwrap_err();
        });
    }

    #[test]
    fn call2_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();
                func.call2(&mut frame, angle, angle)?.into_jlrs_result()?;
                Ok(())
            })
            .unwrap_err();
        });
    }

    #[test]
    fn call3() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(4, |global, mut frame| unsafe {
                let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();
                func.call3(&mut frame, angle, angle, angle)?
                    .into_jlrs_result()?;
                Ok(())
            })
            .unwrap_err();
        });
    }

    #[test]
    fn call3_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();
                func.call3(&mut frame, angle, angle, angle)?
                    .into_jlrs_result()?;
                Ok(())
            })
            .unwrap_err();
        });
    }

    #[test]
    fn call() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(5, |global, mut frame| unsafe {
                let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();
                func.call(&mut frame, &mut [angle, angle, angle])?
                    .into_jlrs_result()?;
                Ok(())
            })
            .unwrap_err();
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

            jlrs.scope(|global, mut frame| unsafe {
                let angle = Value::new(&mut frame, std::f32::consts::PI)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();
                func.call(&mut frame, &mut [angle, angle, angle])?
                    .into_jlrs_result()?;
                Ok(())
            })
            .unwrap_err();
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
                        func.call(output, &mut [arg0, arg1, arg2, arg3])
                    })?
                    .unwrap()
                    .unbox::<u32>()
            });

            assert_eq!(out.unwrap(), 10);
        });
    }
}
