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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeBool { a: true };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<bool>().unwrap(), true);
                    assert!(v.is::<BitsTypeBool>());
                    assert!(v.cast::<BitsTypeBool>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeChar { a: 'b' };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<char>().unwrap(), 'b');
                    assert!(v.is::<BitsTypeChar>());
                    assert!(v.cast::<BitsTypeChar>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeUInt8 { a: 1 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<u8>().unwrap(), 1);
                    assert!(v.is::<BitsTypeUInt8>());
                    assert!(v.cast::<BitsTypeUInt8>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeUInt16 { a: 2 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<u16>().unwrap(), 2);
                    assert!(v.is::<BitsTypeUInt16>());
                    assert!(v.cast::<BitsTypeUInt16>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeUInt32 { a: 3 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<u32>().unwrap(), 3);
                    assert!(v.is::<BitsTypeUInt32>());
                    assert!(v.cast::<BitsTypeUInt32>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeUInt64 { a: 4 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<u64>().unwrap(), 4);
                    assert!(v.is::<BitsTypeUInt64>());
                    assert!(v.cast::<BitsTypeUInt64>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeUInt { a: 5 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<u64>().unwrap(), 5);
                    assert!(v.is::<BitsTypeUInt>());
                    assert!(v.cast::<BitsTypeUInt>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeInt8 { a: -1 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<i8>().unwrap(), -1);
                    assert!(v.is::<BitsTypeInt8>());
                    assert!(v.cast::<BitsTypeInt8>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeInt16 { a: -2 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<i16>().unwrap(), -2);
                    assert!(v.is::<BitsTypeInt16>());
                    assert!(v.cast::<BitsTypeInt16>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeInt32 { a: -3 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<i32>().unwrap(), -3);
                    assert!(v.is::<BitsTypeInt32>());
                    assert!(v.cast::<BitsTypeInt32>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeInt64 { a: -4 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<i64>().unwrap(), -4);
                    assert!(v.is::<BitsTypeInt64>());
                    assert!(v.cast::<BitsTypeInt64>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeInt { a: -5 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<i64>().unwrap(), -5);
                    assert!(v.is::<BitsTypeInt>());
                    assert!(v.cast::<BitsTypeInt>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeFloat32 { a: 1.2 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<f32>().unwrap(), 1.2);
                    assert!(v.is::<BitsTypeFloat32>());
                    assert!(v.cast::<BitsTypeFloat32>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeFloat64 { a: -2.3 };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<f64>().unwrap(), -2.3);
                    assert!(v.is::<BitsTypeFloat64>());
                    assert!(v.cast::<BitsTypeFloat64>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsCharFloat32Float64 {
                        a: 'a',
                        b: 3.0,
                        c: 4.0,
                    };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<char>().unwrap(), 'a');
                    assert!(v.is::<BitsCharFloat32Float64>());
                    assert!(v.cast::<BitsCharFloat32Float64>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsIntBool { a: 1, b: true };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();

                    assert_eq!(first.cast::<i64>().unwrap(), 1);
                    assert!(v.is::<BitsIntBool>());
                    assert!(v.cast::<BitsIntBool>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsCharBitsIntChar {
                        a: 'a',
                        b: BitsIntChar { a: 1, b: 'b' },
                    };
                    let v = Value::new(&mut *frame, s).unwrap();
                    let first = v.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.cast::<char>().unwrap(), 'a');
                    assert!(v.is::<BitsCharBitsIntChar>());
                    assert!(v.cast::<BitsCharBitsIntChar>().is_ok());

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
                .dynamic_frame(|_global, frame| {
                    let s = BitsUInt8TupleInt32Int64 {
                        a: 0,
                        b: Tuple2(-1, -3),
                    };
                    let v = Value::new(&mut *frame, s).unwrap();

                    let first = v.get_nth_field(&mut *frame, 0).unwrap();
                    let second = v.get_nth_field(&mut *frame, 1).unwrap();

                    assert_eq!(first.cast::<u8>().unwrap(), 0);
                    assert_eq!(second.cast::<Tuple2<i32, i64>>().unwrap(), Tuple2(-1, -3));
                    assert!(v.is::<BitsUInt8TupleInt32Int64>());
                    assert!(v.cast::<BitsUInt8TupleInt32Int64>().is_ok());

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
                .dynamic_frame(|global, frame| {
                    let rs_val = BitsUInt8TupleInt32TupleInt16UInt16 {
                        a: 0,
                        b: Tuple2(-1, Tuple2(-1, 3)),
                    };
                    let jl_val = Value::new(&mut *frame, rs_val).unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<BitsUInt8TupleInt32TupleInt16UInt16>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.cast::<u8>().unwrap(), rs_val.a);

                    let second = jl_val.get_nth_field(&mut *frame, 1).unwrap();
                    assert_eq!(
                        second.cast::<Tuple2<i32, Tuple2<i16, u16>>>().unwrap(),
                        rs_val.b
                    );

                    assert!(jl_val.is::<BitsUInt8TupleInt32TupleInt16UInt16>());
                    assert!(jl_val.cast::<BitsUInt8TupleInt32TupleInt16UInt16>().is_ok());

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
                .dynamic_frame(|global, frame| {
                    let constr = Module::main(global)
                        .submodule("WithBitsUnion")?
                        .function("SingleVariant")?;
                    let v1 = Value::new(&mut *frame, 1i8)?;
                    let v2 = Value::new(&mut *frame, 2i32)?;
                    let v3 = Value::new(&mut *frame, 3i8)?;
                    let jl_val = constr.call3(&mut *frame, v1, v2, v3)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<SingleVariant>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.cast::<i8>().unwrap(), 1);

                    let third = jl_val.get_nth_field(&mut *frame, 2).unwrap();
                    assert_eq!(third.cast::<i8>().unwrap(), 3);

                    assert!(jl_val.is::<SingleVariant>());
                    assert!(jl_val.cast::<SingleVariant>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_double_variant() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|global, frame| {
                    let constr = Module::main(global)
                        .submodule("WithBitsUnion")?
                        .function("DoubleVariant")?;

                    let v1 = Value::new(&mut *frame, 1i8)?;
                    let v2 = Value::new(&mut *frame, 2i16)?;
                    let v3 = Value::new(&mut *frame, 3i8)?;
                    let jl_val = constr.call3(&mut *frame, v1, v2, v3)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<DoubleVariant>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.cast::<i8>().unwrap(), 1);

                    let second = jl_val.get_nth_field(&mut *frame, 1).unwrap();
                    assert_eq!(second.cast::<i16>().unwrap(), 2);

                    let third = jl_val.get_nth_field(&mut *frame, 2).unwrap();
                    assert_eq!(third.cast::<i8>().unwrap(), 3);

                    assert!(jl_val.is::<DoubleVariant>());
                    assert!(jl_val.cast::<DoubleVariant>().is_ok());

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
                .dynamic_frame(|global, frame| {
                    let constr = Module::main(global)
                        .submodule("WithBitsUnion")?
                        .function("SizeAlignMismatch")?;

                    let v1 = Value::new(&mut *frame, 1i8)?;
                    let v2 = Value::new(&mut *frame, 2i32)?;
                    let v3 = Value::new(&mut *frame, 3i8)?;
                    let jl_val = constr.call3(&mut *frame, v1, v2, v3)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<SizeAlignMismatch>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.cast::<i8>().unwrap(), 1);

                    let second = jl_val.get_nth_field(&mut *frame, 1).unwrap();
                    assert_eq!(second.cast::<i32>().unwrap(), 2);

                    let third = jl_val.get_nth_field(&mut *frame, 2).unwrap();
                    assert_eq!(third.cast::<i8>().unwrap(), 3);

                    assert!(jl_val.is::<SizeAlignMismatch>());
                    assert!(jl_val.cast::<SizeAlignMismatch>().is_ok());

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
                .dynamic_frame(|global, frame| {
                    let constr = Module::main(global)
                        .submodule("WithBitsUnion")?
                        .function("UnionInTuple")?;

                    let v1 = Value::new(&mut *frame, 1i8)?;
                    let v2 = Value::new(&mut *frame, Tuple1(2i16))?;
                    let v3 = Value::new(&mut *frame, 3i8)?;
                    let jl_val = constr.call3(&mut *frame, v1, v2, v3)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<UnionInTuple>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.cast::<i8>().unwrap(), 1);

                    let second = jl_val.get_nth_field(&mut *frame, 1).unwrap();
                    assert_eq!(second.cast::<Tuple1<i16>>().unwrap(), Tuple1(2));

                    let third = jl_val.get_nth_field(&mut *frame, 2).unwrap();
                    assert_eq!(third.cast::<i8>().unwrap(), 3);

                    assert!(jl_val.is::<UnionInTuple>());
                    let uit = jl_val.cast::<UnionInTuple>()?;
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
                .dynamic_frame(|global, frame| {
                    let constr = Module::main(global)
                        .submodule("WithNonBitsUnion")?
                        .function("NonBitsUnion")?;

                    let v1 = Value::new(&mut *frame, 1i8)?;
                    let jl_val = constr.call1(&mut *frame, v1)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<NonBitsUnion>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.cast::<i8>().unwrap(), 1);

                    assert!(jl_val.is::<NonBitsUnion>());
                    assert!(jl_val.cast::<NonBitsUnion>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_with_generic_t_i32() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|global, frame| {
                    let constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithGenericT")?;

                    let v1 = Value::new(&mut *frame, 1i32)?;
                    let jl_val = constr.call1(&mut *frame, v1)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithGenericT<i32>>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert_eq!(first.cast::<i32>().unwrap(), 1);

                    assert!(jl_val.is::<WithGenericT<i32>>());
                    assert!(jl_val.cast::<WithGenericT<i32>>().is_ok());

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
                .dynamic_frame(|global, frame| {
                    let constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithGenericT")?;

                    let v1 = Value::new(&mut *frame, 1i32)?;
                    let wgt = constr.call1(&mut *frame, v1)?.unwrap();

                    let constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithGenericUnionAll")?;

                    let jl_val = constr.call1(&mut *frame, wgt)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithGenericUnionAll>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first.cast::<WithGenericT<i32>>().is_ok());

                    assert!(jl_val.is::<WithGenericUnionAll>());
                    assert!(jl_val.cast::<WithGenericUnionAll>().is_ok());

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
                .dynamic_frame(|global, frame| {
                    let constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithGenericT")?;

                    let v1 = Value::new(&mut *frame, 1i32)?;
                    let wgt = constr.call1(&mut *frame, v1)?.unwrap();

                    let constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithNestedGenericT")?;

                    let jl_val = constr.call1(&mut *frame, wgt)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithNestedGenericT<i32>>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first.cast::<WithGenericT<i32>>().is_ok());

                    assert!(jl_val.is::<WithNestedGenericT<i32>>());
                    assert!(jl_val.cast::<WithNestedGenericT<i32>>().is_ok());

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
                .dynamic_frame(|global, frame| {
                    let constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithGenericT")?;

                    let wgt = constr
                        .call1(&mut *frame, Module::base(global).into())?
                        .unwrap();

                    let constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithPropagatedLifetime")?;

                    let jl_val = constr.call1(&mut *frame, wgt)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithPropagatedLifetime>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first.cast::<WithGenericT<Module>>().is_ok());

                    assert!(jl_val.is::<WithPropagatedLifetime>());
                    assert!(jl_val.cast::<WithPropagatedLifetime>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_with_propagated_lifetimes() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|global, frame| {
                    let arr = Value::new_array::<i32, _, _, _>(&mut *frame, (2, 2))?;

                    let wgt_constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithGenericT")?;

                    let wgt = wgt_constr.call1(&mut *frame, arr)?.unwrap();

                    let constr = Module::base(global).function("tuple")?;
                    let int = Value::new(&mut *frame, 2i32)?;
                    let tup = constr.call2(&mut *frame, int, wgt)?.unwrap();

                    let a = wgt_constr.call1(&mut *frame, tup)?.unwrap();
                    let constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithPropagatedLifetimes")?;

                    let jl_val = constr.call1(&mut *frame, a)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithPropagatedLifetimes>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first
                        .cast::<WithGenericT<Tuple2<i32, WithGenericT<Array>>>>()
                        .is_ok());

                    assert!(jl_val.is::<WithPropagatedLifetimes>());
                    assert!(jl_val.cast::<WithPropagatedLifetimes>().is_ok());

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
                .dynamic_frame(|global, frame| {
                    let wgt_constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithGenericT")?;

                    let v1 = Value::new(&mut *frame, 1i64)?;
                    let wgt = wgt_constr.call1(&mut *frame, v1)?.unwrap();

                    let constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithSetGeneric")?;

                    let jl_val = constr.call1(&mut *frame, wgt)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithSetGeneric>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first.cast::<WithGenericT<i64>>().is_ok());

                    assert!(jl_val.is::<WithSetGeneric>());
                    assert!(jl_val.cast::<WithSetGeneric>().is_ok());

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
                .dynamic_frame(|global, frame| {
                    let wgt_constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithGenericT")?;

                    let v1 = Value::new(&mut *frame, 1i64)?;
                    let wgt = wgt_constr.call1(&mut *frame, v1)?.unwrap();

                    let tup_constr = Module::base(global).function("tuple")?;
                    let v2 = tup_constr.call1(&mut *frame, wgt)?.unwrap();

                    let constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("WithSetGenericTuple")?;

                    let jl_val = constr.call1(&mut *frame, v2)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithSetGenericTuple>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first.cast::<Tuple1<WithGenericT<i64>>>().is_ok());

                    assert!(jl_val.is::<WithSetGenericTuple>());
                    assert!(jl_val.cast::<WithSetGenericTuple>().is_ok());

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
                .dynamic_frame(|global, frame| {
                    let wvt_constr = Module::main(global)
                        .submodule("WithGeneric")?
                        .function("withvaluetype")?;

                    let v1 = Value::new(&mut *frame, 1i64)?;
                    let jl_val = wvt_constr.call1(&mut *frame, v1)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(&mut *frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<WithValueType>());

                    let first = jl_val.get_nth_field(&mut *frame, 0).unwrap();
                    assert!(first.cast::<i64>().is_ok());

                    assert!(jl_val.is::<WithValueType>());
                    assert!(jl_val.cast::<WithValueType>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }
}
