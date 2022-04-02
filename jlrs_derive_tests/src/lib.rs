mod impls;
mod util;

#[cfg(test)]
mod tests {
    use super::impls::*;
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn derive_bits_type_bool() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeBool { a: Bool::new(true) };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<bool>().unwrap().as_bool(), true);
                    assert!(v.is::<BitsTypeBool>());
                    assert!(v.unbox::<BitsTypeBool>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_char() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeChar { a: Char::new('b') };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<char>().unwrap().try_as_char().unwrap(), 'b');
                    assert!(v.is::<BitsTypeChar>());
                    assert!(v.unbox::<BitsTypeChar>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_uint8() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeUInt8 { a: 1 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<u8>().unwrap(), 1);
                    assert!(v.is::<BitsTypeUInt8>());
                    assert!(v.unbox::<BitsTypeUInt8>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_uint16() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeUInt16 { a: 2 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<u16>().unwrap(), 2);
                    assert!(v.is::<BitsTypeUInt16>());
                    assert!(v.unbox::<BitsTypeUInt16>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_uint32() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeUInt32 { a: 3 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<u32>().unwrap(), 3);
                    assert!(v.is::<BitsTypeUInt32>());
                    assert!(v.unbox::<BitsTypeUInt32>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_uint64() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeUInt64 { a: 4 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<u64>().unwrap(), 4);
                    assert!(v.is::<BitsTypeUInt64>());
                    assert!(v.unbox::<BitsTypeUInt64>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_uint() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeUInt { a: 5 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<u64>().unwrap(), 5);
                    assert!(v.is::<BitsTypeUInt>());
                    assert!(v.unbox::<BitsTypeUInt>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_int8() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeInt8 { a: -1 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<i8>().unwrap(), -1);
                    assert!(v.is::<BitsTypeInt8>());
                    assert!(v.unbox::<BitsTypeInt8>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_int16() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeInt16 { a: -2 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<i16>().unwrap(), -2);
                    assert!(v.is::<BitsTypeInt16>());
                    assert!(v.unbox::<BitsTypeInt16>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_int32() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeInt32 { a: -3 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<i32>().unwrap(), -3);
                    assert!(v.is::<BitsTypeInt32>());
                    assert!(v.unbox::<BitsTypeInt32>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_int64() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeInt64 { a: -4 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<i64>().unwrap(), -4);
                    assert!(v.is::<BitsTypeInt64>());
                    assert!(v.unbox::<BitsTypeInt64>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_int() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeInt { a: -5 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<i64>().unwrap(), -5);
                    assert!(v.is::<BitsTypeInt>());
                    assert!(v.unbox::<BitsTypeInt>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_float32() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeFloat32 { a: 1.2 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<f32>().unwrap(), 1.2);
                    assert!(v.is::<BitsTypeFloat32>());
                    assert!(v.unbox::<BitsTypeFloat32>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_float64() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsTypeFloat64 { a: -2.3 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<f64>().unwrap(), -2.3);
                    assert!(v.is::<BitsTypeFloat64>());
                    assert!(v.unbox::<BitsTypeFloat64>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_char_float32_float64() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsCharFloat32Float64 {
                        a: Char::new('a'),
                        b: 3.0,
                        c: 4.0,
                    };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<char>().unwrap().try_as_char().unwrap(), 'a');
                    assert!(v.is::<BitsCharFloat32Float64>());
                    assert!(v.unbox::<BitsCharFloat32Float64>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_int_bool() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsIntBool {
                        a: 1,
                        b: Bool::new(true),
                    };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.unbox::<i64>().unwrap(), 1);
                    assert!(v.is::<BitsIntBool>());
                    assert!(v.unbox::<BitsIntBool>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_char_bits_int_char() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsCharBitsIntChar {
                        a: Char::new('a'),
                        b: BitsIntChar {
                            a: 1,
                            b: Char::new('b'),
                        },
                    };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.unbox::<char>().unwrap().try_as_char().unwrap(), 'a');
                    assert!(v.is::<BitsCharBitsIntChar>());
                    assert!(v.unbox::<BitsCharBitsIntChar>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_uint8_tuple_int32_int64() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let s = BitsUInt8TupleInt32Int64 {
                        a: 0,
                        b: Tuple2(-1, -3),
                    };
                    let v = Value::new(&mut *frame, s).unwrap();

                    let first = v.get_nth_field(&mut *frame, 0).unwrap();
                    let second = v.get_nth_field(&mut *frame, 1).unwrap();

                    assert_eq!(first.unbox::<u8>().unwrap(), 0);
                    assert_eq!(second.unbox::<Tuple2<i32, i64>>().unwrap(), Tuple2(-1, -3));
                    assert!(v.is::<BitsUInt8TupleInt32Int64>());
                    assert!(v.unbox::<BitsUInt8TupleInt32Int64>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_uint8_tuple_int32_tuple_int16_uint16() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| {
                    let rs_val = BitsUInt8TupleInt32TupleInt16UInt16 {
                        a: 0,
                        b: Tuple2(-1, Tuple2(-1, 3)),
                    };
                    let jl_val = Value::new(&mut *frame, rs_val.clone()).unwrap();

                    unsafe {
                        assert!(Module::base(global)
                            .function_ref("typeof")?
                            .wrapper_unchecked()
                            .call1(&mut *frame, jl_val)?
                            .unwrap()
                            .cast::<DataType>()?
                            .is::<BitsUInt8TupleInt32TupleInt16UInt16>());
                    }

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.unbox::<u8>().unwrap(), (&rs_val).a);

                    let second = jl_val.get_nth_field(&mut *frame, 1).unwrap();
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
                .unwrap()
        })
    }

    #[test]
    fn derive_single_variant() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let constr = Module::main(global)
                        .submodule_ref("WithBitsUnion")?
                        .wrapper_unchecked()
                        .global_ref("SingleVariant")?
                        .wrapper_unchecked();
                    let v1 = Value::new(&mut *frame, 1i8)?;
                    let v2 = Value::new(&mut *frame, 2i32)?;
                    let v3 = Value::new(&mut *frame, 3i8)?;
                    let jl_val = constr.call3(&mut *frame, v1, v2, v3)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<SingleVariant>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.unbox::<i8>().unwrap(), 1);

                    let third = jl_val.get_nth_field(&mut *frame, 2).unwrap();
                    assert_eq!(third.unbox::<i8>().unwrap(), 3);

                    assert!(jl_val.is::<SingleVariant>());
                    assert!(jl_val.unbox::<SingleVariant>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    fn derive_double_variant() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let constr = Module::main(global)
                        .submodule_ref("WithBitsUnion")?
                        .wrapper_unchecked()
                        .global_ref("DoubleVariant")?
                        .wrapper_unchecked()
                        .cast::<DataType>()?;

                    let v1 = Value::new(&mut *frame, 1i8)?;
                    let v2 = Value::new(&mut *frame, 2i16)?;
                    let v3 = Value::new(&mut *frame, 3i8)?;
                    let jl_val = constr
                        .instantiate(&mut *frame, &mut [v1, v2, v3])?
                        .into_jlrs_result()?;

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<DoubleVariant>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.unbox::<i8>().unwrap(), 1);

                    let second = jl_val.get_nth_field(&mut *frame, 1).unwrap();
                    assert_eq!(second.unbox::<i16>().unwrap(), 2);

                    let third = jl_val.get_nth_field(&mut *frame, 2).unwrap();
                    assert_eq!(third.unbox::<i8>().unwrap(), 3);

                    assert!(jl_val.is::<DoubleVariant>());
                    assert!(jl_val.unbox::<DoubleVariant>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_size_align_mismatch() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let constr = Module::main(global)
                        .submodule_ref("WithBitsUnion")?
                        .wrapper_unchecked()
                        .global_ref("SizeAlignMismatch")?
                        .wrapper_unchecked();

                    let v1 = Value::new(&mut *frame, 1i8)?;
                    let v2 = Value::new(&mut *frame, 2i32)?;
                    let v3 = Value::new(&mut *frame, 3i8)?;
                    let jl_val = constr.call3(&mut *frame, v1, v2, v3)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<SizeAlignMismatch>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.unbox::<i8>().unwrap(), 1);

                    let second = jl_val.get_nth_field(&mut *frame, 1).unwrap();
                    assert_eq!(second.unbox::<i32>().unwrap(), 2);

                    let third = jl_val.get_nth_field(&mut *frame, 2).unwrap();
                    assert_eq!(third.unbox::<i8>().unwrap(), 3);

                    assert!(jl_val.is::<SizeAlignMismatch>());
                    assert!(jl_val.unbox::<SizeAlignMismatch>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_union_in_tuple() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let constr = Module::main(global)
                        .submodule_ref("WithBitsUnion")?
                        .wrapper_unchecked()
                        .global_ref("UnionInTuple")?
                        .wrapper_unchecked();

                    let v1 = Value::new(&mut *frame, 1i8)?;
                    let v2 = Value::new(&mut *frame, Tuple1(2i32))?;
                    let v3 = Value::new(&mut *frame, 3i8)?;
                    let jl_val = constr.call3(&mut *frame, v1, v2, v3)?.unwrap();

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.unbox::<i8>().unwrap(), 1);

                    let second = jl_val.get_nth_field(&mut *frame, 1).unwrap();
                    assert_eq!(second.unbox::<Tuple1<i32>>().unwrap(), Tuple1(2));

                    let third = jl_val.get_nth_field(&mut *frame, 2).unwrap();
                    assert_eq!(third.unbox::<i8>().unwrap(), 3);

                    let uit = jl_val.unbox::<UnionInTuple>()?;
                    assert_eq!(uit.a, 1);
                    assert_eq!(uit.c, 3);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_non_bits_union() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let constr = Module::main(global)
                        .submodule_ref("WithNonBitsUnion")?
                        .wrapper_unchecked()
                        .global_ref("NonBitsUnion")?
                        .wrapper_unchecked();

                    let v1 = Value::new(&mut *frame, 1i8)?;
                    let jl_val = constr.call1(&mut *frame, v1)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<NonBitsUnion>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.unbox::<i8>().unwrap(), 1);

                    assert!(jl_val.is::<NonBitsUnion>());
                    assert!(jl_val.unbox::<NonBitsUnion>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    /*
    #[test]
    fn derive_string() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe  {
                    let constr = Module::main(global)
                        .submodule_ref("WithStrings")?.wrapper_unchecked()
                        .global_ref("WithString")?.wrapper_unchecked();

                    let v1 = Value::new(&mut *frame, "foo")?;
                    let jl_val = constr.call1(&mut *frame, v1)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?.wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithString>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.unbox::<String>().unwrap().unwrap(), "foo");

                    assert!(jl_val.is::<WithString>());
                    assert!(jl_val.unbox::<WithString>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }
    */

    #[test]
    fn derive_with_generic_t_i32() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithGenericT")?
                        .wrapper_unchecked();

                    let v1 = Value::new(&mut *frame, 1i32)?;
                    let jl_val = constr.call1(&mut *frame, v1)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithGenericT<i32>>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.unbox::<i32>().unwrap(), 1);

                    assert!(jl_val.is::<WithGenericT<i32>>());
                    assert!(jl_val.unbox::<WithGenericT<i32>>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_with_unionall() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithGenericT")?
                        .wrapper_unchecked();

                    let v1 = Value::new(&mut *frame, 1i32)?;
                    let wgt = constr.call1(&mut *frame, v1)?.unwrap();

                    let constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithGenericUnionAll")?
                        .wrapper_unchecked();

                    let jl_val = constr.call1(&mut *frame, wgt)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithGenericUnionAll>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<i32>>().is_ok());

                    assert!(jl_val.is::<WithGenericUnionAll>());
                    assert!(jl_val.unbox::<WithGenericUnionAll>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_with_nested_generic() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithGenericT")?
                        .wrapper_unchecked();

                    let v1 = Value::new(&mut *frame, 1i32)?;
                    let wgt = constr.call1(&mut *frame, v1)?.unwrap();

                    let constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithNestedGenericT")?
                        .wrapper_unchecked();

                    let jl_val = constr.call1(&mut *frame, wgt)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithNestedGenericT<i32>>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<i32>>().is_ok());

                    assert!(jl_val.is::<WithNestedGenericT<i32>>());
                    assert!(jl_val.unbox::<WithNestedGenericT<i32>>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_with_propagated_lifetime() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithGenericT")?
                        .wrapper_unchecked();

                    let wgt = constr
                        .call1(&mut *frame, Module::base(global).as_value())?
                        .unwrap();

                    let constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithPropagatedLifetime")?
                        .wrapper_unchecked();

                    let jl_val = constr.call1(&mut *frame, wgt)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithPropagatedLifetime>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<Module>>().is_ok());

                    assert!(jl_val.is::<WithPropagatedLifetime>());
                    assert!(jl_val.unbox::<WithPropagatedLifetime>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    fn derive_with_propagated_lifetimes() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let arr =
                        Array::new::<i32, _, _, _>(&mut *frame, (2, 2))?.into_jlrs_result()?;

                    let wgt_constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithGenericT")?
                        .wrapper_unchecked();

                    let wgt = wgt_constr.call1(&mut *frame, arr.as_value())?.unwrap();

                    let constr = Module::base(global)
                        .function_ref("tuple")?
                        .wrapper_unchecked();
                    let int = Value::new(&mut *frame, 2i32)?;
                    let tup = constr.call2(&mut *frame, int, wgt)?.unwrap();

                    let a = wgt_constr.call1(&mut *frame, tup)?.unwrap();
                    let constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithPropagatedLifetimes")?
                        .wrapper_unchecked();

                    let jl_val = constr.call1(&mut *frame, a)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithPropagatedLifetimes>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first
                        .unbox::<WithGenericT<Tuple2<i32, WithGenericT<Array>>>>()
                        .is_ok());

                    assert!(jl_val.is::<WithPropagatedLifetimes>());
                    assert!(jl_val.unbox::<WithPropagatedLifetimes>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_with_set_generic() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let wgt_constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithGenericT")?
                        .wrapper_unchecked();

                    let v1 = Value::new(&mut *frame, 1i64)?;
                    let wgt = wgt_constr.call1(&mut *frame, v1)?.unwrap();

                    let constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithSetGeneric")?
                        .wrapper_unchecked();

                    let jl_val = constr.call1(&mut *frame, wgt)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithSetGeneric>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<i64>>().is_ok());

                    assert!(jl_val.is::<WithSetGeneric>());
                    assert!(jl_val.unbox::<WithSetGeneric>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_with_set_generic_tuple() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let wgt_constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithGenericT")?
                        .wrapper_unchecked();

                    let v1 = Value::new(&mut *frame, 1i64)?;
                    let wgt = wgt_constr.call1(&mut *frame, v1)?.unwrap();

                    let tup_constr = Module::base(global)
                        .function_ref("tuple")?
                        .wrapper_unchecked();
                    let v2 = tup_constr.call1(&mut *frame, wgt)?.unwrap();

                    let constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("WithSetGenericTuple")?
                        .wrapper_unchecked();

                    let jl_val = constr.call1(&mut *frame, v2)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithSetGenericTuple>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    first.unbox::<Tuple1<WithGenericT<i64>>>().unwrap();

                    assert!(jl_val.is::<WithSetGenericTuple>());
                    assert!(jl_val.unbox::<WithSetGenericTuple>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_with_value_type() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|global, frame| unsafe {
                    let wvt_constr = Module::main(global)
                        .submodule_ref("WithGeneric")?
                        .wrapper_unchecked()
                        .global_ref("withvaluetype")?
                        .wrapper_unchecked();

                    let v1 = Value::new(&mut *frame, 1i64)?;
                    let jl_val = wvt_constr.call1(&mut *frame, v1)?.unwrap();

                    assert!(Module::base(global)
                        .function_ref("typeof")?
                        .wrapper_unchecked()
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithValueType>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first.unbox::<i64>().is_ok());

                    assert!(jl_val.is::<WithValueType>());
                    assert!(jl_val.unbox::<WithValueType>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }
    #[test]
    fn derive_zero_sized() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let v = Value::new(&mut *frame, ZeroSized {})?;
                    assert!(v.unbox::<ZeroSized>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }
}
