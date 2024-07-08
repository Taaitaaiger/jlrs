#[cfg(all(test, feature = "jlrs-derive", feature = "local-rt"))]
mod derive_util;

#[cfg(all(test, feature = "jlrs-derive", feature = "local-rt"))]
mod tests {
    use std::os::raw::c_void;

    use jlrs::{
        data::{
            layout::{
                julia_enum::Enum,
                valid_layout::{ValidField, ValidLayout},
            },
            types::construct_type::{ConstantBool, ConstructType},
        },
        prelude::*,
    };

    use super::derive_util::{derive_impls::*, JULIA_DERIVE};
    fn derive_bits_type_bool() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeBool { a: Bool::new(true) };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<bool>().unwrap().as_bool(), true);
                    assert!(v.is::<BitsTypeBool>());
                    assert!(v.unbox::<BitsTypeBool>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_elided() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let a = HasElidedParam { a: 3.0 };
                    type ElidedParam<T, const U: bool> =
                        HasElidedParamTypeConstructor<T, ConstantBool<U>>;
                    let v = unsafe {
                        Value::try_new_with::<ElidedParam<f64, true>, _, _>(&mut frame, a)?
                    };

                    assert!(v.is::<HasElidedParam<f64>>());
                    assert!(v.unbox::<HasElidedParam<f64>>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_elided_with_ptr() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let main_mod = Module::main(&frame).as_ref();
                    let a = HasElidedParam { a: Some(main_mod) };
                    type ElidedParam<T, const U: bool> =
                        HasElidedParamTypeConstructor<T, ConstantBool<U>>;
                    let v = unsafe {
                        Value::try_new_with::<ElidedParam<Module, true>, _, _>(&mut frame, a)?
                    };

                    assert!(v.is::<HasElidedParam<Option<ModuleRef>>>());
                    assert!(v.unbox::<HasElidedParam<Option<ModuleRef>>>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_double_variant() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "DoubleVariant")?
                        .as_managed()
                        .cast::<DataType>()?;

                    let v2 = Value::new(&mut frame, 2i16);
                    let jl_val = constr
                        .instantiate(&mut frame, &mut [v2])?
                        .into_jlrs_result()?;

                    assert!(jl_val.datatype().is::<DoubleVariant>());

                    let field = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(field.unbox::<i16>().unwrap(), 2);

                    assert!(jl_val.is::<DoubleVariant>());
                    assert!(jl_val.unbox::<DoubleVariant>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn rebox_double_variant() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "DoubleVariant")?
                        .as_managed()
                        .cast::<DataType>()?;

                    let v2 = Value::new(&mut frame, 2i16);
                    let jl_val = constr
                        .instantiate(&mut frame, &mut [v2])?
                        .into_jlrs_result()?;

                    let unboxed = jl_val.unbox::<DoubleVariant>()?;
                    assert!(
                        Value::try_new_with::<DoubleVariant, _, _>(&mut frame, unboxed).is_ok()
                    );

                    Ok(())
                })
                .unwrap();
        })
    }

    fn cannot_rebox_as_incompatible() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "DoubleVariant")?
                        .as_managed()
                        .cast::<DataType>()?;

                    let v2 = Value::new(&mut frame, 2i16);
                    let jl_val = constr
                        .instantiate(&mut frame, &mut [v2])?
                        .into_jlrs_result()?;

                    let unboxed = jl_val.unbox::<DoubleVariant>()?;
                    assert!(
                        Value::try_new_with::<DoubleUVariant, _, _>(&mut frame, unboxed).is_err()
                    );

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_generic_tu() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let ty = WithGenericTU::<isize, usize>::construct_type(&mut frame);
                    assert_eq!(
                        ty.cast::<DataType>().unwrap().size().unwrap() as usize,
                        std::mem::size_of::<isize>() + std::mem::size_of::<usize>()
                    );
                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_char() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeChar { a: Char::new('b') };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<char>().unwrap().try_as_char().unwrap(), 'b');
                    assert!(v.is::<BitsTypeChar>());
                    assert!(v.unbox::<BitsTypeChar>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_uint8() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeUInt8 { a: 1 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<u8>().unwrap(), 1);
                    assert!(v.is::<BitsTypeUInt8>());
                    assert!(v.unbox::<BitsTypeUInt8>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_uint16() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeUInt16 { a: 2 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<u16>().unwrap(), 2);
                    assert!(v.is::<BitsTypeUInt16>());
                    assert!(v.unbox::<BitsTypeUInt16>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_uint32() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeUInt32 { a: 3 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<u32>().unwrap(), 3);
                    assert!(v.is::<BitsTypeUInt32>());
                    assert!(v.unbox::<BitsTypeUInt32>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_uint64() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeUInt64 { a: 4 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<u64>().unwrap(), 4);
                    assert!(v.is::<BitsTypeUInt64>());
                    assert!(v.unbox::<BitsTypeUInt64>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_uint() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeUInt { a: 5 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<usize>().unwrap(), 5);
                    assert!(v.is::<BitsTypeUInt>());
                    assert!(v.unbox::<BitsTypeUInt>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_int8() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeInt8 { a: -1 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<i8>().unwrap(), -1);
                    assert!(v.is::<BitsTypeInt8>());
                    assert!(v.unbox::<BitsTypeInt8>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_int16() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeInt16 { a: -2 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<i16>().unwrap(), -2);
                    assert!(v.is::<BitsTypeInt16>());
                    assert!(v.unbox::<BitsTypeInt16>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_int32() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeInt32 { a: -3 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<i32>().unwrap(), -3);
                    assert!(v.is::<BitsTypeInt32>());
                    assert!(v.unbox::<BitsTypeInt32>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_int64() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeInt64 { a: -4 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<i64>().unwrap(), -4);
                    assert!(v.is::<BitsTypeInt64>());
                    assert!(v.unbox::<BitsTypeInt64>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_int() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeInt { a: -5 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<isize>().unwrap(), -5);
                    assert!(v.is::<BitsTypeInt>());
                    assert!(v.unbox::<BitsTypeInt>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_float32() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeFloat32 { a: 1.2 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<f32>().unwrap(), 1.2);
                    assert!(v.is::<BitsTypeFloat32>());
                    assert!(v.unbox::<BitsTypeFloat32>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_type_float64() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsTypeFloat64 { a: -2.3 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<f64>().unwrap(), -2.3);
                    assert!(v.is::<BitsTypeFloat64>());
                    assert!(v.unbox::<BitsTypeFloat64>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_char_float32_float64() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsCharFloat32Float64 {
                        a: Char::new('a'),
                        b: 3.0,
                        c: 4.0,
                    };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<char>().unwrap().try_as_char().unwrap(), 'a');
                    assert!(v.is::<BitsCharFloat32Float64>());
                    assert!(v.unbox::<BitsCharFloat32Float64>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_int_bool() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsIntBool {
                        a: 1,
                        b: Bool::new(true),
                    };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<isize>().unwrap(), 1);
                    assert!(v.is::<BitsIntBool>());
                    assert!(v.unbox::<BitsIntBool>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_char_bits_int_char() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsCharBitsIntChar {
                        a: Char::new('a'),
                        b: BitsIntChar {
                            a: 1,
                            b: Char::new('b'),
                        },
                    };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(first.unbox::<char>().unwrap().try_as_char().unwrap(), 'a');
                    assert!(v.is::<BitsCharBitsIntChar>());
                    assert!(v.unbox::<BitsCharBitsIntChar>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_uint8_tuple_int32_int64() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let s = BitsUInt8TupleInt32Int64 {
                        a: 0,
                        b: Tuple2(-1, -3),
                    };
                    let v = Value::new(&mut frame, s);

                    let first = v.get_nth_field(&mut frame, 0).unwrap();
                    let second = v.get_nth_field(&mut frame, 1).unwrap();

                    assert_eq!(first.unbox::<u8>().unwrap(), 0);
                    assert_eq!(second.unbox::<Tuple2<i32, i64>>().unwrap(), Tuple2(-1, -3));
                    assert!(v.is::<BitsUInt8TupleInt32Int64>());
                    assert!(v.unbox::<BitsUInt8TupleInt32Int64>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_bits_uint8_tuple_int32_tuple_int16_uint16() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let rs_val = BitsUInt8TupleInt32TupleInt16UInt16 {
                        a: 0,
                        b: Tuple2(-1, Tuple2(-1, 3)),
                    };
                    let jl_val = Value::new(&mut frame, rs_val.clone());

                    unsafe {
                        assert!(Module::base(&frame)
                            .function(&frame, "typeof")?
                            .as_managed()
                            .call1(&mut frame, jl_val)
                            .unwrap()
                            .cast::<DataType>()?
                            .is::<BitsUInt8TupleInt32TupleInt16UInt16>());
                    }

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(first.unbox::<u8>().unwrap(), (&rs_val).a);

                    let second = jl_val.get_nth_field(&mut frame, 1).unwrap();
                    assert_eq!(
                        second.unbox::<Tuple2<i32, Tuple2<i16, u16>>>().unwrap(),
                        rs_val.b
                    );

                    assert!(jl_val.is::<BitsUInt8TupleInt32TupleInt16UInt16>());
                    assert!(jl_val
                        .unbox::<BitsUInt8TupleInt32TupleInt16UInt16>()
                        .is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_single_variant() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "SingleVariant")?
                        .as_managed();
                    let v2 = Value::new(&mut frame, 2i32);
                    let jl_val = constr.call1(&mut frame, v2).unwrap();

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<SingleVariant>());

                    assert!(jl_val.is::<SingleVariant>());
                    assert!(jl_val.unbox::<SingleVariant>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_size_align_mismatch() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "SizeAlignMismatch")?
                        .as_managed();

                    let v2 = Value::new(&mut frame, 2i32);
                    let jl_val = constr.call1(&mut frame, v2).unwrap();

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<SizeAlignMismatch>());

                    let second = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(second.unbox::<i32>().unwrap(), 2);

                    assert!(jl_val.is::<SizeAlignMismatch>());
                    assert!(jl_val.unbox::<SizeAlignMismatch>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_union_in_tuple() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "UnionInTuple")?
                        .as_managed();

                    let v2 = Value::new(&mut frame, Tuple1(2i32));
                    let jl_val = constr.call1(&mut frame, v2).unwrap();

                    let second = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(second.unbox::<Tuple1<i32>>().unwrap(), Tuple1(2));

                    let _uit = jl_val.unbox::<UnionInTuple>()?;

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_non_bits_union() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "NonBitsUnion")?
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i8);
                    let jl_val = constr.call1(&mut frame, v1).unwrap();

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<NonBitsUnion>());

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(first.unbox::<i8>().unwrap(), 1);

                    assert!(jl_val.is::<NonBitsUnion>());
                    assert!(jl_val.unbox::<NonBitsUnion>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_string() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "WithString")?
                        .as_managed();

                    let s = JuliaString::new(&mut frame, "foo");
                    let jl_val = constr.call1(&mut frame, s.as_value()).into_jlrs_result()?;

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .into_jlrs_result()?
                        .cast::<DataType>()?
                        .is::<WithString>());

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(first.unbox::<String>().unwrap().unwrap(), "foo");

                    assert!(jl_val.is::<WithString>());
                    assert!(jl_val.unbox::<WithString>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_with_generic_t_i32() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")?
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i32);
                    let jl_val = constr.call1(&mut frame, v1).unwrap();

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithGenericT<i32>>());

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(first.unbox::<i32>().unwrap(), 1);

                    assert!(jl_val.is::<WithGenericT<i32>>());
                    assert!(jl_val.unbox::<WithGenericT<i32>>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_with_unionall() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")?
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i32);
                    let wgt = constr.call1(&mut frame, v1).unwrap();

                    let constr = Module::main(&frame)
                        .global(&frame, "WithGenericUnionAll")?
                        .as_managed();

                    let jl_val = constr.call1(&mut frame, wgt).unwrap();

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithGenericUnionAll>());

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<i32>>().is_ok());

                    assert!(jl_val.is::<WithGenericUnionAll>());
                    assert!(jl_val.unbox::<WithGenericUnionAll>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_with_nested_generic() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")?
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i32);
                    let wgt = constr.call1(&mut frame, v1).unwrap();

                    let constr = Module::main(&frame)
                        .global(&frame, "WithNestedGenericT")?
                        .as_managed();

                    let jl_val = constr.call1(&mut frame, wgt).unwrap();

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithNestedGenericT<i32>>());

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<i32>>().is_ok());

                    assert!(jl_val.is::<WithNestedGenericT<i32>>());
                    assert!(jl_val.unbox::<WithNestedGenericT<i32>>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_with_propagated_lifetime() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let global = frame.unrooted();
                    let constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")?
                        .as_managed();

                    let wgt = constr
                        .call1(&mut frame, Module::base(&global).as_value())
                        .unwrap();

                    let constr = Module::main(&frame)
                        .global(&frame, "WithPropagatedLifetime")?
                        .as_managed();

                    let jl_val = constr.call1(&mut frame, wgt).unwrap();

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithPropagatedLifetime>());

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<Option<ModuleRef>>>().is_ok());

                    assert!(jl_val.is::<WithPropagatedLifetime>());
                    assert!(jl_val.unbox::<WithPropagatedLifetime>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_with_propagated_lifetimes() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let arr = TypedArray::<i32>::new(&mut frame, (2, 2)).into_jlrs_result()?;

                    let wgt_constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")?
                        .as_managed();

                    let wgt = wgt_constr.call1(&mut frame, arr.as_value()).unwrap();

                    let constr = Module::base(&frame).function(&frame, "tuple")?.as_managed();
                    let int = Value::new(&mut frame, 2i32);
                    let tup = constr.call2(&mut frame, int, wgt).unwrap();

                    let a = wgt_constr.call1(&mut frame, tup).unwrap();
                    let constr = Module::main(&frame)
                        .global(&frame, "WithPropagatedLifetimes")?
                        .as_managed();

                    let jl_val = constr.call1(&mut frame, a).unwrap();

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithPropagatedLifetimes>());

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(first
                        .unbox::<WithGenericT<Tuple2<i32, WithGenericT<Option<ArrayRef>>>>>()
                        .is_ok());

                    assert!(jl_val.is::<WithPropagatedLifetimes>());
                    assert!(jl_val.unbox::<WithPropagatedLifetimes>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_with_set_generic() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let wgt_constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")?
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i64);
                    let wgt = wgt_constr.call1(&mut frame, v1).unwrap();

                    let constr = Module::main(&frame)
                        .global(&frame, "WithSetGeneric")?
                        .as_managed();

                    let jl_val = constr.call1(&mut frame, wgt).unwrap();

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithSetGeneric>());

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<i64>>().is_ok());

                    assert!(jl_val.is::<WithSetGeneric>());
                    assert!(jl_val.unbox::<WithSetGeneric>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_with_set_generic_tuple() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let wgt_constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")?
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i64);
                    let wgt = wgt_constr.call1(&mut frame, v1).unwrap();

                    let tup_constr = Module::base(&frame).function(&frame, "tuple")?.as_managed();
                    let v2 = tup_constr.call1(&mut frame, wgt).unwrap();

                    let constr = Module::main(&frame)
                        .global(&frame, "WithSetGenericTuple")?
                        .as_managed();

                    let jl_val = constr.call1(&mut frame, v2).unwrap();

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithSetGenericTuple>());

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    first.unbox::<Tuple1<WithGenericT<i64>>>().unwrap();

                    assert!(jl_val.is::<WithSetGenericTuple>());
                    assert!(jl_val.unbox::<WithSetGenericTuple>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_with_value_type() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let b = Value::new(&mut frame, true);
                    let wvt_constr = Module::main(&frame)
                        .global(&frame, "WithValueType")?
                        .as_managed()
                        .apply_type(&mut frame, [b])
                        .into_jlrs_result()?;

                    let v1 = Value::new(&mut frame, 1i64);
                    let jl_val = wvt_constr.call1(&mut frame, v1).unwrap();

                    assert!(Module::base(&frame)
                        .function(&frame, "typeof")?
                        .as_managed()
                        .call1(&mut frame, jl_val)
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithValueType>());

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(first.unbox::<i64>().is_ok());

                    assert!(jl_val.is::<WithValueType>());
                    assert!(jl_val.unbox::<WithValueType>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn derive_zero_sized() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let v = Value::new(&mut frame, Empty {});
                    assert!(v.unbox::<Empty>().is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn isbits_into_julia() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let wvt = WithValueType { a: 1 };
                    type WVT = WithValueTypeTypeConstructor<ConstantBool<true>>;
                    let v = Value::new_bits_from_layout::<WVT, _>(&mut frame, wvt.clone())?;
                    let wvt_unboxed = v.unbox::<WithValueType>()?;
                    assert_eq!(wvt, wvt_unboxed);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn trivial_isbits_into_julia() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let layout = WithGenericTU { a: 1i32, b: 2u32 };
                    let v = Value::new_bits(&mut frame, layout.clone());
                    let layout_unboxed = v.unbox::<WithGenericTU<i32, u32>>()?;
                    assert_eq!(layout, layout_unboxed);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn test_enums() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    // Test IntoJulia, Typecheck, ValidLayout, ValidField and Unbox for each variant.
                    let mut test_fn = |layout| -> JlrsResult<_> {
                        let v = Value::new(&mut frame, layout);

                        assert!(v.is::<StandardEnum>());
                        assert!(StandardEnum::valid_layout(v.datatype().as_value()));
                        assert!(StandardEnum::valid_field(v.datatype().as_value()));

                        let layout_unboxed = v.unbox::<StandardEnum>()?;
                        assert_eq!(layout, layout_unboxed);
                        Ok(())
                    };

                    test_fn(StandardEnum::SeA)?;
                    test_fn(StandardEnum::SeB)?;
                    test_fn(StandardEnum::SeC)?;

                    Ok(())
                })
                .unwrap();
        })
    }

    fn test_enums_ccall() {
        JULIA_DERIVE.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    // Test that enums can be passed and returned by value
                    unsafe extern "C" fn echo(s: StandardEnum) -> StandardEnum {
                        s
                    }

                    let echo_v = Value::new(&mut frame, echo as *mut c_void);
                    let se_a = StandardEnum::SeA.as_value(&frame);
                    let func = unsafe {
                        Value::eval_string(
                            &mut frame,
                            "x(f, s::StandardEnum) = ccall(f, StandardEnum, (StandardEnum,), s)",
                        )
                    }
                    .unwrap();

                    let res = unsafe { func.call2(&mut frame, echo_v, se_a) }.unwrap();
                    assert_eq!(se_a, res);

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn derive_tests() {
        derive_bits_type_bool();
        derive_elided();
        derive_elided_with_ptr();
        derive_double_variant();
        rebox_double_variant();
        cannot_rebox_as_incompatible();
        derive_generic_tu();
        derive_bits_type_char();
        derive_bits_type_uint8();
        derive_bits_type_uint16();
        derive_bits_type_uint32();
        derive_bits_type_uint64();
        derive_bits_type_uint();
        derive_bits_type_int8();
        derive_bits_type_int16();
        derive_bits_type_int32();
        derive_bits_type_int64();
        derive_bits_type_int();
        derive_bits_type_float32();
        derive_bits_type_float64();
        derive_bits_char_float32_float64();
        derive_bits_int_bool();
        derive_bits_char_bits_int_char();
        derive_bits_uint8_tuple_int32_int64();
        derive_bits_uint8_tuple_int32_tuple_int16_uint16();
        derive_single_variant();
        derive_size_align_mismatch();
        derive_union_in_tuple();
        derive_non_bits_union();
        derive_with_generic_t_i32();
        derive_with_unionall();
        derive_with_nested_generic();
        derive_with_propagated_lifetime();
        derive_with_set_generic();
        derive_with_set_generic_tuple();
        derive_with_value_type();
        derive_zero_sized();
        derive_with_propagated_lifetimes();
        derive_string();
        isbits_into_julia();
        trivial_isbits_into_julia();
        test_enums();
        test_enums_ccall();
    }
}
