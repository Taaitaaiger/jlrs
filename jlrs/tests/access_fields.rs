mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use std::sync::atomic::Ordering;

    use jlrs::{
        convert::to_symbol::ToSymbol,
        data::{
            layout::{
                tuple::{Tuple, Tuple0},
                union::EmptyUnion,
            },
            types::typecheck::Mutable,
        },
        prelude::*,
    };

    use super::util::{JULIA, MIXED_BAG_JL};

    fn empty_union_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let mut tys = [Value::new(&mut frame, 0usize)];
                    let res = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "WithEmpty")
                        .unwrap()
                        .as_managed()
                        .apply_type(&mut frame, &mut tys)
                        .unwrap()
                        .call(&mut frame, [])
                        .unwrap();

                    assert!(res
                        .field_accessor()
                        .field(1)
                        .unwrap()
                        .access::<EmptyUnion>()
                        .is_err());
                })
            });
        })
    }

    fn access_tuple_fields() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "inlinetuple")
                        .unwrap()
                        .as_managed();
                    let tup = func.call(&mut frame, []).unwrap();

                    assert!(tup.is::<Tuple>());
                    assert_eq!(tup.n_fields(), 3);
                    let v1 = tup.get_nth_field(&mut frame, 0).unwrap();
                    let v2 = tup.get_nth_field(&mut frame, 1).unwrap();
                    let output = frame.output();
                    let v3 = frame.scope(|_| tup.get_nth_field(output, 2)).unwrap();

                    assert!(v1.is::<u32>());
                    assert!(v2.is::<u16>());
                    assert!(v3.is::<i64>());
                })
            })
        })
    }

    fn cannot_access_oob_tuple_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "inlinetuple")
                        .unwrap()
                        .as_managed();
                    let tup = func.call(&mut frame, []).unwrap();
                    assert!(tup.get_nth_field(&mut frame, 3).is_err());
                })
            })
        })
    }

    fn access_non_pointer_tuple_field_must_alloc() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "inlinetuple")
                        .unwrap()
                        .as_managed();
                    let tup = func.call(&mut frame, []).unwrap();
                    assert!(tup.get_nth_field_ref(2).is_err());
                })
            })
        })
    }

    fn access_mutable_struct_fields() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    //mutable struct MutableStruct
                    //  x
                    //  y::UInt64
                    //end
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "MutableStruct")
                        .unwrap()
                        .as_managed();

                    let x = Value::new(&mut frame, 2.0f32);
                    let y = Value::new(&mut frame, 3u64);

                    let mut_struct = func.call(&mut frame, &mut [x, y]).unwrap();
                    assert!(mut_struct.is::<Mutable>());

                    assert!(mut_struct.get_field(&mut frame, "x").is_ok());
                    let x_val = mut_struct.get_field_ref("x");
                    assert!(x_val.is_ok());
                    {
                        assert!(x_val.unwrap().unwrap().as_managed().is::<f32>());
                    }
                    let output = frame.output();
                    let _ = frame.scope(|_| mut_struct.get_field(output, "y")).unwrap();
                    assert!(mut_struct.get_field_ref("y").is_err());
                })
            })
        })
    }

    fn cannot_access_unknown_mutable_struct_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    //mutable struct MutableStruct
                    //  x
                    //  y::UInt64
                    //end
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "MutableStruct")
                        .unwrap()
                        .as_managed();

                    let x = Value::new(&mut frame, 2.0f32);
                    let y = Value::new(&mut frame, 3u64);

                    let mut_struct = func.call(&mut frame, &mut [x, y]).unwrap();
                    assert!(mut_struct.is::<Mutable>());

                    assert!(mut_struct.get_field(&mut frame, "z").is_err());
                })
            })
        })
    }

    fn access_bounds_error_fields() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let oob_idx = stack
                    .scope(|mut frame| unsafe {
                        let idx = Value::new(&mut frame, 4usize);
                        let data = vec![1.0f64, 2., 3.];
                        let array = TypedArray::<f64>::from_vec_unchecked(&mut frame, data, 3);
                        let func = Module::base(&frame)
                            .global(&frame, "getindex")
                            .unwrap()
                            .as_managed();
                        let out = func.call(&mut frame, [array.as_value(), idx]).unwrap_err();

                        assert_eq!(out.datatype_name(), "BoundsError");

                        let field_names = out.field_names();
                        let f0: String = field_names[0].as_string().unwrap();
                        assert_eq!(f0, "a");
                        let f1: String = field_names[1].as_string().unwrap();
                        assert_eq!(f1, "i");

                        out.get_field_ref("a").unwrap();

                        out.get_field(&mut frame, field_names[1])
                            .unwrap()
                            .get_nth_field(&mut frame, 0)
                            .unwrap()
                            .unbox::<isize>()
                    })
                    .unwrap();

                assert_eq!(oob_idx, 4);
            });
        });
    }

    fn access_bounds_error_fields_oob() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let idx = Value::new(&mut frame, 4usize);
                    let data = vec![1.0f64, 2., 3.];
                    let array = TypedArray::<f64>::from_vec(&mut frame, data, 3)
                        .unwrap()
                        .unwrap();

                    let func = Module::base(&frame)
                        .global(&frame, "getindex")
                        .unwrap()
                        .as_managed();
                    let out = func.call(&mut frame, [array.as_value(), idx]).unwrap_err();

                    let field_names = out.field_names();
                    assert!(out
                        .get_field(&mut frame, field_names[1])
                        .unwrap()
                        .get_nth_field(&mut frame, 123)
                        .is_err());
                })
            });
        });
    }

    fn access_bounds_error_fields_output() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let idx = Value::new(&mut frame, 4usize);
                    let data = vec![1.0f64, 2., 3.];
                    let ty = DataType::float64_type(&frame);
                    let array = Array::from_vec_for_unchecked(&mut frame, ty.as_value(), data, 3);
                    let func = Module::base(&frame)
                        .global(&frame, "getindex")
                        .unwrap()
                        .as_managed();
                    let out = func.call(&mut frame, [array.as_value(), idx]).unwrap_err();

                    let field_names = out.field_names();
                    let output = frame.output();
                    let _ = frame
                        .scope(|mut frame| {
                            let field = out.get_field(&mut frame, field_names[1]).unwrap();
                            field.get_nth_field(output, 0)
                        })
                        .unwrap();
                })
            });
        });
    }

    fn access_bounds_error_fields_output_oob() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let idx = Value::new(&mut frame, 4usize);
                    let data = vec![1.0f64, 2., 3.];
                    let array = TypedArray::<f64>::from_vec_unchecked(&mut frame, data, 3);
                    let func = Module::base(&frame)
                        .global(&frame, "getindex")
                        .unwrap()
                        .as_managed();
                    let out = func.call(&mut frame, [array.as_value(), idx]).unwrap_err();

                    let field_names = out.field_names();
                    let output = frame.output();
                    let _ = frame
                        .scope(|mut frame| {
                            let field = out.get_field(&mut frame, field_names[1]).unwrap();
                            field.get_nth_field(output, 123)
                        })
                        .unwrap_err();
                })
            });
        });
    }

    fn access_nested_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let value = Value::eval_string(&mut frame, MIXED_BAG_JL)
                        .unwrap()
                        .cast::<Module>()
                        .unwrap()
                        .global(&frame, "mixedbag")
                        .unwrap()
                        .as_managed();

                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")
                            .unwrap()
                            .field("mutable_unions")
                            .unwrap()
                            .field("bits_union")
                            .unwrap()
                            .access::<i32>()
                            .unwrap();

                        assert_eq!(field, 3);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")
                            .unwrap()
                            .field("mutable_unions")
                            .unwrap()
                            .atomic_field("atomic_union", Ordering::Relaxed)
                            .unwrap()
                            .access::<i64>()
                            .unwrap();

                        assert_eq!(field, 5);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")
                            .unwrap()
                            .field("mutable_unions")
                            .unwrap()
                            .field("normal_union")
                            .unwrap()
                            .access::<Nothing>()
                            .unwrap();

                        assert_eq!(field, Nothing);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")
                            .unwrap()
                            .field("immutable_unions")
                            .unwrap()
                            .field("bits_union")
                            .unwrap()
                            .access::<i64>()
                            .unwrap();

                        assert_eq!(field, 7);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")
                            .unwrap()
                            .field("immutable_unions")
                            .unwrap()
                            .field("normal_union")
                            .unwrap()
                            .access::<WeakModule>()
                            .unwrap();

                        assert_eq!(field.as_managed(), Module::main(&frame));
                    }

                    {
                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("i8")
                                .unwrap()
                                .access::<i8>()
                                .unwrap();

                            assert_eq!(field, 1);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .atomic_field("i16", Ordering::Acquire)
                                .unwrap()
                                .access::<i16>()
                                .unwrap();

                            assert_eq!(field, 2);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("i24")
                                .unwrap()
                                .field(0)
                                .unwrap()
                                .access::<i8>()
                                .unwrap();

                            assert_eq!(field, 3);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("i48")
                                .unwrap()
                                .field(2)
                                .unwrap()
                                .access::<i8>()
                                .unwrap();

                            assert_eq!(field, 8);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("i72")
                                .unwrap()
                                .field(1)
                                .unwrap()
                                .access::<i8>()
                                .unwrap();

                            assert_eq!(field, 13);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("ptr")
                                .unwrap()
                                .access::<WeakModule>()
                                .unwrap();

                            assert_eq!(field.as_managed(), Module::main(&frame));
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("wrapped_ptr")
                                .unwrap()
                                .field([0])
                                .unwrap()
                                .access::<WeakModule>()
                                .unwrap();

                            assert_eq!(field.as_managed(), Module::base(&frame));
                        }
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")
                            .unwrap()
                            .field("number")
                            .unwrap()
                            .access::<f64>()
                            .unwrap();

                        assert_eq!(field, 3.0);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")
                            .unwrap()
                            .field("mutable_unions")
                            .unwrap()
                            .field("bits_union")
                            .unwrap()
                            .access::<i32>()
                            .unwrap();

                        assert_eq!(field, -3);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")
                            .unwrap()
                            .field("mutable_unions")
                            .unwrap()
                            .atomic_field("atomic_union", Ordering::Relaxed)
                            .unwrap()
                            .access::<i64>()
                            .unwrap();

                        assert_eq!(field, -5);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")
                            .unwrap()
                            .field("mutable_unions")
                            .unwrap()
                            .field("normal_union")
                            .unwrap()
                            .access::<WeakModule>()
                            .unwrap();

                        assert_eq!(field.as_managed(), Module::main(&frame));
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")
                            .unwrap()
                            .field("immutable_unions")
                            .unwrap()
                            .field("bits_union")
                            .unwrap()
                            .access::<i64>()
                            .unwrap();

                        assert_eq!(field, -7);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")
                            .unwrap()
                            .field("immutable_unions")
                            .unwrap()
                            .field("normal_union")
                            .unwrap()
                            .access::<Nothing>()
                            .unwrap();

                        assert_eq!(field, Nothing);
                    }

                    {
                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("i8")
                                .unwrap()
                                .access::<i8>()
                                .unwrap();

                            assert_eq!(field, -1);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .atomic_field("i16", Ordering::Acquire)
                                .unwrap()
                                .access::<i16>()
                                .unwrap();

                            assert_eq!(field, -2);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("i24")
                                .unwrap()
                                .field(0)
                                .unwrap()
                                .access::<i8>()
                                .unwrap();

                            assert_eq!(field, -3);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("i48")
                                .unwrap()
                                .field(2)
                                .unwrap()
                                .access::<i8>()
                                .unwrap();

                            assert_eq!(field, -8);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("i72")
                                .unwrap()
                                .field(1)
                                .unwrap()
                                .access::<i8>()
                                .unwrap();

                            assert_eq!(field, -13);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("ptr")
                                .unwrap()
                                .access::<WeakModule>()
                                .unwrap();

                            assert_eq!(field.as_managed(), Module::main(&frame));
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")
                                .unwrap()
                                .field("atomics")
                                .unwrap()
                                .field("wrapped_ptr")
                                .unwrap()
                                .field(0)
                                .unwrap()
                                .access::<WeakModule>()
                                .unwrap();

                            assert_eq!(field.as_managed(), Module::base(&frame));
                        }
                    }
                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")
                            .unwrap()
                            .field("number")
                            .unwrap()
                            .access::<i16>()
                            .unwrap();

                        assert_eq!(field, -3);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("tuples")
                            .unwrap()
                            .field("empty")
                            .unwrap()
                            .access::<Tuple0>()
                            .unwrap();

                        assert_eq!(field, Tuple0());
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("tuples")
                            .unwrap()
                            .field("single")
                            .unwrap()
                            .field(0)
                            .unwrap()
                            .access::<i32>()
                            .unwrap();

                        assert_eq!(field, 1);
                    }

                    {
                        let s = JuliaString::new(&mut frame, "double");
                        let field = value
                            .field_accessor()
                            .field("tuples")
                            .unwrap()
                            .field(s)
                            .unwrap()
                            .field(1)
                            .unwrap()
                            .access::<i64>()
                            .unwrap();

                        assert_eq!(field, -4);
                    }

                    {
                        let s = "double".to_symbol(&frame);
                        let field = value
                            .field_accessor()
                            .field("tuples")
                            .unwrap()
                            .field(s)
                            .unwrap()
                            .field(1)
                            .unwrap()
                            .access::<i64>()
                            .unwrap();

                        assert_eq!(field, -4);
                    }

                    {
                        assert!(value
                            .field_accessor()
                            .field("tuples")
                            .unwrap()
                            .field("double")
                            .unwrap()
                            .field([1, 1])
                            .is_err());
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("tuples")
                            .unwrap()
                            .field("abstract")
                            .unwrap()
                            .field(1)
                            .unwrap()
                            .access::<f64>()
                            .unwrap();

                        assert_eq!(field, 4.0);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")
                            .unwrap()
                            .field("u8vec")
                            .unwrap()
                            .field(1)
                            .unwrap()
                            .access::<u8>()
                            .unwrap();

                        assert_eq!(field, 2);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")
                            .unwrap()
                            .field("unionvec")
                            .unwrap()
                            .field(0)
                            .unwrap()
                            .access::<u8>()
                            .unwrap();

                        assert_eq!(field, 1);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")
                            .unwrap()
                            .field("unionvec")
                            .unwrap()
                            .field(1)
                            .unwrap()
                            .access::<u16>()
                            .unwrap();

                        assert_eq!(field, 2);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")
                            .unwrap()
                            .field("wrappervec")
                            .unwrap()
                            .field(1)
                            .unwrap()
                            .access::<WeakModule>()
                            .unwrap();

                        assert_eq!(field.as_managed(), Module::base(&frame));
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")
                            .unwrap()
                            .field("ptrvec")
                            .unwrap()
                            .field(1)
                            .unwrap()
                            .field(0)
                            .unwrap()
                            .access::<f32>()
                            .unwrap();

                        assert_eq!(field, 2.0);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")
                            .unwrap()
                            .field("inlinedptrvec")
                            .unwrap()
                            .field(2)
                            .unwrap()
                            .field(0)
                            .unwrap()
                            .access::<u16>()
                            .unwrap();

                        assert_eq!(field, 5);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")
                            .unwrap()
                            .field("inlinedptrvec")
                            .unwrap()
                            .field(1)
                            .unwrap()
                            .field("mut_f32")
                            .unwrap()
                            .field("a")
                            .unwrap()
                            .access::<f32>()
                            .unwrap();

                        assert_eq!(field, 4.0);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")
                            .unwrap()
                            .field("u8array")
                            .unwrap()
                            .field([1, 1])
                            .unwrap()
                            .access::<u8>()
                            .unwrap();

                        assert_eq!(field, 4);
                    }

                    {
                        assert!(value
                            .field_accessor()
                            .field("arrays")
                            .unwrap()
                            .field("u8array")
                            .unwrap()
                            .field("wrongkind")
                            .is_err());
                    }

                    {
                        let sym = "wrongkind".to_symbol(&frame);
                        assert!(value
                            .field_accessor()
                            .field("arrays")
                            .unwrap()
                            .field("u8array")
                            .unwrap()
                            .field(sym)
                            .is_err());
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")
                            .unwrap()
                            .field("inlinedptrarray")
                            .unwrap()
                            .field([1, 0])
                            .unwrap()
                            .field(1)
                            .unwrap()
                            .field(0)
                            .unwrap()
                            .access::<f32>()
                            .unwrap();

                        assert_eq!(field, 6.0);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("nonexistent")
                            .unwrap()
                            .access::<WeakValue>();

                        assert!(field.is_err());
                    }
                })
            })
        })
    }

    #[test]
    fn access_field_tests() {
        empty_union_field();
        access_mutable_struct_fields();
        cannot_access_unknown_mutable_struct_field();
        access_tuple_fields();
        cannot_access_oob_tuple_field();
        access_non_pointer_tuple_field_must_alloc();
        access_bounds_error_fields();
        access_bounds_error_fields_oob();
        access_bounds_error_fields_output();
        access_bounds_error_fields_output_oob();
        access_nested_field();
    }
}
