#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use crate::util::JULIA;
    use jlrs::prelude::*;
    use jlrs::wrappers::ptr::array::dimensions::Dims;
    use jlrs::{
        layout::valid_layout::ValidLayout,
        wrappers::ptr::{ArrayRef, ValueRef},
    };

    #[test]
    fn array_can_be_cast() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let arr_val = Array::new::<f32, _, _, _>(&mut frame, (1, 2))?.into_jlrs_result();
                assert!(arr_val.is_ok());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn union_array_1d() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_global, mut frame| unsafe {
                let arr_val = Value::eval_string(
                    &mut frame,
                    "a = Vector{Union{Int32, Float32, Bool}}()
                push!(a, Int32(1))
                push!(a, Float32(2.0))
                push!(a, false)
                a",
                )?
                .unwrap();
                let arr = arr_val.cast::<Array>();
                assert!(arr.is_ok());
                let arr = arr.unwrap();
                {
                    let ud = arr.union_data(&frame).unwrap();
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

    #[test]
    fn array_dimensions() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let arr_val = Array::new::<f32, _, _, _>(&mut frame, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;
                let dims = arr.dimensions().into_dimensions();
                assert_eq!(dims.as_slice(), &[1, 2]);
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn check_array_contents_info() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let arr_val = Array::new::<f32, _, _, _>(&mut frame, (1, 2))?.into_jlrs_result()?;
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

    #[test]
    fn cannot_unbox_new_as_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(1, |_, mut frame| {
                let p = Value::new(&mut frame, 1u8)?;
                p.cast::<Array>()?;
                Ok(())
            });

            assert!(out.is_err());
        });
    }

    #[test]
    fn cannot_unbox_array_with_wrong_type() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(1, |_, mut frame| {
                let array = Array::new::<f32, _, _, _>(&mut frame, (3, 1))?.into_jlrs_result()?;
                array.copy_inline_data::<u8, _>(&frame)
            });

            assert!(out.is_err());
        });
    }

    #[test]
    fn typed_array_can_be_cast() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let arr_val = Array::new::<f32, _, _, _>(&mut frame, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val.as_value().cast::<TypedArray<f32>>();
                assert!(arr.is_ok());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn typed_array_dimensions() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let arr_val = Array::new::<f32, _, _, _>(&mut frame, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val.as_value().cast::<TypedArray<f32>>()?;
                let dims = arr.dimensions().into_dimensions();
                assert_eq!(dims.as_slice(), &[1, 2]);
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn check_typed_array_contents_info() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let arr_val = Array::new::<f32, _, _, _>(&mut frame, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val.as_value().cast::<TypedArray<f32>>()?;
                assert!(!arr.has_inlined_pointers());
                assert!(arr.is_inline_array());
                assert!(!arr.is_value_array());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn cannot_copy_value_data() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(0, |global, frame| {
                let arr_val = Value::an_empty_vec_any(global);

                assert!(arr_val
                    .cast::<Array>()?
                    .copy_inline_data::<ValueRef, _>(&frame)
                    .is_err());

                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn cannot_access_value_as_inline() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(0, |global, mut frame| {
                let arr_val = Value::an_empty_vec_any(global);
                assert!(arr_val
                    .cast::<Array>()?
                    .inline_data::<ValueRef, _>(&mut frame)
                    .is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn cannot_access_value_as_inline_mut() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(0, |global, mut frame| unsafe {
                let arr_val = Value::an_empty_vec_any(global);
                assert!(arr_val
                    .cast::<Array>()?
                    .inline_data_mut::<ValueRef, _>(&mut frame)
                    .is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn cannot_access_value_as_unrestricted_inline_mut() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(0, |global, mut frame| unsafe {
                let arr_val = Value::an_empty_vec_any(global);
                assert!(arr_val
                    .cast::<Array>()?
                    .unrestricted_inline_data_mut::<ValueRef, _>(&mut frame)
                    .is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn cannot_access_value_as_unrestricted_inline_mut_wrong_type() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(0, |global, mut frame| unsafe {
                let arr_val = Value::an_empty_vec_any(global);
                assert!(arr_val
                    .cast::<Array>()?
                    .unrestricted_inline_data_mut::<f64, _>(&mut frame)
                    .is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn cannot_access_f32_as_value() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let arr_val = Array::new::<f32, _, _, _>(&mut frame, (1, 2))?.into_jlrs_result()?;
                assert!(arr_val.value_data(&mut frame).is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn cannot_access_f32_as_value_mut() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let arr_val = Array::new::<f32, _, _, _>(&mut frame, (1, 2))?.into_jlrs_result()?;
                assert!(arr_val.value_data_mut(&mut frame).is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn cannot_access_f32_as_unrestricted_value_mut() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let arr_val = Array::new::<f32, _, _, _>(&mut frame, (1, 2))?.into_jlrs_result()?;
                assert!(arr_val.unrestricted_value_data_mut(&mut frame).is_err());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn convert_back_to_value() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(0, |global, _| {
                let arr_val = Value::an_empty_vec_any(global);
                arr_val.cast::<Array>()?.as_value().is::<Array>();
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn invalid_layout() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let not_arr_val = Value::new(&mut frame, 1usize)?;
                assert!(!ArrayRef::valid_layout(not_arr_val));
                Ok(())
            })
            .unwrap();
        });
    }
}
