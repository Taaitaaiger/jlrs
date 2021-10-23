use jlrs::prelude::*;
mod util;
use jlrs::{layout::typecheck::Mutable, wrappers::inline::union::EmptyUnion};
use util::JULIA;

#[test]
fn empty_union_field() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope(|global, frame| unsafe {
            let mut tys = [Value::new(&mut *frame, 0usize)?];
            let res = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .global_ref("WithEmpty")?
                .wrapper_unchecked()
                .apply_type(&mut *frame, &mut tys)?
                .into_jlrs_result()?
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [])?
                .into_jlrs_result()?;

            assert!(res.get_nth_raw_field::<EmptyUnion>(1).is_ok());
            Ok(())
        })
        .unwrap()
    })
}

#[test]
fn access_tuple_fields() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(4, |global, frame| unsafe {
            // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("inlinetuple")?
                .wrapper_unchecked();
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
        jlrs.scope_with_slots(4, |global, frame| unsafe {
            // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("inlinetuple")?
                .wrapper_unchecked();
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
        jlrs.scope_with_slots(4, |global, frame| unsafe {
            // Returns (1, 2, 3) as Tuple{UInt32, UInt16, Int64}
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("inlinetuple")?
                .wrapper_unchecked();
            let tup = func.call0(&mut *frame)?.unwrap();
            assert!(tup.get_nth_field_ref(2).is_err());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn access_mutable_struct_fields() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(5, |global, frame| unsafe {
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

            let x = Value::new(&mut *frame, 2.0f32)?;
            let y = Value::new(&mut *frame, 3u64)?;

            let mut_struct = func
                .instantiate(&mut *frame, &mut [x, y])?
                .into_jlrs_result()?;
            assert!(mut_struct.is::<Mutable>());

            assert!(mut_struct.get_field(&mut *frame, "x").is_ok());
            let x_val = mut_struct.get_field_ref("x");
            assert!(x_val.is_ok());
            {
                assert!(x_val.unwrap().wrapper().unwrap().is::<f32>());
            }
            let _ = frame.value_scope_with_slots(0, |output, frame| {
                let output = output.into_scope(frame);
                mut_struct.get_field(output, "y")
            })?;
            assert!(mut_struct.get_field_ref("y").is_err());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn cannot_access_unknown_mutable_struct_field() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(5, |global, frame| unsafe {
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

            let x = Value::new(&mut *frame, 2.0f32)?;
            let y = Value::new(&mut *frame, 3u64)?;

            let mut_struct = func
                .instantiate(&mut *frame, &mut [x, y])?
                .into_jlrs_result()?;
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
            .scope_with_slots(5, |global, frame| unsafe {
                let idx = Value::new(&mut *frame, 4usize)?;
                let data = vec![1.0f64, 2., 3.];
                let array = Array::from_vec(&mut *frame, data, 3)?;
                let func = Module::base(global)
                    .function_ref("getindex")?
                    .wrapper_unchecked();
                let out = func.call2(&mut *frame, array, idx)?.unwrap_err();

                assert_eq!(out.datatype_name().unwrap(), "BoundsError");

                let field_names = out.field_names();
                let f0: String = field_names[0].as_string().unwrap();
                assert_eq!(f0, "a");
                let f1: String = field_names[1].as_string().unwrap();
                assert_eq!(f1, "i");

                out.get_field_ref("a")?;

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

        jlrs.scope_with_slots(5, |global, frame| unsafe {
            let idx = Value::new(&mut *frame, 4usize)?;
            let data = vec![1.0f64, 2., 3.];
            let array = Array::from_vec(&mut *frame, data, 3)?;
            let func = Module::base(global)
                .function_ref("getindex")?
                .wrapper_unchecked();
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

        jlrs.scope_with_slots(5, |global, frame| unsafe {
            let idx = Value::new(&mut *frame, 4usize)?;
            let data = vec![1.0f64, 2., 3.];
            let array = Array::from_vec(&mut *frame, data, 3)?;
            let func = Module::base(global)
                .function_ref("getindex")?
                .wrapper_unchecked();
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

        jlrs.scope_with_slots(5, |global, frame| unsafe {
            let idx = Value::new(&mut *frame, 4usize)?;
            let data = vec![1.0f64, 2., 3.];
            let array = Array::from_vec(&mut *frame, data, 3)?;
            let func = Module::base(global)
                .function_ref("getindex")?
                .wrapper_unchecked();
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
