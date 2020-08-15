use jlrs::prelude::*;
use jlrs::util::JULIA;
use jlrs::value::datatype::Mutable;

#[test]
fn access_tuple_fields() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(4, |global, frame| {
            // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("inlinetuple")?;
            let tup = func.call0(frame)?.unwrap();

            let output = frame.output()?;

            assert!(tup.is::<Tuple>());
            assert_eq!(tup.n_fields(), 3);
            let v1 = tup.get_nth_field(frame, 0)?;
            let v2 = tup.get_nth_field(frame, 1)?;
            let v3 = tup.get_nth_field_output(frame, output, 2)?;

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
        jlrs.frame(4, |global, frame| {
            // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("inlinetuple")?;
            let tup = func.call0(frame)?.unwrap();
            assert!(tup.get_nth_field(frame, 3).is_err());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn access_non_pointer_tuple_field_must_alloc() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(4, |global, frame| {
            // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("inlinetuple")?;
            let tup = func.call0(frame)?.unwrap();
            assert!(unsafe { tup.get_nth_field_noalloc(2).is_err() });

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn access_mutable_struct_fields() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(5, |global, frame| {
            //mutable struct MutableStruct
            //  x
            //  y::UInt64
            //end
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("MutableStruct")?;

            let x = Value::new(frame, 2.0f32)?;
            let y = Value::new(frame, 3u64)?;

            let mut_struct = func.call2(frame, x, y)?.unwrap();
            assert!(mut_struct.is::<Mutable>());

            assert!(mut_struct.get_field(frame, "x").is_ok());
            let x_val = unsafe { mut_struct.get_field_noalloc("x") };
            assert!(x_val.is_ok());
            assert!(x_val.unwrap().is::<f32>());
            let output = frame.output()?;
            assert!(mut_struct.get_field_output(frame, output, "y").is_ok());
            assert!(unsafe { mut_struct.get_field_noalloc("y").is_err() });

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn cannot_access_unknown_mutable_struct_field() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(5, |global, frame| {
            //mutable struct MutableStruct
            //  x
            //  y::UInt64
            //end
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("MutableStruct")?;

            let x = Value::new(frame, 2.0f32)?;
            let y = Value::new(frame, 3u64)?;

            let mut_struct = func.call2(frame, x, y)?.unwrap();
            assert!(mut_struct.is::<Mutable>());

            assert!(mut_struct.get_field(frame, "z").is_err());
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
            .frame(5, |global, frame| {
                let idx = Value::new(frame, 4usize)?;
                let data = vec![1.0f64, 2., 3.];
                let array = Value::move_array(frame, data, 3)?;
                let func = Module::base(global).function("getindex")?;
                let out = func.call2(frame, array, idx)?.unwrap_err();

                assert_eq!(out.type_name(), "BoundsError");

                let field_names = out.field_names(global);
                let f0: String = field_names[0].into();
                assert_eq!(f0, "a");
                let f1: String = field_names[1].into();
                assert_eq!(f1, "i");

                unsafe {
                    out.get_field_noalloc("a")?;
                }

                out.get_field(frame, field_names[1])?
                    .get_nth_field(frame, 0)?
                    .cast::<isize>()
            })
            .unwrap();

        assert_eq!(oob_idx, 4);
    });
}

#[test]
fn access_bounds_error_fields_oob() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(5, |global, frame| {
            let idx = Value::new(frame, 4usize)?;
            let data = vec![1.0f64, 2., 3.];
            let array = Value::move_array(frame, data, 3)?;
            let func = Module::base(global).function("getindex")?;
            let out = func.call2(frame, array, idx)?.unwrap_err();

            let field_names = out.field_names(global);
            assert!(out
                .get_field(frame, field_names[1])?
                .get_nth_field(frame, 123)
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

        jlrs.frame(5, |global, frame| {
            let idx = Value::new(frame, 4usize)?;
            let data = vec![1.0f64, 2., 3.];
            let array = Value::move_array(frame, data, 3)?;
            let func = Module::base(global).function("getindex")?;
            let out = func.call2(frame, array, idx)?.unwrap_err();
            let output = frame.output()?;

            let field_names = out.field_names(global);
            assert!(out
                .get_field(frame, field_names[1])?
                .get_nth_field_output(frame, output, 0)
                .is_ok());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn access_bounds_error_fields_output_oob() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(5, |global, frame| {
            let idx = Value::new(frame, 4usize)?;
            let data = vec![1.0f64, 2., 3.];
            let array = Value::move_array(frame, data, 3)?;
            let func = Module::base(global).function("getindex")?;
            let out = func.call2(frame, array, idx)?.unwrap_err();
            let output = frame.output()?;

            let field_names = out.field_names(global);
            assert!(out
                .get_field(frame, field_names[1])?
                .get_nth_field_output(frame, output, 123)
                .is_err());
            Ok(())
        })
        .unwrap();
    });
}
