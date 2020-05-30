use jlrs::prelude::*;
use jlrs::util::JULIA;

use std::borrow::Cow;

#[test]
fn create_and_unbox_str_data() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unwrapped_string = jlrs
            .frame(1, |_, frame| {
                frame.frame(1, |frame| {
                    let string = Value::new(frame, "Hellõ world!")?;
                    string.cast::<String>()
                })
            })
            .unwrap();

        assert_eq!(unwrapped_string, "Hellõ world!");
    });
}

#[test]
fn create_and_unbox_string_data() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unwrapped_string = jlrs
            .frame(1, |_, frame| {
                frame.frame(1, |frame| {
                    let string = Value::new(frame, String::from("Hellõ world!"))?;
                    string.cast::<String>()
                })
            })
            .unwrap();

        assert_eq!(unwrapped_string, "Hellõ world!");
    });
}

#[test]
fn create_and_unbox_cow_data() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unwrapped_string = jlrs
            .frame(1, |_, frame| {
                frame.frame(1, |frame| {
                    let string = Value::new(frame, Cow::from("Hellõ world!"))?;
                    string.cast::<String>()
                })
            })
            .unwrap();

        assert_eq!(unwrapped_string, "Hellõ world!");
    });
}
