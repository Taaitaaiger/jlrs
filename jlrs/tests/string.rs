mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use std::borrow::Cow;

    use jlrs::{
        data::{layout::valid_layout::ValidLayout, managed::string::JuliaString},
        prelude::*,
    };

    use crate::util::JULIA;

    fn create_and_unbox_str_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unwrapped_string = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let string = JuliaString::new(&mut frame, "Hellõ world!");
                        Ok(string.as_str()?.to_string())
                    })
                })
                .unwrap();

            assert_eq!(unwrapped_string, "Hellõ world!");
        });
    }

    fn create_and_unbox_string_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unwrapped_string = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let string = JuliaString::new(&mut frame, String::from("Hellõ world!"));
                        Ok(string.as_str()?.to_string())
                    })
                })
                .unwrap();

            assert_eq!(unwrapped_string, "Hellõ world!");
        });
    }

    fn create_and_unbox_cow_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unwrapped_string = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let string = JuliaString::new(&mut frame, Cow::from("Hellõ world!"));
                        Ok(string.as_str()?.to_string())
                    })
                })
                .unwrap();

            assert_eq!(unwrapped_string, "Hellõ world!");
        });
    }

    fn create_and_cast_jl_string() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let v = JuliaString::new(&mut frame, "Foo bar");
                    assert!(v.as_value().is::<JuliaString>());
                    let string = v.as_value().cast::<JuliaString>()?;
                    assert!(StringRef::valid_layout(v.as_value().datatype().as_value()));
                    assert_eq!(string.len(), 7);
                    assert_eq!(string.as_c_str().to_str().unwrap(), "Foo bar");
                    assert_eq!(string.as_str().unwrap(), "Foo bar");
                    assert_eq!(unsafe { string.as_str_unchecked() }, "Foo bar");
                    assert_eq!(string.as_bytes(), b"Foo bar".as_ref());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_non_utf8_string() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let string = JuliaString::new_bytes(&mut frame, &[129, 2, 0, 0]);
                    assert!(string.as_value().is::<JuliaString>());
                    assert!(StringRef::valid_layout(
                        string.as_value().datatype().as_value()
                    ));
                    assert_eq!(string.len(), 4);

                    let r: &[u8] = string.as_c_str().to_bytes();
                    assert_eq!(r.len(), 2);
                    let res = string.as_value().unbox::<String>()?;
                    assert!(res.is_err());
                    let vec = res.unwrap_err();
                    assert_eq!(vec.len(), 4);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_utf8_string() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let string = JuliaString::new_bytes(&mut frame, &[1]);

                    let res = string.as_value().unbox::<String>()?;
                    assert!(res.is_ok());
                    let vec = res.unwrap();
                    assert_eq!(vec.len(), 1);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn format_string() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let string1 = JuliaString::new_bytes(&mut frame, &[129, 2, 0, 0]);
                    let string2 = JuliaString::new(&mut frame, "Foo").clone();

                    let f1 = format!("{:?}", string1);
                    assert_eq!(f1, String::from("<Non-UTF8 string>"));
                    let f2 = format!("{:?}", string2);
                    assert_eq!(f2, String::from("Foo"));

                    Ok(())
                })
                .unwrap();
        });
    }

    fn extend_lifeime() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame).scope(|mut frame| {
                let output = frame.output();

                frame.scope(|mut frame| {
                    let string = JuliaString::new(&mut frame, "Foo");
                    string.root(output)
                });
            });
        });
    }

    #[test]
    fn string_tests() {
        create_and_unbox_str_data();
        create_and_unbox_string_data();
        create_and_unbox_cow_data();
        create_and_cast_jl_string();
        create_non_utf8_string();
        create_utf8_string();
        format_string();
        extend_lifeime();
    }
}
