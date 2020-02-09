use jlrs::prelude::*;

#[test]
fn cannot_unbox_primitive_as_array() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let out = jlrs.session(|session| {
        let p = session.new_primitive(1u8)?;
        session.execute(|exec_ctx| exec_ctx.try_unbox::<UnboxedArray<u8>>(&p))
    });

    assert!(out.is_err());
}

#[test]
fn cannot_unbox_array_with_wrong_type() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let out = jlrs.session(|session| {
        let array = session.new_managed_array::<f32, _>(3)?;
        session.execute(|exec_ctx| {
            let array = array.set_all(exec_ctx, 2.0)?;
            exec_ctx.try_unbox::<UnboxedArray<u8>>(&array)
        })
    });

    assert!(out.is_err());
}
