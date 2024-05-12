mod util;
#[cfg(feature = "local-rt")]
mod tests {
    #[cfg(not(feature = "julia-1-6"))]
    use std::sync::atomic::Ordering;

    use jlrs::{
        convert::to_symbol::ToSymbol,
        data::{layout::union::EmptyUnion, types::typecheck::Mutable},
        prelude::*,
    };

    use super::util::{JULIA, MIXED_BAG_JL};

    fn empty_union_field() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let mut tys = [Value::new(&mut frame, 0usize)];
                    let res = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .global(&frame, "WithEmpty")?
                        .as_managed()
                        .apply_type(&mut frame, &mut tys)
                        .into_jlrs_result()?
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [])?
                        .into_jlrs_result()?;

                    assert!(res
                        .field_accessor()
                        .field(1)?
                        .access::<EmptyUnion>()
                        .is_err());
                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_tuple_fields() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "inlinetuple")?
                        .as_managed();
                    let tup = func.call0(&mut frame).unwrap();

                    assert!(tup.is::<Tuple>());
                    assert_eq!(tup.n_fields(), 3);
                    let v1 = tup.get_nth_field(&mut frame, 0)?;
                    let v2 = tup.get_nth_field(&mut frame, 1)?;
                    let output = frame.output();
                    let v3 = frame.scope(|_| tup.get_nth_field(output, 2))?;

                    assert!(v1.is::<u32>());
                    assert!(v2.is::<u16>());
                    assert!(v3.is::<i64>());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn cannot_access_oob_tuple_field() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "inlinetuple")?
                        .as_managed();
                    let tup = func.call0(&mut frame).unwrap();
                    assert!(tup.get_nth_field(&mut frame, 3).is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_non_pointer_tuple_field_must_alloc() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "inlinetuple")?
                        .as_managed();
                    let tup = func.call0(&mut frame).unwrap();
                    assert!(tup.get_nth_field_ref(2).is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_mutable_struct_fields() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    //mutable struct MutableStruct
                    //  x
                    //  y::UInt64
                    //end
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .global(&frame, "MutableStruct")?
                        .as_managed()
                        .cast::<DataType>()?;

                    let x = Value::new(&mut frame, 2.0f32);
                    let y = Value::new(&mut frame, 3u64);

                    let mut_struct = func
                        .instantiate(&mut frame, &mut [x, y])?
                        .into_jlrs_result()?;
                    assert!(mut_struct.is::<Mutable>());

                    assert!(mut_struct.get_field(&mut frame, "x").is_ok());
                    let x_val = mut_struct.get_field_ref("x");
                    assert!(x_val.is_ok());
                    {
                        assert!(x_val.unwrap().unwrap().as_managed().is::<f32>());
                    }
                    let output = frame.output();
                    let _ = frame.scope(|_| mut_struct.get_field(output, "y"))?;
                    assert!(mut_struct.get_field_ref("y").is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn cannot_access_unknown_mutable_struct_field() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    //mutable struct MutableStruct
                    //  x
                    //  y::UInt64
                    //end
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .global(&frame, "MutableStruct")?
                        .as_managed()
                        .cast::<DataType>()?;

                    let x = Value::new(&mut frame, 2.0f32);
                    let y = Value::new(&mut frame, 3u64);

                    let mut_struct = func
                        .instantiate(&mut frame, &mut [x, y])?
                        .into_jlrs_result()?;
                    assert!(mut_struct.is::<Mutable>());

                    assert!(mut_struct.get_field(&mut frame, "z").is_err());
                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_bounds_error_fields() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let oob_idx = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let idx = Value::new(&mut frame, 4usize);
                    let data = vec![1.0f64, 2., 3.];
                    let array = TypedArray::<f64>::from_vec_unchecked(&mut frame, data, 3);
                    let func = Module::base(&frame)
                        .function(&frame, "getindex")?
                        .as_managed();
                    let out = func.call2(&mut frame, array.as_value(), idx).unwrap_err();

                    assert_eq!(out.datatype_name(), "BoundsError");

                    let field_names = out.field_names();
                    let f0: String = field_names[0].as_string().unwrap();
                    assert_eq!(f0, "a");
                    let f1: String = field_names[1].as_string().unwrap();
                    assert_eq!(f1, "i");

                    out.get_field_ref("a")?;

                    out.get_field(&mut frame, field_names[1])?
                        .get_nth_field(&mut frame, 0)?
                        .unbox::<isize>()
                })
                .unwrap();

            assert_eq!(oob_idx, 4);
        });
    }

    fn access_bounds_error_fields_oob() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let idx = Value::new(&mut frame, 4usize);
                    let data = vec![1.0f64, 2., 3.];
                    let array =
                        TypedArray::<f64>::from_vec(&mut frame, data, 3)?.into_jlrs_result()?;

                    let func = Module::base(&frame)
                        .function(&frame, "getindex")?
                        .as_managed();
                    let out = func.call2(&mut frame, array.as_value(), idx).unwrap_err();

                    let field_names = out.field_names();
                    assert!(out
                        .get_field(&mut frame, field_names[1])?
                        .get_nth_field(&mut frame, 123)
                        .is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn access_bounds_error_fields_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let idx = Value::new(&mut frame, 4usize);
                    let data = vec![1.0f64, 2., 3.];
                    let ty = DataType::float64_type(&frame);
                    let array = Array::from_vec_for_unchecked(&mut frame, ty.as_value(), data, 3);
                    let func = Module::base(&frame)
                        .function(&frame, "getindex")?
                        .as_managed();
                    let out = func.call2(&mut frame, array.as_value(), idx).unwrap_err();

                    let field_names = out.field_names();
                    let output = frame.output();
                    let _ = frame.scope(|mut frame| {
                        let field = out.get_field(&mut frame, field_names[1])?;
                        field.get_nth_field(output, 0)
                    })?;

                    Ok(())
                })
                .unwrap();
        });
    }

    fn access_bounds_error_fields_output_oob() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let idx = Value::new(&mut frame, 4usize);
                    let data = vec![1.0f64, 2., 3.];
                    let array = TypedArray::<f64>::from_vec_unchecked(&mut frame, data, 3);
                    let func = Module::base(&frame)
                        .function(&frame, "getindex")?
                        .as_managed();
                    let out = func.call2(&mut frame, array.as_value(), idx).unwrap_err();

                    let field_names = out.field_names();
                    let output = frame.output();
                    let _ = frame
                        .scope(|mut frame| {
                            let field = out.get_field(&mut frame, field_names[1])?;
                            field.get_nth_field(output, 123)
                        })
                        .unwrap_err();
                    Ok(())
                })
                .unwrap();
        });
    }

    fn access_nested_field() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let value = Value::eval_string(&mut frame, MIXED_BAG_JL)
                        .into_jlrs_result()?
                        .cast::<Module>()?
                        .global(&frame, "mixedbag")?
                        .as_managed();

                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")?
                            .field("mutable_unions")?
                            .field("bits_union")?
                            .access::<i32>()?;

                        assert_eq!(field, 3);
                    }

                    #[cfg(not(feature = "julia-1-6"))]
                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")?
                            .field("mutable_unions")?
                            .atomic_field("atomic_union", Ordering::Relaxed)?
                            .access::<i64>()?;

                        assert_eq!(field, 5);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")?
                            .field("mutable_unions")?
                            .field("normal_union")?
                            .access::<Nothing>()?;

                        assert_eq!(field, Nothing);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")?
                            .field("immutable_unions")?
                            .field("bits_union")?
                            .access::<i64>()?;

                        assert_eq!(field, 7);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")?
                            .field("immutable_unions")?
                            .field("normal_union")?
                            .access::<ModuleRef>()?;

                        assert_eq!(field.as_managed(), Module::main(&frame));
                    }

                    #[cfg(not(feature = "julia-1-6"))]
                    {
                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")?
                                .field("atomics")?
                                .field("i8")?
                                .access::<i8>()?;

                            assert_eq!(field, 1);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")?
                                .field("atomics")?
                                .atomic_field("i16", Ordering::Acquire)?
                                .access::<i16>()?;

                            assert_eq!(field, 2);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")?
                                .field("atomics")?
                                .field("i24")?
                                .field(0)?
                                .access::<i8>()?;

                            assert_eq!(field, 3);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")?
                                .field("atomics")?
                                .field("i48")?
                                .field(2)?
                                .access::<i8>()?;

                            assert_eq!(field, 8);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")?
                                .field("atomics")?
                                .field("i72")?
                                .field(1)?
                                .access::<i8>()?;

                            assert_eq!(field, 13);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")?
                                .field("atomics")?
                                .field("ptr")?
                                .access::<ModuleRef>()?;

                            assert_eq!(field.as_managed(), Module::main(&frame));
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("mutabl")?
                                .field("atomics")?
                                .field("wrapped_ptr")?
                                .field((0,))?
                                .access::<ModuleRef>()?;

                            assert_eq!(field.as_managed(), Module::base(&frame));
                        }
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("mutabl")?
                            .field("number")?
                            .access::<f64>()?;

                        assert_eq!(field, 3.0);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")?
                            .field("mutable_unions")?
                            .field("bits_union")?
                            .access::<i32>()?;

                        assert_eq!(field, -3);
                    }

                    #[cfg(not(feature = "julia-1-6"))]
                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")?
                            .field("mutable_unions")?
                            .atomic_field("atomic_union", Ordering::Relaxed)?
                            .access::<i64>()?;

                        assert_eq!(field, -5);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")?
                            .field("mutable_unions")?
                            .field("normal_union")?
                            .access::<ModuleRef>()?;

                        assert_eq!(field.as_managed(), Module::main(&frame));
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")?
                            .field("immutable_unions")?
                            .field("bits_union")?
                            .access::<i64>()?;

                        assert_eq!(field, -7);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")?
                            .field("immutable_unions")?
                            .field("normal_union")?
                            .access::<Nothing>()?;

                        assert_eq!(field, Nothing);
                    }

                    #[cfg(not(feature = "julia-1-6"))]
                    {
                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")?
                                .field("atomics")?
                                .field("i8")?
                                .access::<i8>()?;

                            assert_eq!(field, -1);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")?
                                .field("atomics")?
                                .atomic_field("i16", Ordering::Acquire)?
                                .access::<i16>()?;

                            assert_eq!(field, -2);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")?
                                .field("atomics")?
                                .field("i24")?
                                .field(0)?
                                .access::<i8>()?;

                            assert_eq!(field, -3);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")?
                                .field("atomics")?
                                .field("i48")?
                                .field(2)?
                                .access::<i8>()?;

                            assert_eq!(field, -8);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")?
                                .field("atomics")?
                                .field("i72")?
                                .field(1)?
                                .access::<i8>()?;

                            assert_eq!(field, -13);
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")?
                                .field("atomics")?
                                .field("ptr")?
                                .access::<ModuleRef>()?;

                            assert_eq!(field.as_managed(), Module::main(&frame));
                        }

                        {
                            let field = value
                                .field_accessor()
                                .field("immutabl")?
                                .field("atomics")?
                                .field("wrapped_ptr")?
                                .field((0,))?
                                .access::<ModuleRef>()?;

                            assert_eq!(field.as_managed(), Module::base(&frame));
                        }
                    }
                    {
                        let field = value
                            .field_accessor()
                            .field("immutabl")?
                            .field("number")?
                            .access::<i16>()?;

                        assert_eq!(field, -3);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("tuples")?
                            .field("empty")?
                            .access::<Tuple0>()?;

                        assert_eq!(field, Tuple0());
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("tuples")?
                            .field("single")?
                            .field(0)?
                            .access::<i32>()?;

                        assert_eq!(field, 1);
                    }

                    {
                        let s = JuliaString::new(&mut frame, "double");
                        let field = value
                            .field_accessor()
                            .field("tuples")?
                            .field(s)?
                            .field(1)?
                            .access::<i64>()?;

                        assert_eq!(field, -4);
                    }

                    {
                        let s = "double".to_symbol(&frame);
                        let field = value
                            .field_accessor()
                            .field("tuples")?
                            .field(s)?
                            .field(1)?
                            .access::<i64>()?;

                        assert_eq!(field, -4);
                    }

                    {
                        assert!(value
                            .field_accessor()
                            .field("tuples")?
                            .field("double")?
                            .field((1, 1))
                            .is_err());
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("tuples")?
                            .field("abstract")?
                            .field(1)?
                            .access::<f64>()?;

                        assert_eq!(field, 4.0);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")?
                            .field("u8vec")?
                            .field(1)?
                            .access::<u8>()?;

                        assert_eq!(field, 2);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")?
                            .field("unionvec")?
                            .field(0)?
                            .access::<u8>()?;

                        assert_eq!(field, 1);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")?
                            .field("unionvec")?
                            .field(1)?
                            .access::<u16>()?;

                        assert_eq!(field, 2);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")?
                            .field("wrappervec")?
                            .field(1)?
                            .access::<ModuleRef>()?;

                        assert_eq!(field.as_managed(), Module::base(&frame));
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")?
                            .field("ptrvec")?
                            .field(1)?
                            .field(0)?
                            .access::<f32>()?;

                        assert_eq!(field, 2.0);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")?
                            .field("inlinedptrvec")?
                            .field(2)?
                            .field(0)?
                            .access::<u16>()?;

                        assert_eq!(field, 5);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")?
                            .field("inlinedptrvec")?
                            .field(1)?
                            .field("mut_f32")?
                            .field("a")?
                            .access::<f32>()?;

                        assert_eq!(field, 4.0);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")?
                            .field("u8array")?
                            .field((1, 1))?
                            .access::<u8>()?;

                        assert_eq!(field, 4);
                    }

                    {
                        assert!(value
                            .field_accessor()
                            .field("arrays")?
                            .field("u8array")?
                            .field("wrongkind")
                            .is_err());
                    }

                    {
                        let sym = "wrongkind".to_symbol(&frame);
                        assert!(value
                            .field_accessor()
                            .field("arrays")?
                            .field("u8array")?
                            .field(sym)
                            .is_err());
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("arrays")?
                            .field("inlinedptrarray")?
                            .field((1, 0))?
                            .field(1)?
                            .field(0)?
                            .access::<f32>()?;

                        assert_eq!(field, 6.0);
                    }

                    {
                        let field = value
                            .field_accessor()
                            .field("nonexistent")?
                            .access::<ValueRef>();

                        assert!(field.is_err());
                    }

                    Ok(())
                })
                .unwrap();
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
