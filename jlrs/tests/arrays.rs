use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn array_can_be_cast() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(1, |_, frame| {
            let arr_val = Value::new_array::<f32, _, _>(frame, (1, 2))?;
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
            let arr_val = Value::new_array::<f32, _, _>(frame, (1, 2))?;
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
            let arr_val = Value::new_array::<f32, _, _>(frame, (1, 2))?;
            let arr = arr_val.cast::<Array>()?;
            assert!(arr.contains::<f32>());
            assert!(arr.contains_inline::<f32>());
            assert!(!arr.has_inlined_pointers());
            assert!(arr.is_inline_array());
            assert!(!arr.is_value_array());
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
            let p = Value::new(frame, 1u8)?;
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
            let array = Value::new_array::<f32, _, _>(frame, (3, 1))?;
            array.cast::<Array>()?.copy_inline_data::<u8>()
        });

        assert!(out.is_err());
    });
}
