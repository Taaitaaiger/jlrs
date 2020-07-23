use jlrs::prelude::*;
use jlrs::traits::ValidLayout;
use jlrs::util::JULIA;
use jlrs::value::jl_string::JlString;
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

#[test]
fn create_and_cast_jl_string() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(1, |_global, frame| {
            let v = Value::new(frame, "Foo bar")?;
            assert!(v.is::<JlString>());
            let string = v.cast::<JlString>()?;
            assert!(unsafe { JlString::valid_layout(v.datatype().unwrap().into()) });
            assert_eq!(string.len(), 7);
            assert_eq!(string.data_cstr().to_str().unwrap(), "Foo bar");
            assert_eq!(string.data_slice(), b"Foo bar".as_ref());

            Ok(())
        })
        .unwrap()
    });
}
