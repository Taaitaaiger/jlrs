use jlrs::prelude::*;
use jlrs::util::JULIA;
use jlrs::value::datatype::Mutable;

#[test]
fn access_tuple_fields() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(4, |global, frame| {
            // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("inlinetuple")?;
            let tup = func.call0(&mut *frame)?.unwrap();

            assert!(tup.is::<Tuple>());
            assert_eq!(tup.n_fields(), 3);
            let v1 = tup.get_nth_field(&mut *frame, 0)?;
            let v2 = tup.get_nth_field(&mut *frame, 1)?;
            let v3 = frame.value_scope_with_slots(0, |output, frame| {
                let output = output.into_scope(frame);
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
        jlrs.scope_with_slots(4, |global, frame| {
            // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("inlinetuple")?;
            let tup = func.call0(&mut *frame)?.unwrap();
            assert!(tup.get_nth_field(&mut *frame, 3).is_err());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn access_non_pointer_tuple_field_must_alloc() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(4, |global, frame| {
            // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("inlinetuple")?;
            let tup = func.call0(&mut *frame)?.unwrap();
            assert!(tup.get_nth_field_noalloc(2).is_err());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn access_mutable_struct_fields() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(5, |global, frame| {
            //mutable struct MutableStruct
            //  x
            //  y::UInt64
            //end
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("MutableStruct")?;

            let x = Value::new(&mut *frame, 2.0f32)?;
            let y = Value::new(&mut *frame, 3u64)?;

            let mut_struct = func.call2(&mut *frame, x, y)?.unwrap();
            assert!(mut_struct.is::<Mutable>());

            assert!(mut_struct.get_field(&mut *frame, "x").is_ok());
            let x_val = mut_struct.get_field_noalloc("x");
            assert!(x_val.is_ok());
            unsafe {
                assert!(x_val.unwrap().assume_reachable().unwrap().is::<f32>());
            }
            let _ = frame.value_scope_with_slots(0, |output, frame| {
                let output = output.into_scope(frame);
                mut_struct.get_field(output, "y")
            })?;
            assert!(mut_struct.get_field_noalloc("y").is_err());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn cannot_access_unknown_mutable_struct_field() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(5, |global, frame| {
            //mutable struct MutableStruct
            //  x
            //  y::UInt64
            //end
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("MutableStruct")?;

            let x = Value::new(&mut *frame, 2.0f32)?;
            let y = Value::new(&mut *frame, 3u64)?;

            let mut_struct = func.call2(&mut *frame, x, y)?.unwrap();
            assert!(mut_struct.is::<Mutable>());

            assert!(mut_struct.get_field(&mut *frame, "z").is_err());
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
            .scope_with_slots(5, |global, frame| {
                let idx = Value::new(&mut *frame, 4usize)?;
                let data = vec![1.0f64, 2., 3.];
                let array = Value::move_array(&mut *frame, data, 3)?;
                let func = Module::base(global).function("getindex")?;
                let out = func.call2(&mut *frame, array, idx)?.unwrap_err();

                assert_eq!(out.type_name().unwrap(), "BoundsError");

                let field_names = out.field_names();
                let f0: String = field_names[0].as_string().unwrap();
                assert_eq!(f0, "a");
                let f1: String = field_names[1].as_string().unwrap();
                assert_eq!(f1, "i");

                out.get_field_noalloc("a")?;

                out.get_field(&mut *frame, field_names[1])?
                    .get_nth_field(&mut *frame, 0)?
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

        jlrs.scope_with_slots(5, |global, frame| {
            let idx = Value::new(&mut *frame, 4usize)?;
            let data = vec![1.0f64, 2., 3.];
            let array = Value::move_array(&mut *frame, data, 3)?;
            let func = Module::base(global).function("getindex")?;
            let out = func.call2(&mut *frame, array, idx)?.unwrap_err();

            let field_names = out.field_names();
            assert!(out
                .get_field(&mut *frame, field_names[1])?
                .get_nth_field(&mut *frame, 123)
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

        jlrs.scope_with_slots(5, |global, frame| {
            let idx = Value::new(&mut *frame, 4usize)?;
            let data = vec![1.0f64, 2., 3.];
            let array = Value::move_array(&mut *frame, data, 3)?;
            let func = Module::base(global).function("getindex")?;
            let out = func.call2(&mut *frame, array, idx)?.unwrap_err();

            let field_names = out.field_names();
            let _ = frame.value_scope_with_slots(1, |output, frame| {
                let field = out.get_field(&mut *frame, field_names[1])?;
                let output = output.into_scope(frame);
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

        jlrs.scope_with_slots(5, |global, frame| {
            let idx = Value::new(&mut *frame, 4usize)?;
            let data = vec![1.0f64, 2., 3.];
            let array = Value::move_array(&mut *frame, data, 3)?;
            let func = Module::base(global).function("getindex")?;
            let out = func.call2(&mut *frame, array, idx)?.unwrap_err();

            let field_names = out.field_names();
            let _ = frame
                .value_scope_with_slots(1, |output, frame| {
                    let field = out.get_field(&mut *frame, field_names[1])?;
                    let output = output.into_scope(frame);
                    field.get_nth_field(output, 123)
                })
                .unwrap_err();
            Ok(())
        })
        .unwrap();
    });
}
