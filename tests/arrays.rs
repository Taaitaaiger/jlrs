use jlrs::prelude::*;

#[test]
fn cannot_unbox_new_as_array() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let out = jlrs.frame(1, |frame| {
        let p = Value::new(frame, 1u8)?;
        p.try_unbox::<Array<u8>>()
    });

    assert!(out.is_err());
}

#[test]
fn cannot_unbox_array_with_wrong_type() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let out = jlrs.frame(1, |frame| {
        let array = Value::array::<f32, _, _>(frame, (3, 1))?;
        array.try_unbox::<Array<u8>>()
    });

    assert!(out.is_err());
}