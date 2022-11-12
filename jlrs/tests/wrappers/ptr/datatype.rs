#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use crate::util::JULIA;
    use jlrs::layout::typecheck::*;
    use jlrs::prelude::*;
    #[cfg(feature = "internal-types")]
    use jlrs::wrappers::ptr::internal::code_instance::CodeInstance;
    #[cfg(feature = "internal-types")]
    use jlrs::wrappers::ptr::internal::expr::Expr;
    #[cfg(feature = "internal-types")]
    use jlrs::wrappers::ptr::internal::method::Method;
    #[cfg(feature = "internal-types")]
    use jlrs::wrappers::ptr::internal::method_instance::MethodInstance;
    use jlrs::wrappers::ptr::type_name::TypeName;
    use jlrs::wrappers::ptr::type_var::TypeVar;
    use jlrs::wrappers::ptr::union::Union;
    use jlrs::wrappers::ptr::union_all::UnionAll;
    use jlrs::wrappers::ptr::{simple_vector::SimpleVector, symbol::SymbolRef};

    #[test]
    fn datatype_methods() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let val = Value::new(&mut frame, 3.0f32);
                    let dt = val.datatype();

                    assert_eq!(dt.size(), 4);
                    assert_eq!(dt.align(), 4);
                    assert_eq!(dt.n_bits(), 32);
                    assert_eq!(dt.n_fields(), 0);
                    assert!(dt.is_inline_alloc());

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn datatype_typechecks() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
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
                    assert!(!dt.is::<Slot>());
                    #[cfg(feature = "internal-types")]
                    assert!(!dt.is::<Expr>());
                    assert!(!dt.is::<GlobalRef>());
                    assert!(!dt.is::<GotoNode>());
                    assert!(!dt.is::<PhiNode>());
                    assert!(!dt.is::<PhiCNode>());
                    assert!(!dt.is::<UpsilonNode>());
                    assert!(!dt.is::<QuoteNode>());
                    #[cfg(feature = "internal-types")]
                    assert!(!dt.is::<LineNode>());
                    #[cfg(feature = "internal-types")]
                    assert!(!dt.is::<MethodInstance>());
                    #[cfg(feature = "internal-types")]
                    assert!(!dt.is::<CodeInstance>());
                    #[cfg(feature = "internal-types")]
                    assert!(!dt.is::<Method>());
                    assert!(!dt.is::<Module>());
                    assert!(!dt.is::<String>());
                    assert!(!dt.is::<Pointer>());
                    assert!(!dt.is::<Intrinsic>());

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn function_returns_datatype() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let dt = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .wrapper()
                        .function(&frame, "datatype")?
                        .wrapper();
                    let dt_val = dt.call0(&mut frame).unwrap();

                    assert!(dt_val.is::<DataType>());
                    assert!(dt_val.cast::<DataType>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_has_typename() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| unsafe {
                    let dt = DataType::tvar_type(&frame);
                    let tn = dt.type_name().wrapper();
                    let s = tn.name().wrapper().as_string().unwrap();

                    assert_eq!(s, "TypeVar");

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_has_fieldnames() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| unsafe {
                    let dt = DataType::tvar_type(&frame);
                    {
                        let tn = dt.field_names().unwrap().wrapper();
                        let tn = tn.typed_data::<SymbolRef>()?.as_slice();

                        assert_eq!(tn[0].unwrap().wrapper().as_string().unwrap(), "name");
                        assert_eq!(tn[1].unwrap().wrapper().as_string().unwrap(), "lb");
                        assert_eq!(tn[2].unwrap().wrapper().as_string().unwrap(), "ub");
                    }

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_field_size() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = DataType::tvar_type(&frame);
                    let sz = dt.field_size(1)?;
                    #[cfg(target_pointer_width = "64")]
                    assert_eq!(sz, 8);
                    #[cfg(target_pointer_width = "32")]
                    assert_eq!(sz, 4);

                    let sz_unchecked = unsafe { dt.field_size_unchecked(1) };
                    assert_eq!(sz, sz_unchecked);

                    assert!(dt.field_size(20).is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_field_offset() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = DataType::tvar_type(&frame);
                    let sz = dt.field_offset(1)?;
                    #[cfg(target_pointer_width = "64")]
                    assert_eq!(sz, 8);
                    #[cfg(target_pointer_width = "32")]
                    assert_eq!(sz, 4);

                    assert!(dt.field_offset(20).is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_pointer_field() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = DataType::tvar_type(&frame);
                    assert!(dt.is_pointer_field(1)?);
                    assert!(dt.is_pointer_field(25).is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_isbits() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = DataType::tvar_type(&frame);
                    assert!(!dt.is_bits());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_supertype() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| unsafe {
                    let dt = DataType::tvar_type(&frame);
                    assert!(dt.super_type().wrapper() == DataType::any_type(&frame));

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_parameters() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| unsafe {
                    assert_eq!(
                        Value::array_int32_type(&frame)
                            .cast::<DataType>()
                            .unwrap()
                            .parameters()
                            .wrapper()
                            .len(),
                        2
                    );

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_instance() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
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
    }

    #[test]
    fn datatype_hash() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = DataType::tvar_type(&frame);
                    assert!(dt.hash() != 0);

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_abstract() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = DataType::tvar_type(&frame);
                    assert!(!dt.is_abstract());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_mutable() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = DataType::tvar_type(&frame);
                    assert!(dt.mutable());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_hasfreetypevast() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = DataType::tvar_type(&frame);
                    assert!(!dt.has_free_type_vars());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_concrete() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = DataType::tvar_type(&frame);
                    assert!(dt.is_concrete_type());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_dispatchtuple() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = DataType::tvar_type(&frame);
                    assert!(!dt.is_dispatch_tuple());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_zeroinit() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    unsafe {
                        let dt = UnionAll::array_type(&frame)
                            .body()
                            .wrapper()
                            .cast::<UnionAll>()?
                            .body()
                            .wrapper()
                            .cast::<DataType>()?;
                        assert!(!dt.zero_init());
                    }

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_concrete_subtype() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = DataType::tvar_type(&frame);
                    assert!(dt.has_concrete_subtype());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_params() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let dt = unsafe { UnionAll::array_type(&frame).base_type().wrapper() };
                    assert_eq!(dt.n_parameters(), 2);
                    let param = unsafe { dt.parameter(0).unwrap().value() };
                    assert!(param.is::<TypeVar>());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_field_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let val = unsafe {
                        DataType::unionall_type(&frame)
                            .field_type(0)
                            .unwrap()
                            .wrapper()
                    };

                    assert!(val.is::<DataType>());
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_concrete_field_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let val = unsafe {
                        DataType::unionall_type(&frame)
                            .field_type_concrete(0)
                            .unwrap()
                            .wrapper()
                    };

                    assert!(val.is::<DataType>());
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_field_name() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
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
    }

    #[test]
    fn datatype_field_name_str() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let name = DataType::uniontype_type(&frame).field_name_str(0).unwrap();

                    assert_eq!(name, "a");
                    let nonexistent = DataType::uniontype_type(&frame).field_name_str(12);
                    assert!(nonexistent.is_none());
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_field_index_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let idx = DataType::uniontype_type(&frame).field_index_unchecked("a");

                    assert_eq!(idx, 0);

                    let idx = DataType::uniontype_type(&frame).field_index_unchecked("c");

                    assert_eq!(idx, -1);
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    #[cfg(not(feature = "lts"))]
    fn datatype_is_const_field() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsStableTests")?
                            .wrapper()
                            .global(&frame, "WithConst")?
                            .value()
                            .cast::<DataType>()?
                    };

                    assert!(ty.is_const_field(0)?);
                    assert!(DataType::uniontype_type(&frame).is_const_field(0)?);
                    assert!(!DataType::tvar_type(&frame).is_const_field(0)?);
                    assert!(!ty.clone().is_const_field(1)?);
                    assert!(ty.is_const_field(2).is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn cannot_instantiate_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = TypedArray::<usize>::new(frame.as_extended_target(), 1)
                        .into_jlrs_result()?
                        .as_value()
                        .datatype();

                    let instance = ty.instantiate(&mut frame, []);
                    assert!(instance.is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn cannot_instantiate_with_incorrect_params() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "WithAbstract")?
                            .value()
                            .cast::<DataType>()?
                    };

                    let instance = ty.instantiate(&mut frame, [])?;
                    assert!(instance.is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn compare_with_value() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let ty = DataType::bool_type(&frame);
                    assert!(ty == ty.as_value());

                    let ty2 = DataType::int32_type(&frame);
                    assert!(ty != ty2.as_value());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn datatype_cached_by_hash() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let ty = DataType::bool_type(&frame);
                    assert!(ty.cached_by_hash());
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn extend_lifetime() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let output = frame.output();

                    frame
                        .scope(|frame| {
                            let ty = DataType::bool_type(&frame);
                            Ok(ty.root(output))
                        })
                        .unwrap();

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn check_names() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    {
                        let ty = DataType::returnnode_type(&frame);
                        assert_eq!(ty.name(), "ReturnNode");
                    }

                    {
                        let ty = DataType::gotoifnot_type(&frame);
                        assert_eq!(ty.name(), "GotoIfNot");
                    }

                    #[cfg(not(feature = "lts"))]
                    {
                        let ty = DataType::atomicerror_type(&frame);
                        assert_eq!(ty.name(), "ConcurrencyViolationError");
                    }

                    {
                        let ty = DataType::method_match_type(&frame);
                        assert_eq!(ty.name(), "MethodMatch");
                    }

                    #[cfg(not(feature = "lts"))]
                    {
                        let ty = DataType::interconditional_type(&frame);
                        assert_eq!(ty.name(), "InterConditional");
                    }

                    #[cfg(not(feature = "lts"))]
                    {
                        let ty = DataType::partial_opaque_type(&frame);
                        assert_eq!(ty.name(), "PartialOpaque");
                    }

                    {
                        let ty = DataType::partial_struct_type(&frame);
                        assert_eq!(ty.name(), "PartialStruct");
                    }

                    {
                        let ty = DataType::const_type(&frame);
                        assert_eq!(ty.name(), "Const");
                    }

                    {
                        let ty = DataType::argument_type(&frame);
                        assert_eq!(ty.name(), "Argument");
                    }

                    Ok(())
                })
                .unwrap();
        })
    }
}
