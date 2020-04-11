use jlrs::prelude::*;

#[test]
fn create_and_unbox_string_data() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unwrapped_string = jlrs
        .frame(1, |frame| {
            frame.frame(1, |frame| {
                let string = Value::new(frame, "Hellõ world!")?;
                string.try_unbox::<String>()
            })
        })
        .unwrap();

    assert_eq!(unwrapped_string, "Hellõ world!");
}
