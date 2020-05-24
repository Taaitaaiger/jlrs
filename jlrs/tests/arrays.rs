use jlrs::prelude::*;
mod util;
use util::JULIA;

#[test]
fn cannot_unbox_new_as_array() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let out = jlrs.frame(1, |_, frame| {
            let p = Value::new(frame, 1u8)?;
            p.cast::<Array>()?.copy_inline_data::<u8>()
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
