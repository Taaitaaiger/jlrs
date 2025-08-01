mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use std::borrow::Cow;

    use jlrs::prelude::*;

    use crate::util::JULIA;

    fn core_module() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let module = Module::core(&frame);
                        let func = module.function(&mut frame, "isa");
                        let int64 = module.global(&mut frame, "Float64");
                        assert!(func.is_ok());
                        assert!(int64.is_ok());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn core_module_dynamic() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let module = Module::core(&frame);
                        let func = module.function(&mut frame, "isa");
                        assert!(func.is_ok());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn base_module() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let module = Module::base(&frame);
                        let func = module.function(&mut frame, "+");
                        let int64 = module.global(&mut frame, "pi");
                        assert!(func.is_ok());
                        assert!(int64.is_ok());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn base_module_dynamic() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let module = Module::base(&frame);
                        let func = module.function(&mut frame, "+");
                        assert!(func.is_ok());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn main_module() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let jlrs_module = Module::jlrs_core(&frame);
                        let func = jlrs_module.function(&mut frame, "valuestring");
                        assert!(func.is_ok());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn main_module_dynamic() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let jlrs_module = Module::jlrs_core(&frame);
                        let func = jlrs_module.function(&mut frame, "valuestring");
                        assert!(func.is_ok());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn jlrs_module() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let jlrs_module = Module::package_root_module(&frame, "JlrsCore");
                        assert!(jlrs_module.is_some());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn error_nonexistent_function() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        assert!(Module::base(&frame).function(&mut frame, "foo").is_err());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn error_nonexistent_function_dynamic() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        assert!(Module::base(&frame).function(&mut frame, "foo").is_err());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn error_nonexistent_submodule() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        assert!(Module::base(&frame).submodule(&mut frame, "foo").is_err());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn error_nonexistent_submodule_dynamic() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        assert!(Module::base(&frame).submodule(&mut frame, "foo").is_err());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn function_returns_module() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| unsafe {
                        let base = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .as_managed()
                            .function(&frame, "base")?
                            .as_managed();
                        let base_val = base.call0(&mut frame).unwrap();

                        assert!(base_val.is::<Module>());
                        assert!(base_val.cast::<Module>().is_ok());
                        assert!(base_val.cast::<Symbol>().is_err());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn use_string_for_access() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        assert!(Module::main(&frame)
                            .submodule(&mut frame, "JlrsTests".to_string())
                            .is_ok());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn use_cow_for_access() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        assert!(Module::main(&frame)
                            .submodule(&mut frame, Cow::from("JlrsTests"))
                            .is_ok());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    struct MyString(String);
    impl AsRef<str> for MyString {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    fn use_dyn_str_for_access() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let name = MyString("JlrsTests".to_string());
                        assert!(Module::main(&frame)
                            .submodule(&mut frame, &name as &dyn AsRef<str>)
                            .is_ok());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn set_const() {
        JULIA.with(|handle| {
            handle.borrow().error_color(true);
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| unsafe {
                        let main = Module::main(&frame);
                        let value = Value::new(&mut frame, 2usize);
                        main.set_const(&mut frame, "ONE", value)
                            .into_jlrs_result()?;

                        let value = main.global(&frame, "ONE")?.as_managed();
                        assert_eq!(value.unbox::<usize>()?, 2);
                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn eval_using() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let res = stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| unsafe {
                        assert!(Module::main(&frame)
                            .global(&mut frame, "Hermitian")
                            .is_err());
                        Value::eval_string(&mut frame, "using LinearAlgebra: Hermitian").unwrap();
                        assert!(Module::main(&frame).global(&mut frame, "Hermitian").is_ok());

                        Ok(())
                    });

                assert!(res.is_ok());
            })
        })
    }

    fn module_parent() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let res = stack.returning::<JlrsResult<_>>().scope(|frame| {
                    let main = Module::main(&frame);
                    assert_eq!(main, main.parent());

                    Ok(())
                });

                assert!(res.is_ok());
            })
        })
    }

    fn extend_lifetime_with_root() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let res = stack.returning::<JlrsResult<_>>().scope(|mut frame| {
                    let output = frame.output();

                    frame.scope(|frame| {
                        let inner_global = frame.unrooted();
                        Module::main(&inner_global).root(output)
                    });

                    Ok(())
                });

                assert!(res.is_ok());
            })
        })
    }

    fn submodule_must_be_module() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let res = stack.returning::<JlrsResult<_>>().scope(|mut frame| {
                    let submod = Module::main(&frame).submodule(&mut frame, "+");
                    assert!(submod.is_err());

                    Ok(())
                });

                assert!(res.is_ok());
            })
        })
    }

    fn set_const_unchecked() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let res = stack.returning::<JlrsResult<_>>().scope(|mut frame| {
                    let main = Module::base(&frame);

                    // TODO: Uncommenting this line breaks the test on Julia 1.12.0-rc1
                    // assert!(main.global(&mut frame, "BAR").is_err());

                    let value = Value::new(&mut frame, 1usize);
                    unsafe { main.set_const_unchecked("BAR", value) };

                    assert_eq!(value, main.global(&mut frame, "BAR")?);

                    Ok(())
                });

                assert!(res.is_ok());
            })
        })
    }

    fn function_must_be_function() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let res = stack.returning::<JlrsResult<_>>().scope(|mut frame| {
                    let main = Module::base(&frame);

                    let value = Value::new(&mut frame, 1usize);
                    unsafe { main.set_const_unchecked("BAZ", value) };

                    assert!(main.function(&mut frame, "BAZ").is_err());
                    Ok(())
                });

                assert!(res.is_ok());
            })
        })
    }

    #[test]
    fn module_tests() {
        core_module();
        core_module_dynamic();
        base_module();
        base_module_dynamic();
        main_module();
        main_module_dynamic();
        jlrs_module();
        error_nonexistent_function();
        error_nonexistent_function_dynamic();
        error_nonexistent_submodule();
        error_nonexistent_submodule_dynamic();
        function_returns_module();
        use_string_for_access();
        use_cow_for_access();
        use_dyn_str_for_access();
        set_const();
        eval_using();
        module_parent();
        extend_lifetime_with_root();
        submodule_must_be_module();
        set_const_unchecked();
        function_must_be_function();
    }
}
