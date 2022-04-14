mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;
    use jlrs::wrappers::ptr::string::JuliaString;
    use jlrs::{layout::valid_layout::ValidLayout, wrappers::ptr::StringRef};
    use std::borrow::Cow;

    #[test]
    fn create_and_unbox_str_data() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unwrapped_string = jlrs
                .scope_with_capacity(1, |_, frame| {
                    frame.scope_with_capacity(1, |frame| {
                        let string = JuliaString::new(frame, "Hellõ world!")?;
                        Ok(string.as_str()?.to_string())
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
                .scope_with_capacity(1, |_, frame| {
                    frame.scope_with_capacity(1, |frame| {
                        let string = JuliaString::new(frame, String::from("Hellõ world!"))?;
                        Ok(string.as_str()?.to_string())
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
                .scope_with_capacity(1, |_, frame| {
                    frame.scope_with_capacity(1, |frame| {
                        let string = JuliaString::new(frame, Cow::from("Hellõ world!"))?;
                        Ok(string.as_str()?.to_string())
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

            jlrs.scope_with_capacity(1, |_global, frame| {
                let v = JuliaString::new(frame, "Foo bar")?;
                assert!(v.as_value().is::<JuliaString>());
                let string = v.as_value().cast::<JuliaString>()?;
                assert!(StringRef::valid_layout(v.as_value().datatype().as_value()));
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
}
