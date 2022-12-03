mod util;

#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
mod tests {
    use jlrs::{
        data::managed::array::dimensions::Dims, layout::valid_layout::ValidLayout, prelude::*,
    };

    use crate::util::JULIA;

    fn array_can_be_cast() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result();
                    assert!(arr_val.is_ok());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_array_1d() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let arr_val = Value::eval_string(
                        &mut frame,
                        "a = Vector{Union{Int32, Float32, Bool}}()
                        push!(a, Int32(1))
                        push!(a, Float32(2.0))
                        push!(a, false)
                        a",
                    )
                    .unwrap();
                    let arr = arr_val.cast::<Array>();
                    assert!(arr.is_ok());
                    let arr = arr.unwrap();
                    {
                        let ud = arr.union_data().unwrap();
                        let v1 = ud.get::<i32, _>(0).unwrap();
                        assert_eq!(v1, 1);
                        let v2 = ud.get::<f32, _>(1).unwrap();
                        assert_eq!(v2, 2.0);
                        let v3 = ud.get::<bool, _>(2).unwrap();
                        assert_eq!(v3, false);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_dimensions() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let arr = arr_val;
                    let dims = unsafe { arr.dimensions() }.into_dimensions();
                    assert_eq!(dims.as_slice(), &[1, 2]);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn check_array_contents_info() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let arr = arr_val;
                    assert!(arr.contains::<f32>());
                    assert!(arr.contains_inline::<f32>());
                    assert!(arr.try_as_typed::<f32>().is_ok());
                    assert!(arr.try_as_typed::<f64>().is_err());
                    assert!(!arr.has_inlined_pointers());
                    assert!(arr.is_inline_array());
                    assert!(!arr.is_value_array());
                    assert_eq!(arr.element_type().cast::<DataType>()?.name(), "Float32");

                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_unbox_new_as_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs.instance(&mut frame).scope(|mut frame| {
                let p = Value::new(&mut frame, 1u8);
                p.cast::<Array>()?;
                Ok(())
            });

            assert!(out.is_err());
        });
    }

    fn cannot_unbox_array_with_wrong_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs.instance(&mut frame).scope(|mut frame| {
                let array = Array::new::<f32, _, _>(frame.as_extended_target(), (3, 1))
                    .into_jlrs_result()?;
                unsafe { array.copy_inline_data::<u8>() }
            });

            assert!(out.is_err());
        });
    }

    fn typed_array_can_be_cast() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let arr = arr_val.as_value().cast::<TypedArray<f32>>();
                    assert!(arr.is_ok());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn typed_array_dimensions() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let arr = arr_val.as_value().cast::<TypedArray<f32>>()?;
                    let dims = unsafe { arr.dimensions() }.into_dimensions();
                    assert_eq!(dims.as_slice(), &[1, 2]);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn check_typed_array_contents_info() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let arr = arr_val.as_value().cast::<TypedArray<f32>>()?;
                    assert!(!arr.has_inlined_pointers());
                    assert!(arr.is_inline_array());
                    assert!(!arr.is_value_array());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_copy_value_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|frame| unsafe {
                    let arr_val = Value::an_empty_vec_any(&frame);

                    assert!(arr_val
                        .cast::<Array>()?
                        .copy_inline_data::<Option<ValueRef>>()
                        .is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_access_value_as_inline() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|frame| unsafe {
                    let arr_val = Value::an_empty_vec_any(&frame);
                    assert!(arr_val
                        .cast::<Array>()?
                        .inline_data::<Option<ValueRef>>()
                        .is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_access_value_as_inline_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|frame| unsafe {
                    let arr_val = Value::an_empty_vec_any(&frame);
                    assert!(arr_val
                        .cast::<Array>()?
                        .inline_data_mut::<Option<ValueRef>>()
                        .is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_access_value_as_unrestricted_inline_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|frame| unsafe {
                    let arr_val = Value::an_empty_vec_any(&frame);
                    assert!(arr_val
                        .cast::<Array>()?
                        .inline_data_mut::<Option<ValueRef>>()
                        .is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_access_value_as_unrestricted_inline_mut_wrong_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|frame| unsafe {
                    let arr_val = Value::an_empty_vec_any(&frame);
                    assert!(arr_val.cast::<Array>()?.inline_data_mut::<f64>().is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_access_f32_as_value() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    assert!(arr_val.value_data().is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_access_f32_as_value_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let mut arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    assert!(arr_val.value_data_mut().is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_access_f32_as_unrestricted_value_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let mut arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    assert!(arr_val.value_data_mut().is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn convert_back_to_value() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let arr_val = Value::an_empty_vec_any(&frame);
                    arr_val.cast::<Array>()?.as_value().is::<Array>();
                    Ok(())
                })
                .unwrap();
        });
    }

    fn invalid_layout() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let not_arr_val = Value::new(&mut frame, 1usize);
                    assert!(!ArrayRef::valid_layout(not_arr_val));
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn arrays_tests() {
        array_can_be_cast();
        union_array_1d();
        array_dimensions();
        check_array_contents_info();
        cannot_unbox_new_as_array();
        cannot_unbox_array_with_wrong_type();
        typed_array_can_be_cast();
        typed_array_dimensions();
        check_typed_array_contents_info();
        cannot_copy_value_data();
        cannot_access_value_as_inline();
        cannot_access_value_as_inline_mut();
        cannot_access_value_as_unrestricted_inline_mut();
        cannot_access_value_as_unrestricted_inline_mut_wrong_type();
        cannot_access_f32_as_value();
        cannot_access_f32_as_value_mut();
        cannot_access_f32_as_unrestricted_value_mut();
        convert_back_to_value();
        invalid_layout();
    }
}
