#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use crate::util::JULIA;
    use jlrs::{memory::gc::Gc, prelude::*};
    use std::borrow::Cow;

    #[test]
    fn core_module() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                let module = Module::core(global);
                let func = module.function(&mut frame, "isa");
                let int64 = module.global(&mut frame, "Float64");
                assert!(func.is_ok());
                assert!(int64.is_ok());
                Ok(())
            })
            .unwrap()
        });
    }

    #[test]
    fn core_module_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                let module = Module::core(global);
                let func = module.function(&mut frame, "isa");
                assert!(func.is_ok());
                Ok(())
            })
            .unwrap()
        });
    }

    #[test]
    fn base_module() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                let module = Module::base(global);
                let func = module.function(&mut frame, "+");
                let int64 = module.global(&mut frame, "pi");
                assert!(func.is_ok());
                assert!(int64.is_ok());
                Ok(())
            })
            .unwrap()
        });
    }

    #[test]
    fn base_module_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                let module = Module::base(global);
                let func = module.function(&mut frame, "+");
                assert!(func.is_ok());
                Ok(())
            })
            .unwrap()
        });
    }

    #[test]
    fn main_module() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                let main_module = Module::main(global);
                let jlrs_module = main_module.submodule(&mut frame, "Jlrs");
                assert!(jlrs_module.is_ok());
                let func = jlrs_module.unwrap().function(&mut frame, "valuestring");
                assert!(func.is_ok());
                Ok(())
            })
            .unwrap()
        });
    }

    #[test]
    fn main_module_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                let main_module = Module::main(global);
                let jlrs_module = main_module.submodule(&mut frame, "Jlrs");
                assert!(jlrs_module.is_ok());
                let func = jlrs_module.unwrap().function(&mut frame, "valuestring");
                assert!(func.is_ok());
                Ok(())
            })
            .unwrap()
        });
    }

    #[test]
    fn error_nonexistent_function() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                assert!(Module::base(global).function(&mut frame, "foo").is_err());
                Ok(())
            })
            .unwrap()
        });
    }

    #[test]
    fn error_nonexistent_function_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                assert!(Module::base(global).function(&mut frame, "foo").is_err());
                Ok(())
            })
            .unwrap()
        });
    }

    #[test]
    fn error_nonexistent_submodule() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                assert!(Module::base(global).submodule(&mut frame, "foo").is_err());
                Ok(())
            })
            .unwrap()
        });
    }

    #[test]
    fn error_nonexistent_submodule_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| {
                assert!(Module::base(global).submodule(&mut frame, "foo").is_err());
                Ok(())
            })
            .unwrap()
        });
    }

    #[test]
    fn function_returns_module() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| unsafe {
                let base = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("base")?
                    .wrapper_unchecked();
                let base_val = base.call0(&mut frame).unwrap();

                assert!(base_val.is::<Module>());
                assert!(base_val.cast::<Module>().is_ok());
                assert!(base_val.cast::<Symbol>().is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn use_string_for_access() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                assert!(Module::main(global)
                    .submodule(&mut frame, "JlrsTests".to_string())
                    .is_ok());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn use_cow_for_access() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                assert!(Module::main(global)
                    .submodule(&mut frame, Cow::from("JlrsTests"))
                    .is_ok());

                Ok(())
            })
            .unwrap();
        })
    }

    struct MyString(String);
    impl AsRef<str> for MyString {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    #[test]
    fn use_dyn_str_for_access() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let name = MyString("JlrsTests".to_string());
                assert!(Module::main(global)
                    .submodule(&mut frame, &name as &dyn AsRef<str>)
                    .is_ok());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn set_global() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| unsafe {
                let main = Module::main(global);
                let value = Value::new(&mut frame, 1usize);

                main.set_global(&mut frame, "one", value)
                    .into_jlrs_result()?;

                let value = main.global_ref("one")?.wrapper_unchecked();
                assert_eq!(value.unbox::<usize>()?, 1);
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn set_const() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| unsafe {
                let main = Module::main(global);
                let value = Value::new(&mut frame, 2usize);
                main.set_const(&mut frame, "ONE", value)
                    .into_jlrs_result()?;

                let value = main.global_ref("ONE")?.wrapper_unchecked();
                assert_eq!(value.unbox::<usize>()?, 2);
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn set_const_twice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.error_color(true).unwrap();
            jlrs.error_color(false).unwrap();
            let err = jlrs.scope(|global, mut frame| {
                let main = Module::main(global);
                let value1 = Value::new(&mut frame, 3usize);
                let value2 = Value::new(&mut frame, 4usize);
                main.set_const_unrooted(global, "TWICE", value1)
                    .map_err(|v| unsafe { v.value_unchecked() })
                    .into_jlrs_result()?;
                main.set_const(&mut frame, "TWICE", value2)
                    .into_jlrs_result()?;
                Ok(())
            });

            assert!(err.is_err());
        })
    }

    #[test]
    fn eval_using() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let res = jlrs.scope(|global, mut frame| unsafe {
                assert!(Module::main(global)
                    .global(&mut frame, "Hermitian")
                    .is_err());
                Value::eval_string(&mut frame, "using LinearAlgebra: Hermitian").unwrap();
                assert!(Module::main(global).global(&mut frame, "Hermitian").is_ok());

                Ok(())
            });

            assert!(res.is_ok());
        })
    }

    #[test]
    fn module_parent() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let res = jlrs.scope(|global, mut frame| {
                let main = Module::main(global);
                assert_eq!(main, main.parent(&mut frame)?);

                Ok(())
            });

            assert!(res.is_ok());
        })
    }

    #[test]
    fn extend_lifetime() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let res = jlrs.scope(|global, mut frame| {
                frame
                    .scope(|mut frame| {
                        let inner_global = frame.as_scope().global();
                        let main = Module::main(inner_global);
                        unsafe { Ok(main.extend(global)) }
                    })
                    .unwrap();

                Ok(())
            });

            assert!(res.is_ok());
        })
    }

    #[test]
    fn extend_lifetime_with_root() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let res = jlrs.scope(|_, mut frame| {
                let output = frame.output();

                frame
                    .scope(|mut frame| {
                        let inner_global = frame.as_scope().global();
                        Ok(Module::main(inner_global).root(output))
                    })
                    .unwrap();

                Ok(())
            });

            assert!(res.is_ok());
        })
    }

    #[test]
    fn is_imported() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let res = jlrs.scope(|global, mut frame| {
                let main = Module::main(global);
                assert!(!main.is_imported("+"));
                unsafe {
                    Value::eval_string(&mut frame, "import Base: +").into_jlrs_result()?;
                }
                assert!(main.is_imported("+"));

                Ok(())
            });

            assert!(res.is_ok());
        })
    }

    #[test]
    fn submodule_must_be_module() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let res = jlrs.scope(|global, mut frame| {
                let submod = Module::main(global).submodule(&mut frame, "+");
                assert!(submod.is_err());

                Ok(())
            });

            assert!(res.is_ok());
        })
    }

    #[test]
    #[cfg(not(all(feature = "lts", any(windows, feature = "windows"))))]
    fn cant_redefine_const() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let res = jlrs.scope(|global, mut frame| {
                let value = Value::new(&mut frame, 1usize);
                let main = Module::base(global);

                assert!(main.set_const(&mut frame, "pi", value).is_err());

                unsafe { assert!(main.set_global(&mut frame, "pi", value).is_err()) }

                Ok(())
            });

            assert!(res.is_ok());
        })
    }

    #[test]
    fn set_global_unchecked() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let res = jlrs.scope(|global, mut frame| {
                let main = Module::base(global);
                assert!(main.global(&mut frame, "FOO").is_err());

                let value = Value::new(&mut frame, 1usize);
                unsafe { main.set_global_unchecked("FOO", value) }

                assert_eq!(value, main.global(&mut frame, "FOO")?);

                Ok(())
            });

            assert!(res.is_ok());
        })
    }

    #[test]
    fn set_const_unchecked() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let res = jlrs.scope(|global, mut frame| {
                let main = Module::base(global);
                assert!(main.global(&mut frame, "BAR").is_err());

                let value = Value::new(&mut frame, 1usize);
                unsafe { main.set_const_unchecked("BAR", value) };

                assert_eq!(value, main.global(&mut frame, "BAR")?);

                Ok(())
            });

            assert!(res.is_ok());
        })
    }

    #[test]
    fn function_must_be_function() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let res = jlrs.scope(|global, mut frame| {
                let main = Module::base(global);

                let value = Value::new(&mut frame, 1usize);
                unsafe { main.set_const_unchecked("BAZ", value) };

                assert!(main.function(&mut frame, "BAZ").is_err());
                Ok(())
            });

            assert!(res.is_ok());
        })
    }

    #[test]
    fn global_can_be_leaked() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let leaked = jlrs
                .scope(|global, mut frame| {
                    let main = Module::base(global);

                    let value = Value::new(&mut frame, 1usize);
                    unsafe { main.set_const_unchecked("QUX", value) };
                    main.leaked_global("QUX")
                })
                .unwrap();

            jlrs.gc_collect(jlrs::memory::gc::GcCollection::Full);

            let res = jlrs.scope(|global, _| {
                let value = unsafe { leaked.as_value(global) };
                assert_eq!(value.unbox::<usize>()?, 1);
                Ok(())
            });

            assert!(res.is_ok());
        })
    }

    #[test]
    fn leaked_global_must_exist() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let res = jlrs.scope(|global, _| {
                let main = Module::base(global);
                main.leaked_global("QUx")
            });

            assert!(res.is_err());
        })
    }

    #[test]
    fn module_can_be_leaked() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let leaked = jlrs
                .scope(|global, _| Ok(Module::base(global).as_leaked()))
                .unwrap();

            let res = jlrs.scope(|global, _| {
                let value = unsafe { leaked.as_value(global) };
                assert_eq!(value, Module::base(global).clone());
                Ok(())
            });

            assert!(res.is_ok());
        })
    }
}
