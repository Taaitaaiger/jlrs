use jlrs::prelude::*;

#[test]
fn create_and_unbox_string_data() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unwrapped_string = jlrs
        .session(|session| {
            let string = session.new_string("Hellõ world!")?;
            session.execute(|exec_ctx| exec_ctx.try_unbox::<String>(&string))
        })
        .unwrap();

    assert_eq!(unwrapped_string, "Hellõ world!");
}

#[test]
fn create_and_unbox_string_data_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    let rs_string = String::from("Hellõ world!");

    let unwrapped_string = jlrs
        .session(|session| {
            session.with_temporaries(|mut alloc_ctx| {
                let string = alloc_ctx.new_string(&rs_string)?;
                alloc_ctx.execute(|exec_ctx| exec_ctx.try_unbox::<String>(&string))
            })
        })
        .unwrap();

    assert_eq!(unwrapped_string, "Hellõ world!");
}
