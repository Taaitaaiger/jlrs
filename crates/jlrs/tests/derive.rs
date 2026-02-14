#[cfg(all(test, feature = "jlrs-derive", feature = "local-rt"))]
mod derive_util;

#[cfg(all(test, feature = "jlrs-derive", feature = "local-rt"))]
mod tests {
    use std::os::raw::c_void;

    use jlrs::{
        data::{
            layout::{
                julia_enum::Enum,
                tuple::{Tuple1, Tuple2},
                valid_layout::{ValidField, ValidLayout},
            },
            types::construct_type::{ConstantBool, ConstructType},
        },
        prelude::*,
    };

    use super::derive_util::{JULIA_DERIVE, derive_impls::*};
    fn derive_bits_type_bool() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeBool { a: Bool::new(true) };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<bool>().unwrap().as_bool(), true);
                    assert!(v.is::<BitsTypeBool>());
                    assert!(v.unbox::<BitsTypeBool>().is_ok());
                })
            })
        })
    }

    fn derive_elided() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let a = HasElidedParam { a: 3.0 };
                    type ElidedParam<T, const U: bool> =
                        HasElidedParamTypeConstructor<T, ConstantBool<U>>;
                    let v = unsafe {
                        Value::try_new_with::<ElidedParam<f64, true>, _, _>(&mut frame, a).unwrap()
                    };

                    assert!(v.is::<HasElidedParam<f64>>());
                    assert!(v.unbox::<HasElidedParam<f64>>().is_ok());
                })
            })
        })
    }

    fn derive_elided_with_ptr() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let main_mod = Module::main(&frame).as_weak();
                    let a = HasElidedParam { a: Some(main_mod) };
                    type ElidedParam<T, const U: bool> =
                        HasElidedParamTypeConstructor<T, ConstantBool<U>>;
                    let v = unsafe {
                        Value::try_new_with::<ElidedParam<Module, true>, _, _>(&mut frame, a)
                            .unwrap()
                    };

                    assert!(v.is::<HasElidedParam<Option<WeakModule>>>());
                    assert!(v.unbox::<HasElidedParam<Option<WeakModule>>>().is_ok());
                })
            })
        })
    }

    fn derive_double_variant() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "DoubleVariant")
                        .unwrap()
                        .as_managed();

                    let v2 = Value::new(&mut frame, 2i16);
                    let jl_val = constr.call(&mut frame, &mut [v2]).unwrap();

                    assert!(jl_val.datatype().is::<DoubleVariant>());

                    let field = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(field.unbox::<i16>().unwrap(), 2);

                    assert!(jl_val.is::<DoubleVariant>());
                    assert!(jl_val.unbox::<DoubleVariant>().is_ok());
                })
            })
        })
    }

    fn rebox_double_variant() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "DoubleVariant")
                        .unwrap()
                        .as_managed();

                    let v2 = Value::new(&mut frame, 2i16);
                    let jl_val = constr.call(&mut frame, &mut [v2]).unwrap();

                    let unboxed = jl_val.unbox::<DoubleVariant>().unwrap();
                    assert!(
                        Value::try_new_with::<DoubleVariant, _, _>(&mut frame, unboxed).is_ok()
                    );
                })
            })
        })
    }

    fn cannot_rebox_as_incompatible() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "DoubleVariant")
                        .unwrap()
                        .as_managed();

                    let v2 = Value::new(&mut frame, 2i16);
                    let jl_val = constr.call(&mut frame, &mut [v2]).unwrap();

                    let unboxed = jl_val.unbox::<DoubleVariant>().unwrap();
                    assert!(
                        Value::try_new_with::<DoubleUVariant, _, _>(&mut frame, unboxed).is_err()
                    );
                })
            })
        })
    }

    fn derive_generic_tu() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let ty = WithGenericTU::<isize, usize>::construct_type(&mut frame);
                    assert_eq!(
                        ty.cast::<DataType>().unwrap().size().unwrap() as usize,
                        std::mem::size_of::<isize>() + std::mem::size_of::<usize>()
                    );
                })
            })
        })
    }

    fn derive_bits_type_char() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeChar { a: Char::new('b') };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<char>().unwrap().try_as_char().unwrap(), 'b');
                    assert!(v.is::<BitsTypeChar>());
                    assert!(v.unbox::<BitsTypeChar>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_uint8() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeUInt8 { a: 1 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<u8>().unwrap(), 1);
                    assert!(v.is::<BitsTypeUInt8>());
                    assert!(v.unbox::<BitsTypeUInt8>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_uint16() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeUInt16 { a: 2 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<u16>().unwrap(), 2);
                    assert!(v.is::<BitsTypeUInt16>());
                    assert!(v.unbox::<BitsTypeUInt16>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_uint32() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeUInt32 { a: 3 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<u32>().unwrap(), 3);
                    assert!(v.is::<BitsTypeUInt32>());
                    assert!(v.unbox::<BitsTypeUInt32>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_uint64() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeUInt64 { a: 4 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<u64>().unwrap(), 4);
                    assert!(v.is::<BitsTypeUInt64>());
                    assert!(v.unbox::<BitsTypeUInt64>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_uint() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeUInt { a: 5 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<usize>().unwrap(), 5);
                    assert!(v.is::<BitsTypeUInt>());
                    assert!(v.unbox::<BitsTypeUInt>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_int8() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeInt8 { a: -1 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<i8>().unwrap(), -1);
                    assert!(v.is::<BitsTypeInt8>());
                    assert!(v.unbox::<BitsTypeInt8>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_int16() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeInt16 { a: -2 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<i16>().unwrap(), -2);
                    assert!(v.is::<BitsTypeInt16>());
                    assert!(v.unbox::<BitsTypeInt16>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_int32() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeInt32 { a: -3 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<i32>().unwrap(), -3);
                    assert!(v.is::<BitsTypeInt32>());
                    assert!(v.unbox::<BitsTypeInt32>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_int64() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeInt64 { a: -4 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<i64>().unwrap(), -4);
                    assert!(v.is::<BitsTypeInt64>());
                    assert!(v.unbox::<BitsTypeInt64>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_int() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeInt { a: -5 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<isize>().unwrap(), -5);
                    assert!(v.is::<BitsTypeInt>());
                    assert!(v.unbox::<BitsTypeInt>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_float32() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeFloat32 { a: 1.2 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<f32>().unwrap(), 1.2);
                    assert!(v.is::<BitsTypeFloat32>());
                    assert!(v.unbox::<BitsTypeFloat32>().is_ok());
                })
            })
        })
    }

    fn derive_bits_type_float64() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsTypeFloat64 { a: -2.3 };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<f64>().unwrap(), -2.3);
                    assert!(v.is::<BitsTypeFloat64>());
                    assert!(v.unbox::<BitsTypeFloat64>().is_ok());
                })
            })
        })
    }

    fn derive_bits_char_float32_float64() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
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
                })
            })
        })
    }

    fn derive_bits_int_bool() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let s = BitsIntBool {
                        a: 1,
                        b: Bool::new(true),
                    };
                    let v = Value::new(&mut frame, s);
                    let first = v.get_nth_field(&mut frame, 0).unwrap();

                    assert_eq!(first.unbox::<isize>().unwrap(), 1);
                    assert!(v.is::<BitsIntBool>());
                    assert!(v.unbox::<BitsIntBool>().is_ok());
                })
            })
        })
    }

    fn derive_bits_char_bits_int_char() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
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
                })
            })
        })
    }

    fn derive_bits_uint8_tuple_int32_int64() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
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
                })
            })
        })
    }

    fn derive_bits_uint8_tuple_int32_tuple_int16_uint16() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let rs_val = BitsUInt8TupleInt32TupleInt16UInt16 {
                        a: 0,
                        b: Tuple2(-1, Tuple2(-1, 3)),
                    };
                    let jl_val = Value::new(&mut frame, rs_val.clone());

                    unsafe {
                        assert!(
                            Module::base(&frame)
                                .global(&frame, "typeof")
                                .unwrap()
                                .as_managed()
                                .call(&mut frame, [jl_val])
                                .unwrap()
                                .cast::<DataType>()
                                .unwrap()
                                .is::<BitsUInt8TupleInt32TupleInt16UInt16>()
                        );
                    }

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(first.unbox::<u8>().unwrap(), (&rs_val).a);

                    let second = jl_val.get_nth_field(&mut frame, 1).unwrap();
                    assert_eq!(
                        second.unbox::<Tuple2<i32, Tuple2<i16, u16>>>().unwrap(),
                        rs_val.b
                    );

                    assert!(jl_val.is::<BitsUInt8TupleInt32TupleInt16UInt16>());
                    assert!(
                        jl_val
                            .unbox::<BitsUInt8TupleInt32TupleInt16UInt16>()
                            .is_ok()
                    );
                })
            })
        })
    }

    fn derive_single_variant() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "SingleVariant")
                        .unwrap()
                        .as_managed();
                    let v2 = Value::new(&mut frame, 2i32);
                    let jl_val = constr.call(&mut frame, [v2]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<SingleVariant>()
                    );

                    assert!(jl_val.is::<SingleVariant>());
                    assert!(jl_val.unbox::<SingleVariant>().is_ok());
                })
            })
        })
    }

    fn derive_size_align_mismatch() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "SizeAlignMismatch")
                        .unwrap()
                        .as_managed();

                    let v2 = Value::new(&mut frame, 2i32);
                    let jl_val = constr.call(&mut frame, [v2]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<SizeAlignMismatch>()
                    );

                    let second = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(second.unbox::<i32>().unwrap(), 2);

                    assert!(jl_val.is::<SizeAlignMismatch>());
                    assert!(jl_val.unbox::<SizeAlignMismatch>().is_ok());
                })
            })
        })
    }

    fn derive_union_in_tuple() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "UnionInTuple")
                        .unwrap()
                        .as_managed();

                    let v2 = Value::new(&mut frame, Tuple1(2i32));
                    let jl_val = constr.call(&mut frame, [v2]).unwrap();

                    let second = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(second.unbox::<Tuple1<i32>>().unwrap(), Tuple1(2));

                    let _uit = jl_val.unbox::<UnionInTuple>().unwrap();
                })
            })
        })
    }

    fn derive_non_bits_union() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "NonBitsUnion")
                        .unwrap()
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i8);
                    let jl_val = constr.call(&mut frame, [v1]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<NonBitsUnion>()
                    );

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(first.unbox::<i8>().unwrap(), 1);

                    assert!(jl_val.is::<NonBitsUnion>());
                    assert!(jl_val.unbox::<NonBitsUnion>().is_ok());
                })
            })
        })
    }

    fn derive_string() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "WithString")
                        .unwrap()
                        .as_managed();

                    let s = JuliaString::new(&mut frame, "foo");
                    let jl_val = constr.call(&mut frame, [s.as_value()]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<WithString>()
                    );

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(first.unbox::<String>().unwrap().unwrap(), "foo");

                    assert!(jl_val.is::<WithString>());
                    assert!(jl_val.unbox::<WithString>().is_ok());
                })
            })
        })
    }

    fn derive_with_generic_t_i32() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")
                        .unwrap()
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i32);
                    let jl_val = constr.call(&mut frame, [v1]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<WithGenericT<i32>>()
                    );

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(first.unbox::<i32>().unwrap(), 1);

                    assert!(jl_val.is::<WithGenericT<i32>>());
                    assert!(jl_val.unbox::<WithGenericT<i32>>().is_ok());
                })
            })
        })
    }

    fn derive_with_unionall() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")
                        .unwrap()
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i32);
                    let wgt = constr.call(&mut frame, [v1]).unwrap();

                    let constr = Module::main(&frame)
                        .global(&frame, "WithGenericUnionAll")
                        .unwrap()
                        .as_managed();

                    let jl_val = constr.call(&mut frame, [wgt]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<WithGenericUnionAll>()
                    );

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<i32>>().is_ok());

                    assert!(jl_val.is::<WithGenericUnionAll>());
                    assert!(jl_val.unbox::<WithGenericUnionAll>().is_ok());
                })
            })
        })
    }

    fn derive_with_nested_generic() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")
                        .unwrap()
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i32);
                    let wgt = constr.call(&mut frame, [v1]).unwrap();

                    let constr = Module::main(&frame)
                        .global(&frame, "WithNestedGenericT")
                        .unwrap()
                        .as_managed();

                    let jl_val = constr.call(&mut frame, [wgt]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<WithNestedGenericT<i32>>()
                    );

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<i32>>().is_ok());

                    assert!(jl_val.is::<WithNestedGenericT<i32>>());
                    assert!(jl_val.unbox::<WithNestedGenericT<i32>>().is_ok());
                })
            })
        })
    }

    fn derive_with_propagated_lifetime() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let global = frame.unrooted();
                    let constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")
                        .unwrap()
                        .as_managed();

                    let wgt = constr
                        .call(&mut frame, [Module::base(&global).as_value()])
                        .unwrap();

                    let constr = Module::main(&frame)
                        .global(&frame, "WithPropagatedLifetime")
                        .unwrap()
                        .as_managed();

                    let jl_val = constr.call(&mut frame, [wgt]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<WithPropagatedLifetime>()
                    );

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<Option<WeakModule>>>().is_ok());

                    assert!(jl_val.is::<WithPropagatedLifetime>());
                    assert!(jl_val.unbox::<WithPropagatedLifetime>().is_ok());
                })
            })
        })
    }

    fn derive_with_propagated_lifetimes() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let arr = TypedArray::<i32>::new(&mut frame, [2, 2]).unwrap();

                    let wgt_constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")
                        .unwrap()
                        .as_managed();

                    let wgt = wgt_constr.call(&mut frame, [arr.as_value()]).unwrap();

                    let constr = Module::base(&frame)
                        .global(&frame, "tuple")
                        .unwrap()
                        .as_managed();
                    let int = Value::new(&mut frame, 2i32);
                    let tup = constr.call(&mut frame, [int, wgt]).unwrap();

                    let a = wgt_constr.call(&mut frame, [tup]).unwrap();
                    let constr = Module::main(&frame)
                        .global(&frame, "WithPropagatedLifetimes")
                        .unwrap()
                        .as_managed();

                    let jl_val = constr.call(&mut frame, [a]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<WithPropagatedLifetimes>()
                    );

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(
                        first
                            .unbox::<WithGenericT<Tuple2<i32, WithGenericT<Option<WeakArray>>>>>()
                            .is_ok()
                    );

                    assert!(jl_val.is::<WithPropagatedLifetimes>());
                    assert!(jl_val.unbox::<WithPropagatedLifetimes>().is_ok());
                })
            })
        })
    }

    fn derive_with_set_generic() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let wgt_constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")
                        .unwrap()
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i64);
                    let wgt = wgt_constr.call(&mut frame, [v1]).unwrap();

                    let constr = Module::main(&frame)
                        .global(&frame, "WithSetGeneric")
                        .unwrap()
                        .as_managed();

                    let jl_val = constr.call(&mut frame, [wgt]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<WithSetGeneric>()
                    );

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(first.unbox::<WithGenericT<i64>>().is_ok());

                    assert!(jl_val.is::<WithSetGeneric>());
                    assert!(jl_val.unbox::<WithSetGeneric>().is_ok());
                })
            })
        })
    }

    fn derive_with_set_generic_tuple() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let wgt_constr = Module::main(&frame)
                        .global(&frame, "WithGenericT")
                        .unwrap()
                        .as_managed();

                    let v1 = Value::new(&mut frame, 1i64);
                    let wgt = wgt_constr.call(&mut frame, [v1]).unwrap();

                    let tup_constr = Module::base(&frame)
                        .global(&frame, "tuple")
                        .unwrap()
                        .as_managed();
                    let v2 = tup_constr.call(&mut frame, [wgt]).unwrap();

                    let constr = Module::main(&frame)
                        .global(&frame, "WithSetGenericTuple")
                        .unwrap()
                        .as_managed();

                    let jl_val = constr.call(&mut frame, [v2]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<WithSetGenericTuple>()
                    );

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    first.unbox::<Tuple1<WithGenericT<i64>>>().unwrap();

                    assert!(jl_val.is::<WithSetGenericTuple>());
                    assert!(jl_val.unbox::<WithSetGenericTuple>().is_ok());
                })
            })
        })
    }

    fn derive_with_value_type() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let b = Value::new(&mut frame, true);
                    let wvt_constr = Module::main(&frame)
                        .global(&frame, "WithValueType")
                        .unwrap()
                        .as_managed()
                        .apply_type(&mut frame, [b])
                        .unwrap();

                    let v1 = Value::new(&mut frame, 1i64);
                    let jl_val = wvt_constr.call(&mut frame, [v1]).unwrap();

                    assert!(
                        Module::base(&frame)
                            .global(&frame, "typeof")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [jl_val])
                            .unwrap()
                            .cast::<DataType>()
                            .unwrap()
                            .is::<WithValueType>()
                    );

                    let first = jl_val.get_nth_field(&mut frame, 0).unwrap();
                    assert!(first.unbox::<i64>().is_ok());

                    assert!(jl_val.is::<WithValueType>());
                    assert!(jl_val.unbox::<WithValueType>().is_ok());
                })
            })
        })
    }

    fn derive_zero_sized() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let v = Value::new(&mut frame, Empty {});
                    assert!(v.unbox::<Empty>().is_ok());
                })
            })
        })
    }

    fn isbits_into_julia() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let wvt = WithValueType { a: 1 };
                    type WVT = WithValueTypeTypeConstructor<ConstantBool<true>>;
                    let v = Value::new_bits_from_layout::<WVT, _>(&mut frame, wvt.clone()).unwrap();
                    v.unbox::<WithValueType>().unwrap();
                })
            })
        })
    }

    fn trivial_isbits_into_julia() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let layout = WithGenericTU { a: 1i32, b: 2u32 };
                    let v = Value::new_bits(&mut frame, layout.clone());
                    let layout_unboxed = v.unbox::<WithGenericTU<i32, u32>>().unwrap();
                    assert_eq!(layout, layout_unboxed);
                })
            })
        })
    }

    fn with_u128() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow().local_scope::<_, 4>(|mut frame| {
                let a = -1;
                let b = 0xffffffffffffffff1;
                let c = -2;

                let with_u128 = WithU128::new(a, b, c);
                let with_u128_v = Value::new(&mut frame, with_u128);
                assert_eq!(with_u128_v.datatype().size().unwrap() as usize, size_of::<WithU128>());

                let a_v = with_u128_v.get_field(&mut frame, "a").unwrap();
                assert_eq!(a_v.unbox::<i8>().unwrap(), a);

                let b_v = with_u128_v.get_field(&mut frame, "b").unwrap();
                assert_eq!(b_v.unbox::<u128>().unwrap(), b);

                let c_v = with_u128_v.get_field(&mut frame, "c").unwrap();
                assert_eq!(c_v.unbox::<i8>().unwrap(), c);

                let unboxed = with_u128_v.unbox::<WithU128>().unwrap();
                assert_eq!(unboxed.a(), a);
                assert_eq!(unboxed.b(), b);
                assert_eq!(unboxed.c(), c);
            });
        })
    }

    fn with_i128() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow().local_scope::<_, 4>(|mut frame| {
                let a = -1;
                let b = -0xffffffffffffffff1;
                let c = -2;

                let with_i128 = WithI128::new(a, b, c);
                let with_i128_v = Value::new(&mut frame, with_i128);
                assert_eq!(with_i128_v.datatype().size().unwrap() as usize, size_of::<WithI128>());

                let a_v = with_i128_v.get_field(&mut frame, "a").unwrap();
                assert_eq!(a_v.unbox::<i8>().unwrap(), a);

                let b_v = with_i128_v.get_field(&mut frame, "b").unwrap();
                assert_eq!(b_v.unbox::<i128>().unwrap(), b);

                let c_v = with_i128_v.get_field(&mut frame, "c").unwrap();
                assert_eq!(c_v.unbox::<i8>().unwrap(), c);

                let unboxed = with_i128_v.unbox::<WithI128>().unwrap();
                assert_eq!(unboxed.a(), a);
                assert_eq!(unboxed.b(), b);
                assert_eq!(unboxed.c(), c);
            });
        })
    }

    fn test_enums() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    // Test IntoJulia, Typecheck, ValidLayout, ValidField and Unbox for each variant.
                    let mut test_fn = |layout| {
                        let v = Value::new(&mut frame, layout);

                        assert!(v.is::<StandardEnum>());
                        assert!(StandardEnum::valid_layout(v.datatype().as_value()));
                        assert!(StandardEnum::valid_field(v.datatype().as_value()));

                        let layout_unboxed = v.unbox::<StandardEnum>().unwrap();
                        assert_eq!(layout, layout_unboxed);
                    };

                    test_fn(StandardEnum::SeA);
                    test_fn(StandardEnum::SeB);
                    test_fn(StandardEnum::SeC);
                })
            })
        })
    }

    fn test_enums_ccall() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
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

                    let res = unsafe { func.call(&mut frame, [echo_v, se_a]) }.unwrap();
                    assert_eq!(se_a, res);
                })
            })
        })
    }

    fn derive_complex_bits_union() {
        JULIA_DERIVE.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let elided_ua = Module::main(&frame)
                        .global(&frame, "Elided")
                        .unwrap()
                        .as_managed();

                    let true_v = Value::true_v(&frame);
                    let one = Value::new(&mut frame, 1i64);

                    let i64_ty = i64::construct_type(&frame).as_managed();
                    let inner_ctor = elided_ua.apply_type(&mut frame, [one, i64_ty]).unwrap();

                    let outer_ctor = elided_ua
                        .apply_type(&mut frame, [true_v, inner_ctor])
                        .unwrap();
                    let inner = inner_ctor.call(&mut frame, [one]).unwrap();
                    let outer = outer_ctor.call(&mut frame, [inner]).unwrap();

                    let constr = Module::main(&frame)
                        .global(&frame, "WithElidedInUnion")
                        .unwrap()
                        .as_managed();

                    let data = constr.call(&mut frame, [outer]).unwrap();
                    let content = data.get_nth_field(&mut frame, 0).unwrap();
                    assert_eq!(content.unbox::<Elided<Elided<i64>>>().unwrap().a.a, 1);

                    assert!(data.is::<WithElidedInUnion>());
                    assert!(data.unbox::<WithElidedInUnion>().is_ok());
                })
            })
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
        derive_complex_bits_union();
        with_u128();
        with_i128();
    }
}
