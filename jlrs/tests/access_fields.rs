mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    #[cfg(not(feature = "lts"))]
    use std::sync::atomic::Ordering;

    use super::util::{JULIA, MIXED_BAG_JL};
    use jlrs::convert::to_symbol::ToSymbol;
    use jlrs::prelude::*;
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    use jlrs::{layout::typecheck::Mutable, wrappers::inline::union::EmptyUnion};

    #[test]
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    fn empty_union_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| unsafe {
                let mut tys = [Value::new(&mut frame, 0usize)];
                let res = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .global_ref("WithEmpty")?
                    .wrapper_unchecked()
                    .apply_type(&mut frame, &mut tys)
                    .into_jlrs_result()?
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [])?
                    .into_jlrs_result()?;

                assert!(res
                    .field_accessor(&mut frame)
                    .field(1)?
                    .access::<EmptyUnion>()
                    .is_err());
                Ok(())
            })
            .unwrap()
        })
    }

    #[test]
    fn access_tuple_fields() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| unsafe {
                // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("inlinetuple")?
                    .wrapper_unchecked();
                let tup = func.call0(&mut frame).unwrap();

                assert!(tup.is::<Tuple>());
                assert_eq!(tup.n_fields(), 3);
                let v1 = tup.get_nth_field(&mut frame, 0)?;
                let v2 = tup.get_nth_field(&mut frame, 1)?;
                let (output, frame) = frame.split();
                let v3 = frame.scope(|mut frame| {
                    let output = output.into_scope(&mut frame);
                    tup.get_nth_field(output, 2)
                })?;

                assert!(v1.is::<u32>());
                assert!(v2.is::<u16>());
                assert!(v3.is::<i64>());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn cannot_access_oob_tuple_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| unsafe {
                // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("inlinetuple")?
                    .wrapper_unchecked();
                let tup = func.call0(&mut frame).unwrap();
                assert!(tup.get_nth_field(&mut frame, 3).is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_non_pointer_tuple_field_must_alloc() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| unsafe {
                // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("inlinetuple")?
                    .wrapper_unchecked();
                let tup = func.call0(&mut frame).unwrap();
                assert!(tup.get_nth_field_ref(2).is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    fn access_mutable_struct_fields() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| unsafe {
                //mutable struct MutableStruct
                //  x
                //  y::UInt64
                //end
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .global_ref("MutableStruct")?
                    .wrapper_unchecked()
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
                    assert!(x_val.unwrap().wrapper().unwrap().is::<f32>());
                }
                let (output, frame) = frame.split();
                let _ = frame.scope(|mut frame| {
                    let output = output.into_scope(&mut frame);
                    mut_struct.get_field(output, "y")
                })?;
                assert!(mut_struct.get_field_ref("y").is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    fn cannot_access_unknown_mutable_struct_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| unsafe {
                //mutable struct MutableStruct
                //  x
                //  y::UInt64
                //end
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .global_ref("MutableStruct")?
                    .wrapper_unchecked()
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

    #[test]
    fn access_bounds_error_fields() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let oob_idx = jlrs
                .scope(|global, mut frame| unsafe {
                    let idx = Value::new(&mut frame, 4usize);
                    let data = vec![1.0f64, 2., 3.];
                    let array = Array::from_vec_unchecked(&mut frame, data, 3)?;
                    let func = Module::base(global)
                        .function_ref("getindex")?
                        .wrapper_unchecked();
                    let out = func.call2(&mut frame, array.as_value(), idx).unwrap_err();

                    assert_eq!(out.datatype_name().unwrap(), "BoundsError");

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

    #[test]
    fn access_bounds_error_fields_oob() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let idx = Value::new(&mut frame, 4usize);
                let data = vec![1.0f64, 2., 3.];
                let array = Array::from_vec_unchecked(&mut frame, data, 3)?;
                let func = Module::base(global)
                    .function_ref("getindex")?
                    .wrapper_unchecked();
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

    #[test]
    fn access_bounds_error_fields_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let idx = Value::new(&mut frame, 4usize);
                let data = vec![1.0f64, 2., 3.];
                let array = Array::from_vec_unchecked(&mut frame, data, 3)?;
                let func = Module::base(global)
                    .function_ref("getindex")?
                    .wrapper_unchecked();
                let out = func.call2(&mut frame, array.as_value(), idx).unwrap_err();

                let field_names = out.field_names();
                let (output, frame) = frame.split();
                let _ = frame.scope(|mut frame| {
                    let field = out.get_field(&mut frame, field_names[1])?;
                    let output = output.into_scope(&mut frame);
                    field.get_nth_field(output, 0)
                })?;

                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn access_bounds_error_fields_output_oob() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, mut frame| unsafe {
                let idx = Value::new(&mut frame, 4usize);
                let data = vec![1.0f64, 2., 3.];
                let array = Array::from_vec_unchecked(&mut frame, data, 3)?;
                let func = Module::base(global)
                    .function_ref("getindex")?
                    .wrapper_unchecked();
                let out = func.call2(&mut frame, array.as_value(), idx).unwrap_err();

                let field_names = out.field_names();
                let (output, frame) = frame.split();
                let _ = frame
                    .scope(|mut frame| {
                        let field = out.get_field(&mut frame, field_names[1])?;
                        let output = output.into_scope(&mut frame);
                        field.get_nth_field(output, 123)
                    })
                    .unwrap_err();
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn access_nested_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| unsafe {
                let value = Value::eval_string(&mut frame, MIXED_BAG_JL)
                    .into_jlrs_result()?
                    .cast::<Module>()?
                    .global_ref("mixedbag")?
                    .wrapper_unchecked();

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("mutabl")?
                        .field("mutable_unions")?
                        .field("bits_union")?
                        .access::<i32>()?;

                    assert_eq!(field, 3);
                }

                #[cfg(not(feature = "lts"))]
                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("mutabl")?
                        .field("mutable_unions")?
                        .atomic_field("atomic_union", Ordering::Relaxed)?
                        .access::<i64>()?;

                    assert_eq!(field, 5);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("mutabl")?
                        .field("mutable_unions")?
                        .field("normal_union")?
                        .access::<Nothing>()?;

                    assert_eq!(field, Nothing);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("mutabl")?
                        .field("immutable_unions")?
                        .field("bits_union")?
                        .access::<i64>()?;

                    assert_eq!(field, 7);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("mutabl")?
                        .field("immutable_unions")?
                        .field("normal_union")?
                        .access::<ModuleRef>()?;

                    assert_eq!(field.wrapper_unchecked(), Module::main(global));
                }

                #[cfg(not(feature = "lts"))]
                {
                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("mutabl")?
                            .field("atomics")?
                            .field("i8")?
                            .access::<i8>()?;

                        assert_eq!(field, 1);
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("mutabl")?
                            .field("atomics")?
                            .atomic_field("i16", Ordering::Acquire)?
                            .access::<i16>()?;

                        assert_eq!(field, 2);
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("mutabl")?
                            .field("atomics")?
                            .field("i24")?
                            .field(0)?
                            .access::<i8>()?;

                        assert_eq!(field, 3);
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("mutabl")?
                            .field("atomics")?
                            .field("i48")?
                            .field(2)?
                            .access::<i8>()?;

                        assert_eq!(field, 8);
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("mutabl")?
                            .field("atomics")?
                            .field("i72")?
                            .field(1)?
                            .access::<i8>()?;

                        assert_eq!(field, 13);
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("mutabl")?
                            .field("atomics")?
                            .field("ptr")?
                            .access::<ModuleRef>()?;

                        assert_eq!(field.wrapper_unchecked(), Module::main(global));
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("mutabl")?
                            .field("atomics")?
                            .field("wrapped_ptr")?
                            .field((0,))?
                            .access::<ModuleRef>()?;

                        assert_eq!(field.wrapper_unchecked(), Module::base(global));
                    }
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("mutabl")?
                        .field("number")?
                        .access::<f64>()?;

                    assert_eq!(field, 3.0);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("immutabl")?
                        .field("mutable_unions")?
                        .field("bits_union")?
                        .access::<i32>()?;

                    assert_eq!(field, -3);
                }

                #[cfg(not(feature = "lts"))]
                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("immutabl")?
                        .field("mutable_unions")?
                        .atomic_field("atomic_union", Ordering::Relaxed)?
                        .access::<i64>()?;

                    assert_eq!(field, -5);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("immutabl")?
                        .field("mutable_unions")?
                        .field("normal_union")?
                        .access::<ModuleRef>()?;

                    assert_eq!(field.wrapper_unchecked(), Module::main(global));
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("immutabl")?
                        .field("immutable_unions")?
                        .field("bits_union")?
                        .access::<i64>()?;

                    assert_eq!(field, -7);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("immutabl")?
                        .field("immutable_unions")?
                        .field("normal_union")?
                        .access::<Nothing>()?;

                    assert_eq!(field, Nothing);
                }

                #[cfg(not(feature = "lts"))]
                {
                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("immutabl")?
                            .field("atomics")?
                            .field("i8")?
                            .access::<i8>()?;

                        assert_eq!(field, -1);
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("immutabl")?
                            .field("atomics")?
                            .atomic_field("i16", Ordering::Acquire)?
                            .access::<i16>()?;

                        assert_eq!(field, -2);
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("immutabl")?
                            .field("atomics")?
                            .field("i24")?
                            .field(0)?
                            .access::<i8>()?;

                        assert_eq!(field, -3);
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("immutabl")?
                            .field("atomics")?
                            .field("i48")?
                            .field(2)?
                            .access::<i8>()?;

                        assert_eq!(field, -8);
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("immutabl")?
                            .field("atomics")?
                            .field("i72")?
                            .field(1)?
                            .access::<i8>()?;

                        assert_eq!(field, -13);
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("immutabl")?
                            .field("atomics")?
                            .field("ptr")?
                            .access::<ModuleRef>()?;

                        assert_eq!(field.wrapper_unchecked(), Module::main(global));
                    }

                    {
                        let field = value
                            .field_accessor(&mut frame)
                            .field("immutabl")?
                            .field("atomics")?
                            .field("wrapped_ptr")?
                            .field((0,))?
                            .access::<ModuleRef>()?;

                        assert_eq!(field.wrapper_unchecked(), Module::base(global));
                    }
                }
                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("immutabl")?
                        .field("number")?
                        .access::<i16>()?;

                    assert_eq!(field, -3);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("tuples")?
                        .field("empty")?
                        .access::<Tuple0>()?;

                    assert_eq!(field, Tuple0());
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("tuples")?
                        .field("single")?
                        .field(0)?
                        .access::<i32>()?;

                    assert_eq!(field, 1);
                }

                {
                    let s = JuliaString::new(&mut frame, "double")?;
                    let field = value
                        .field_accessor(&mut frame)
                        .field("tuples")?
                        .field(s)?
                        .field(1)?
                        .access::<i64>()?;

                    assert_eq!(field, -4);
                }

                {
                    let s = "double".to_symbol(global);
                    let field = value
                        .field_accessor(&mut frame)
                        .field("tuples")?
                        .field(s)?
                        .field(1)?
                        .access::<i64>()?;

                    assert_eq!(field, -4);
                }

                {
                    assert!(value
                        .field_accessor(&mut frame)
                        .field("tuples")?
                        .field("double")?
                        .field((1, 1))
                        .is_err());
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("tuples")?
                        .field("abstract")?
                        .field(1)?
                        .access::<f64>()?;

                    assert_eq!(field, 4.0);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("arrays")?
                        .field("u8vec")?
                        .field(1)?
                        .access::<u8>()?;

                    assert_eq!(field, 2);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("arrays")?
                        .field("unionvec")?
                        .field(0)?
                        .access::<u8>()?;

                    assert_eq!(field, 1);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("arrays")?
                        .field("unionvec")?
                        .field(1)?
                        .access::<u16>()?;

                    assert_eq!(field, 2);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("arrays")?
                        .field("wrappervec")?
                        .field(1)?
                        .access::<ModuleRef>()?;

                    assert_eq!(field.wrapper_unchecked(), Module::base(global));
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("arrays")?
                        .field("ptrvec")?
                        .field(1)?
                        .field(0)?
                        .access::<f32>()?;

                    assert_eq!(field, 2.0);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
                        .field("arrays")?
                        .field("inlinedptrvec")?
                        .field(2)?
                        .field(0)?
                        .access::<u16>()?;

                    assert_eq!(field, 5);
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
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
                        .field_accessor(&mut frame)
                        .field("arrays")?
                        .field("u8array")?
                        .field((1, 1))?
                        .access::<u8>()?;

                    assert_eq!(field, 4);
                }

                {
                    assert!(value
                        .field_accessor(&mut frame)
                        .field("arrays")?
                        .field("u8array")?
                        .field("wrongkind")
                        .is_err());
                }

                {
                    let sym = "wrongkind".to_symbol(global);
                    assert!(value
                        .field_accessor(&mut frame)
                        .field("arrays")?
                        .field("u8array")?
                        .field(sym)
                        .is_err());
                }

                {
                    let field = value
                        .field_accessor(&mut frame)
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
                        .field_accessor(&mut frame)
                        .field("nonexistent")?
                        .access::<ValueRef>()?;

                    assert!(field.is_undefined());
                }

                Ok(())
            })
            .unwrap();
        })
    }
}
