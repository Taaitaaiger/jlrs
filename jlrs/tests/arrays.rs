use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn array_can_be_cast() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(1, |_, frame| {
            let arr_val = Value::new_array::<f32, _, _, _>(&mut *frame, (1, 2))?;
            let arr = arr_val.cast::<Array>();
            assert!(arr.is_ok());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn array_dimensions() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(1, |_, frame| {
            let arr_val = Value::new_array::<f32, _, _, _>(&mut *frame, (1, 2))?;
            let arr = arr_val.cast::<Array>()?;
            let dims = arr.dimensions();
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

        jlrs.frame(1, |_, frame| {
            let arr_val = Value::new_array::<f32, _, _, _>(&mut *frame, (1, 2))?;
            let arr = arr_val.cast::<Array>()?;
            assert!(arr.contains::<f32>());
            assert!(arr.contains_inline::<f32>());
            assert!(arr.into_typed_array::<f32>().is_ok());
            assert!(arr.into_typed_array::<f64>().is_err());
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

        let out = jlrs.frame(1, |_, frame| {
            let p = Value::new(&mut *frame, 1u8)?;
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

        let out = jlrs.frame(1, |_, frame| {
            let array = Value::new_array::<f32, _, _, _>(&mut *frame, (3, 1))?;
            array.cast::<Array>()?.copy_inline_data::<u8>()
        });

        assert!(out.is_err());
    });
}

#[test]
fn typed_array_can_be_cast() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(1, |_, frame| {
            let arr_val = Value::new_array::<f32, _, _, _>(&mut *frame, (1, 2))?;
            let arr = arr_val.cast::<TypedArray<f32>>();
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

        jlrs.frame(1, |_, frame| {
            let arr_val = Value::new_array::<f32, _, _, _>(&mut *frame, (1, 2))?;
            let arr = arr_val.cast::<TypedArray<f32>>()?;
            let dims = arr.dimensions();
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

        jlrs.frame(1, |_, frame| {
            let arr_val = Value::new_array::<f32, _, _, _>(&mut *frame, (1, 2))?;
            let arr = arr_val.cast::<TypedArray<f32>>()?;
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

        jlrs.frame(0, |global, _| {
            let arr_val = unsafe { Value::an_empty_vec_any(global) };
            assert!(arr_val
                .cast::<Array>()?
                .copy_inline_data::<Value>()
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

        jlrs.frame(0, |global, frame| {
            let arr_val = unsafe { Value::an_empty_vec_any(global) };
            assert!(arr_val
                .cast::<Array>()?
                .inline_data::<Value, _>(&mut *frame)
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

        jlrs.frame(0, |global, frame| {
            let arr_val = unsafe { Value::an_empty_vec_any(global) };
            assert!(arr_val
                .cast::<Array>()?
                .inline_data_mut::<Value, _>(&mut *frame)
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

        jlrs.frame(0, |global, frame| unsafe {
            let arr_val = Value::an_empty_vec_any(global);
            assert!(arr_val
                .cast::<Array>()?
                .unrestricted_inline_data_mut::<Value, _>(&mut *frame)
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

        jlrs.frame(0, |global, frame| unsafe {
            let arr_val = Value::an_empty_vec_any(global);
            assert!(arr_val
                .cast::<Array>()?
                .unrestricted_inline_data_mut::<f64, _>(&mut *frame)
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

        jlrs.frame(1, |_, frame| unsafe {
            let arr_val = Value::new_array::<f32, _, _, _>(&mut *frame, (1, 2))?;
            assert!(arr_val.cast::<Array>()?.value_data(&mut *frame).is_err());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn cannot_access_f32_as_value_mut() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(1, |_, frame| unsafe {
            let arr_val = Value::new_array::<f32, _, _, _>(&mut *frame, (1, 2))?;
            assert!(arr_val
                .cast::<Array>()?
                .value_data_mut(&mut *frame)
                .is_err());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn cannot_access_f32_as_unrestricted_value_mut() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(1, |_, frame| unsafe {
            let arr_val = Value::new_array::<f32, _, _, _>(&mut *frame, (1, 2))?;
            assert!(arr_val
                .cast::<Array>()?
                .unrestricted_value_data_mut(&mut *frame)
                .is_err());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn convert_back_to_value() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(0, |global, _| unsafe {
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

        jlrs.frame(1, |_, frame| unsafe {
            let not_arr_val = Value::new(&mut *frame, 1usize)?;
            assert!(!Array::valid_layout(not_arr_val));
            Ok(())
        })
        .unwrap();
    });
}
