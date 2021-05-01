use jlrs::prelude::*;
use jlrs::util::JULIA;
use jlrs::value::string::JuliaString;
use std::borrow::Cow;

#[test]
fn create_and_unbox_str_data() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unwrapped_string = jlrs
            .scope_with_slots(1, |_, frame| {
                frame.scope_with_slots(1, |frame| {
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
            .scope_with_slots(1, |_, frame| {
                frame.scope_with_slots(1, |frame| {
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
            .scope_with_slots(1, |_, frame| {
                frame.scope_with_slots(1, |frame| {
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

        jlrs.scope_with_slots(1, |_global, frame| {
            let v = Value::new(frame, "Foo bar")?;
            assert!(v.is::<JuliaString>());
            let string = v.cast::<JuliaString>()?;
            assert!(unsafe { JuliaString::valid_layout(v.datatype().into()) });
            assert_eq!(string.len(), 7);
            assert_eq!(string.as_c_str().to_str().unwrap(), "Foo bar");
            assert_eq!(string.as_str().unwrap(), "Foo bar");
            assert_eq!(unsafe { string.as_str_unchecked() }, "Foo bar");
            assert_eq!(string.as_slice(), b"Foo bar".as_ref());

            Ok(())
        })
        .unwrap()
    });
}
