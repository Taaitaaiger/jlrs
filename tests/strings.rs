use jlrs::prelude::*;
mod util;
use util::JULIA;

#[test]
fn create_and_unbox_string_data() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unwrapped_string = jlrs
            .frame(1, |_, frame| {
                frame.frame(1, |frame| {
                    let string = Value::new(frame, "Hellõ world!")?;
                    string.try_unbox::<String>()
                })
            })
            .unwrap();

        assert_eq!(unwrapped_string, "Hellõ world!");
    });
}
