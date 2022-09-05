mod util;
#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn call0_exception_is_caught() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, mut frame| unsafe {
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();

                let res = func.call0(&mut frame)?;
                assert!(res.is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call0_kw_exception_is_caught() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let arg = Value::new(&mut frame, 1usize)?;
                let kw = named_tuple!(&mut frame, "a" => arg)?;

                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked()
                    .provide_keywords(kw)?;

                let res = func.call0(&mut frame)?;
                assert!(res.is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call1_exception_is_caught() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let arg = Value::new(&mut frame, 1usize)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();

                let res = func.call1(&mut frame, arg)?;
                assert!(res.is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call1_kw_exception_is_caught() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let arg = Value::new(&mut frame, 1usize)?;
                let kw = named_tuple!(&mut frame, "a" => arg)?;

                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked()
                    .provide_keywords(kw)?;

                let res = func.call1(&mut frame, arg)?;
                assert!(res.is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call2_exception_is_caught() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let arg = Value::new(&mut frame, 1usize)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();

                let res = func.call2(&mut frame, arg, arg)?;
                assert!(res.is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call2_kw_exception_is_caught() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let arg = Value::new(&mut frame, 1usize)?;
                let kw = named_tuple!(&mut frame, "a" => arg)?;

                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked()
                    .provide_keywords(kw)?;

                let res = func.call2(&mut frame, arg, arg)?;
                assert!(res.is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call3_exception_is_caught() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let arg = Value::new(&mut frame, 1usize)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();

                let res = func.call3(&mut frame, arg, arg, arg)?;
                assert!(res.is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call3_kw_exception_is_caught() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let arg = Value::new(&mut frame, 1usize)?;
                let kw = named_tuple!(&mut frame, "a" => arg)?;

                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked()
                    .provide_keywords(kw)?;

                let res = func.call3(&mut frame, arg, arg, arg)?;
                assert!(res.is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call_exception_is_caught() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked();

                let res = func.call(&mut frame, [])?;
                assert!(res.is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn call_kw_exception_is_caught() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let arg = Value::new(&mut frame, 1usize)?;
                let kw = named_tuple!(&mut frame, "a" => arg)?;

                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("throws_exception")?
                    .wrapper_unchecked()
                    .provide_keywords(kw)?;

                let res = func.call(&mut frame, [])?;
                assert!(res.is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn method_error_is_caught() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_global, mut frame| unsafe {
                let not_a_func = Value::new(&mut frame, 1usize)?;
                let res = not_a_func.call0(&mut frame)?;
                assert!(res.is_err());
                Ok(())
            })
            .unwrap();
        });
    }
}
