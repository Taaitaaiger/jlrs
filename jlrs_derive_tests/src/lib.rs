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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<bool>().unwrap(), true);
                    assert!(v.is::<BitsTypeBool>());
                    assert_eq!(v.cast::<BitsTypeBool>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<char>().unwrap(), 'b');
                    assert!(v.is::<BitsTypeChar>());
                    assert_eq!(v.cast::<BitsTypeChar>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<u8>().unwrap(), 1);
                    assert!(v.is::<BitsTypeUInt8>());
                    assert_eq!(v.cast::<BitsTypeUInt8>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<u16>().unwrap(), 2);
                    assert!(v.is::<BitsTypeUInt16>());
                    assert_eq!(v.cast::<BitsTypeUInt16>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<u32>().unwrap(), 3);
                    assert!(v.is::<BitsTypeUInt32>());
                    assert_eq!(v.cast::<BitsTypeUInt32>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<u64>().unwrap(), 4);
                    assert!(v.is::<BitsTypeUInt64>());
                    assert_eq!(v.cast::<BitsTypeUInt64>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<u64>().unwrap(), 5);
                    assert!(v.is::<BitsTypeUInt>());
                    assert_eq!(v.cast::<BitsTypeUInt>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<i8>().unwrap(), -1);
                    assert!(v.is::<BitsTypeInt8>());
                    assert_eq!(v.cast::<BitsTypeInt8>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<i16>().unwrap(), -2);
                    assert!(v.is::<BitsTypeInt16>());
                    assert_eq!(v.cast::<BitsTypeInt16>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<i32>().unwrap(), -3);
                    assert!(v.is::<BitsTypeInt32>());
                    assert_eq!(v.cast::<BitsTypeInt32>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<i64>().unwrap(), -4);
                    assert!(v.is::<BitsTypeInt64>());
                    assert_eq!(v.cast::<BitsTypeInt64>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<i64>().unwrap(), -5);
                    assert!(v.is::<BitsTypeInt>());
                    assert_eq!(v.cast::<BitsTypeInt>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<f32>().unwrap(), 1.2);
                    assert!(v.is::<BitsTypeFloat32>());
                    assert_eq!(v.cast::<BitsTypeFloat32>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<f64>().unwrap(), -2.3);
                    assert!(v.is::<BitsTypeFloat64>());
                    assert_eq!(v.cast::<BitsTypeFloat64>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<char>().unwrap(), 'a');
                    assert!(v.is::<BitsCharFloat32Float64>());
                    assert_eq!(v.cast::<BitsCharFloat32Float64>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<i64>().unwrap(), 1);
                    assert!(v.is::<BitsIntBool>());
                    assert_eq!(v.cast::<BitsIntBool>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();
                    assert_eq!(first.cast::<char>().unwrap(), 'a');
                    assert!(v.is::<BitsCharBitsIntChar>());
                    assert_eq!(v.cast::<BitsCharBitsIntChar>().unwrap(), s);

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
                    let v = Value::new(frame, s).unwrap();

                    let first = v.get_nth_field(frame, 0).unwrap();
                    let second = v.get_nth_field(frame, 1).unwrap();

                    assert_eq!(first.cast::<u8>().unwrap(), 0);
                    assert_eq!(second.cast::<Tuple2<i32, i64>>().unwrap(), Tuple2(-1, -3));
                    assert!(v.is::<BitsUInt8TupleInt32Int64>());
                    assert_eq!(v.cast::<BitsUInt8TupleInt32Int64>().unwrap(), s);

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
                    let jl_val = Value::new(frame, rs_val).unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<BitsUInt8TupleInt32TupleInt16UInt16>());

                    let first = jl_val.get_nth_field(frame, 0).unwrap();
                    assert_eq!(first.cast::<u8>().unwrap(), rs_val.a);

                    let second = jl_val.get_nth_field(frame, 1).unwrap();
                    assert_eq!(
                        second.cast::<Tuple2<i32, Tuple2<i16, u16>>>().unwrap(),
                        rs_val.b
                    );

                    assert!(jl_val.is::<BitsUInt8TupleInt32TupleInt16UInt16>());
                    assert_eq!(
                        jl_val
                            .cast::<BitsUInt8TupleInt32TupleInt16UInt16>()
                            .unwrap(),
                        rs_val
                    );

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
                    let v1 = Value::new(frame, 1i8)?;
                    let v2 = Value::new(frame, 2i32)?;
                    let v3 = Value::new(frame, 3i8)?;
                    let jl_val = constr.call3(frame, v1, v2, v3)?.unwrap();

                    assert!(Module::base(global)
                        .function("typeof")?
                        .call1(frame, jl_val)?
                        .unwrap()
                        .cast::<DataType>()?
                        .is::<SingleVariant>());

                    let first = jl_val.get_nth_field(frame, 0).unwrap();
                    assert_eq!(first.cast::<i8>().unwrap(), 1);

                    let third = jl_val.get_nth_field(frame, 2).unwrap();
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

                    let v1 = Value::new(frame, 1i8)?;
                    let v2 = Value::new(frame, 2i16)?;
                    let v3 = Value::new(frame, 3i8)?;
                    let jl_val = constr.call3(frame, v1, v2, v3)?.unwrap();

                    assert!(
                        Module::base(global)
                            .function("typeof")?
                            .call1(frame, jl_val)?
                            .unwrap()
                            .cast::<DataType>()?
                            .is::<DoubleVariant>()
                    );

                    let first = jl_val.get_nth_field(frame, 0).unwrap();
                    assert_eq!(first.cast::<i8>().unwrap(), 1);

                    let second = jl_val.get_nth_field(frame, 1).unwrap();
                    assert_eq!(second.cast::<i16>().unwrap(), 2);

                    let third = jl_val.get_nth_field(frame, 2).unwrap();
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

                    let v1 = Value::new(frame, 1i8)?;
                    let v2 = Value::new(frame, 2i32)?;
                    let v3 = Value::new(frame, 3i8)?;
                    let jl_val = constr.call3(frame, v1, v2, v3)?.unwrap();

                    assert!(
                        Module::base(global)
                            .function("typeof")?
                            .call1(frame, jl_val)?
                            .unwrap()
                            .cast::<DataType>()?
                            .is::<SizeAlignMismatch>()
                    );

                    let first = jl_val.get_nth_field(frame, 0).unwrap();
                    assert_eq!(first.cast::<i8>().unwrap(), 1);

                    let second = jl_val.get_nth_field(frame, 1).unwrap();
                    assert_eq!(second.cast::<i32>().unwrap(), 2);

                    let third = jl_val.get_nth_field(frame, 2).unwrap();
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

                    let v1 = Value::new(frame, 1i8)?;
                    let v2 = Value::new(frame, Tuple1(2i16))?;
                    let v3 = Value::new(frame, 3i8)?;
                    let jl_val = constr.call3(frame, v1, v2, v3)?.unwrap();

                    assert!(
                        Module::base(global)
                            .function("typeof")?
                            .call1(frame, jl_val)?
                            .unwrap()
                            .cast::<DataType>()?
                            .is::<UnionInTuple>()
                    );

                    let first = jl_val.get_nth_field(frame, 0).unwrap();
                    assert_eq!(first.cast::<i8>().unwrap(), 1);

                    let second = jl_val.get_nth_field(frame, 1).unwrap();
                    assert_eq!(second.cast::<Tuple1<i16>>().unwrap(), Tuple1(2));

                    let third = jl_val.get_nth_field(frame, 2).unwrap();
                    assert_eq!(third.cast::<i8>().unwrap(), 3);

                    assert!(jl_val.is::<UnionInTuple>());
                    assert!(jl_val.cast::<UnionInTuple>().is_ok());

                    Ok(())
                })
                .unwrap()
        })
    }
}
