mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{
        data::{
            layout::tuple::Tuple,
            managed::{
                simple_vector::SimpleVector, type_name::TypeName, type_var::TypeVar, union::Union,
                union_all::UnionAll,
            },
            types::typecheck::*,
        },
        prelude::*,
    };

    use crate::util::JULIA;

    fn datatype_methods() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let val = Value::new(&mut frame, 3.0f32);
                        let dt = val.datatype();

                        assert_eq!(dt.size().unwrap(), 4);
                        assert_eq!(dt.align().unwrap(), 4);
                        assert_eq!(dt.n_bits().unwrap(), 32);
                        assert_eq!(dt.n_fields().unwrap(), 0);
                        assert!(dt.is_inline_alloc());

                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn datatype_typechecks() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let val = Value::new(&mut frame, 3.0f32);
                        let dt = val.datatype();

                        assert!(!dt.is::<Tuple>());
                        assert!(!dt.is::<NamedTuple>());
                        assert!(!dt.is::<SimpleVector>());
                        assert!(!dt.is::<Mutable>());
                        assert!(dt.is::<Immutable>());
                        assert!(!dt.is::<Union>());
                        assert!(!dt.is::<TypeVar>());
                        assert!(!dt.is::<UnionAll>());
                        assert!(!dt.is::<TypeName>());
                        assert!(!dt.is::<i8>());
                        assert!(!dt.is::<i16>());
                        assert!(!dt.is::<i32>());
                        assert!(!dt.is::<i64>());
                        assert!(!dt.is::<u8>());
                        assert!(!dt.is::<u16>());
                        assert!(!dt.is::<u32>());
                        assert!(!dt.is::<u64>());
                        assert!(dt.is::<f32>());
                        assert!(!dt.is::<f64>());
                        assert!(!dt.is::<bool>());
                        assert!(!dt.is::<char>());
                        assert!(!dt.is::<Symbol>());
                        assert!(!dt.is::<Array>());
                        assert!(!dt.is::<Module>());
                        assert!(!dt.is::<String>());
                        assert!(!dt.is::<Pointer>());

                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn function_returns_datatype() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| unsafe {
                        let dt = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .as_managed()
                            .function(&frame, "datatype")?
                            .as_managed();
                        let dt_val = dt.call0(&mut frame).unwrap();

                        assert!(dt_val.is::<DataType>());
                        assert!(dt_val.cast::<DataType>().is_ok());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_has_constrained_typename() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = DataType::tvar_type(&frame);
                        let tn = dt.type_name();
                        let s = tn.name().as_string().unwrap();

                        assert_eq!(s, "TypeVar");

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_has_fieldnames() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| unsafe {
                        let dt = DataType::tvar_type(&frame);
                        {
                            let tn = dt.field_names();
                            let tn = tn.typed_data_unchecked::<Symbol>();
                            let tn = tn.as_atomic_slice().assume_immutable_non_null();

                            assert_eq!(tn[0].as_string().unwrap(), "name");
                            assert_eq!(tn[1].as_string().unwrap(), "lb");
                            assert_eq!(tn[2].as_string().unwrap(), "ub");
                        }

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_field_size() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = DataType::tvar_type(&frame);
                        let sz = dt.field_size(1).unwrap();
                        #[cfg(target_pointer_width = "64")]
                        assert_eq!(sz, 8);
                        #[cfg(target_pointer_width = "32")]
                        assert_eq!(sz, 4);

                        let sz_unchecked = unsafe { dt.field_size_unchecked(1) };
                        assert_eq!(sz, sz_unchecked);

                        assert!(dt.field_size(20).is_none());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_field_offset() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = DataType::tvar_type(&frame);
                        let sz = dt.field_offset(1).unwrap();
                        #[cfg(target_pointer_width = "64")]
                        assert_eq!(sz, 8);
                        #[cfg(target_pointer_width = "32")]
                        assert_eq!(sz, 4);

                        assert!(dt.field_offset(20).is_none());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_pointer_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = DataType::tvar_type(&frame);
                        assert!(dt.is_pointer_field(1).unwrap());
                        assert!(dt.is_pointer_field(25).is_none());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_isbits() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = DataType::tvar_type(&frame);
                        assert!(!dt.is_bits());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_supertype() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = DataType::tvar_type(&frame);
                        assert!(dt.super_type() == DataType::any_type(&frame));

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_parameters() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        assert_eq!(
                            Value::array_int32_type(&frame)
                                .cast::<DataType>()
                                .unwrap()
                                .parameters()
                                .len(),
                            2
                        );

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_instance() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        assert!(Value::array_int32_type(&frame)
                            .cast::<DataType>()
                            .unwrap()
                            .instance()
                            .is_none());

                        assert!(DataType::nothing_type(&frame).instance().is_some());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_abstract() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = DataType::tvar_type(&frame);
                        assert!(!dt.is_abstract());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_mutable() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = DataType::tvar_type(&frame);
                        assert!(dt.mutable());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_hasfreetypevast() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = DataType::tvar_type(&frame);
                        assert!(!dt.has_free_type_vars());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_concrete() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = DataType::tvar_type(&frame);
                        assert!(dt.is_concrete_type());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_zeroinit() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = UnionAll::array_type(&frame)
                            .body()
                            .cast::<UnionAll>()?
                            .body()
                            .cast::<DataType>()?;
                        assert!(!dt.zero_init());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_params() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let dt = UnionAll::array_type(&frame).base_type();
                        assert_eq!(dt.n_parameters(), 2);
                        let param = dt.parameter(0).unwrap();
                        assert!(param.is::<TypeVar>());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_field_type() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let val = DataType::unionall_type(&frame).field_type(0).unwrap();

                        assert!(val.is::<DataType>());
                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_field_name() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let name = DataType::uniontype_type(&frame)
                            .field_name(0)
                            .unwrap()
                            .as_str()?;

                        assert_eq!(name, "a");
                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_field_name_str() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let name = DataType::uniontype_type(&frame).field_name_str(0).unwrap();

                        assert_eq!(name, "a");
                        let nonexistent = DataType::uniontype_type(&frame).field_name_str(12);
                        assert!(nonexistent.is_none());
                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_field_index_unchecked() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let idx = DataType::uniontype_type(&frame).field_index_unchecked("a");

                        assert_eq!(idx, 0);

                        let idx = DataType::uniontype_type(&frame).field_index_unchecked("c");

                        assert_eq!(idx, -1);
                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn datatype_is_const_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let ty = unsafe {
                            Module::main(&frame)
                                .submodule(&frame, "JlrsStableTests")?
                                .as_managed()
                                .global(&frame, "WithConst")?
                                .as_value()
                                .cast::<DataType>()?
                        };

                        assert!(ty.is_const_field(0).unwrap());
                        assert!(DataType::uniontype_type(&frame).is_const_field(0).unwrap());
                        assert!(!ty.clone().is_const_field(1).unwrap());
                        assert!(ty.is_const_field(2).is_none());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn cannot_instantiate_array() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let ty = TypedArray::<usize>::new(&mut frame, 1)
                            .into_jlrs_result()?
                            .as_value();

                        let instance = unsafe { ty.call(&mut frame, []) };
                        assert!(instance.is_err());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn cannot_instantiate_with_incorrect_params() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| unsafe {
                        let ty = {
                            Module::main(&frame)
                                .submodule(&frame, "JlrsTests")?
                                .as_managed()
                                .global(&frame, "WithAbstract")?
                                .as_value()
                        };

                        let instance = ty.call(&mut frame, []);
                        assert!(instance.is_err());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn compare_with_value() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let ty = DataType::bool_type(&frame);
                        assert!(ty == ty.as_value());

                        let ty2 = DataType::int32_type(&frame);
                        assert!(ty != ty2.as_value());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn extend_lifetime() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let output = frame.output();

                        frame.scope(|frame| {
                            let ty = DataType::bool_type(&frame);
                            ty.root(output)
                        });

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn check_names() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        {
                            let ty = DataType::atomicerror_type(&frame);
                            assert_eq!(ty.name(), "ConcurrencyViolationError");
                        }

                        {
                            let ty = DataType::const_type(&frame);
                            assert_eq!(ty.name(), "Const");
                        }

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    #[test]
    fn datatype_tests() {
        datatype_methods();
        datatype_typechecks();
        function_returns_datatype();
        datatype_has_constrained_typename();
        datatype_has_fieldnames();
        datatype_field_size();
        datatype_field_offset();
        datatype_pointer_field();
        datatype_isbits();
        datatype_supertype();
        datatype_parameters();
        datatype_instance();
        datatype_abstract();
        datatype_mutable();
        datatype_hasfreetypevast();
        datatype_concrete();
        datatype_zeroinit();
        datatype_params();
        datatype_field_type();
        datatype_field_name();
        datatype_field_name_str();
        datatype_field_index_unchecked();
        cannot_instantiate_array();
        datatype_is_const_field();
        cannot_instantiate_with_incorrect_params();
        compare_with_value();
        extend_lifetime();
        check_names();
    }
}
