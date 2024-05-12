mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn create_and_cast_uints() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let p1 = Value::new(&mut frame, 1u8);
                    let p2 = Value::new(&mut frame, 2u16);
                    let p3 = Value::new(&mut frame, 3u32);
                    let p4 = Value::new(&mut frame, 4u64);
                    let p5 = Value::new(&mut frame, 5usize);

                    let u1 = p1.unbox::<u8>()?;
                    let u2 = p2.unbox::<u16>()?;
                    let u3 = p3.unbox::<u32>()?;
                    let u4 = p4.unbox::<u64>()?;
                    let u5 = p5.unbox::<usize>()?;

                    assert_eq!(u1, 1);
                    assert_eq!(u2, 2);
                    assert_eq!(u3, 3);
                    assert_eq!(u4, 4);
                    assert_eq!(u5, 5);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_and_cast_uints_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let p1 = Value::new(&mut frame, 1u8);
                    let p2 = Value::new(&mut frame, 2u16);
                    let p3 = Value::new(&mut frame, 3u32);
                    let p4 = Value::new(&mut frame, 4u64);
                    let p5 = Value::new(&mut frame, 5usize);

                    let u1 = p1.unbox::<u8>()?;
                    let u2 = p2.unbox::<u16>()?;
                    let u3 = p3.unbox::<u32>()?;
                    let u4 = p4.unbox::<u64>()?;
                    let u5 = p5.unbox::<usize>()?;

                    assert_eq!(u1, 1);
                    assert_eq!(u2, 2);
                    assert_eq!(u3, 3);
                    assert_eq!(u4, 4);
                    assert_eq!(u5, 5);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_and_cast_ints() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let p1 = Value::new(&mut frame, 1i8);
                    let p2 = Value::new(&mut frame, 2i16);
                    let p3 = Value::new(&mut frame, 3i32);
                    let p4 = Value::new(&mut frame, 4i64);
                    let p5 = Value::new(&mut frame, 5isize);

                    let u1 = p1.unbox::<i8>()?;
                    let u2 = p2.unbox::<i16>()?;
                    let u3 = p3.unbox::<i32>()?;
                    let u4 = p4.unbox::<i64>()?;
                    let u5 = p5.unbox::<isize>()?;

                    assert_eq!(u1, 1);
                    assert_eq!(u2, 2);
                    assert_eq!(u3, 3);
                    assert_eq!(u4, 4);
                    assert_eq!(u5, 5);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_and_cast_ints_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let p1 = Value::new(&mut frame, 1i8);
                    let p2 = Value::new(&mut frame, 2i16);
                    let p3 = Value::new(&mut frame, 3i32);
                    let p4 = Value::new(&mut frame, 4i64);
                    let p5 = Value::new(&mut frame, 5isize);

                    let u1 = p1.unbox::<i8>()?;
                    let u2 = p2.unbox::<i16>()?;
                    let u3 = p3.unbox::<i32>()?;
                    let u4 = p4.unbox::<i64>()?;
                    let u5 = p5.unbox::<isize>()?;

                    assert_eq!(u1, 1);
                    assert_eq!(u2, 2);
                    assert_eq!(u3, 3);
                    assert_eq!(u4, 4);
                    assert_eq!(u5, 5);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_and_cast_floats() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let p1 = Value::new(&mut frame, 1f32);
                    let p2 = Value::new(&mut frame, 2f64);

                    let u1 = p1.unbox::<f32>()?;
                    let u2 = p2.unbox::<f64>()?;

                    assert_eq!(u1, 1.);
                    assert_eq!(u2, 2.);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_and_cast_floats_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let p1 = Value::new(&mut frame, 1f32);
                    let p2 = Value::new(&mut frame, 2f64);

                    let u1 = p1.unbox::<f32>()?;
                    let u2 = p2.unbox::<f64>()?;

                    assert_eq!(u1, 1.);
                    assert_eq!(u2, 2.);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_and_cast_bool() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let p1 = Value::new(&mut frame, true);
                    let u1 = p1.unbox::<bool>()?.as_bool();
                    assert_eq!(u1, true);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_and_cast_bool_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let p1 = Value::new(&mut frame, false);
                    let u1 = p1.unbox::<bool>()?.as_bool();
                    assert_eq!(u1, false);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_and_cast_char() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let p1 = Value::new(&mut frame, 'a');
                    let u1 = p1.unbox::<char>()?.try_as_char();
                    assert_eq!(u1, Some('a'));
                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_and_cast_char_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let p1 = Value::new(&mut frame, 'a');
                    let u1 = p1.unbox::<char>()?.try_as_char();
                    assert_eq!(u1, Some('a'));
                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_nothing() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    let nothing = Value::nothing(&frame);
                    assert!(nothing.is::<Nothing>());
                    assert!(!nothing.is::<f32>());
                    assert!(nothing.datatype().is::<Nothing>());
                    assert_eq!(nothing.datatype_name(), "Nothing");
                    assert_eq!(nothing.field_names().len(), 0);
                    assert_eq!(nothing.n_fields(), 0);

                    Ok(())
                })
                .unwrap();
        });
    }

    macro_rules! cannot_cast_wrong_type {
        ($name:ident, $val:expr, $from:ty, $to:ty) => {
            fn $name() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();

                    jlrs.instance(&mut frame)
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| {
                            let val = Value::new(&mut frame, $val);
                            assert!(val.is::<$from>());
                            assert!(val.unbox::<$to>().is_err());
                            Ok(())
                        })
                        .unwrap();
                });
            }
        };
    }

    cannot_cast_wrong_type!(cannot_cast_u8_as_u16, 1u8, u8, u16);
    cannot_cast_wrong_type!(cannot_cast_u16_as_u32, 1u16, u16, u32);
    cannot_cast_wrong_type!(cannot_cast_u32_as_u64, 1u32, u32, u64);
    cannot_cast_wrong_type!(cannot_cast_u64_as_i8, 1u64, u64, i8);
    cannot_cast_wrong_type!(cannot_cast_i8_as_i16, 1i8, i8, i16);
    cannot_cast_wrong_type!(cannot_cast_i16_as_i32, 1i16, i16, i32);
    cannot_cast_wrong_type!(cannot_cast_i32_as_i64, 1i32, i32, i64);
    cannot_cast_wrong_type!(cannot_cast_i64_as_u8, 1i64, i64, u8);
    cannot_cast_wrong_type!(cannot_cast_bool_as_char, true, bool, char);
    cannot_cast_wrong_type!(cannot_cast_char_as_bool, 'a', char, bool);
    cannot_cast_wrong_type!(cannot_cast_f32_as_64, 1f32, f32, f64);
    cannot_cast_wrong_type!(cannot_cast_f64_as_32, 1f64, f64, f32);

    unsafe extern "C" fn func() -> bool {
        true
    }

    fn function_pointer() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let val = Value::new(&mut frame, func as *mut std::ffi::c_void);
                    assert!(val.is::<*mut std::ffi::c_void>());

                    let res = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .as_managed()
                            .function(&frame, "callrust")?
                            .as_managed()
                            .call1(&mut frame, val)
                            .unwrap()
                            .unbox::<bool>()?
                            .as_bool()
                    };

                    assert!(res);
                    val.unbox::<*mut std::ffi::c_void>()?;

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn primitives_test() {
        create_and_cast_uints();
        create_and_cast_uints_dynamic();
        create_and_cast_ints();
        create_and_cast_ints_dynamic();
        create_and_cast_floats();
        create_and_cast_floats_dynamic();
        create_and_cast_bool();
        create_and_cast_bool_dynamic();
        create_and_cast_char();
        create_and_cast_char_dynamic();
        create_nothing();
        function_pointer();
        cannot_cast_u8_as_u16();
        cannot_cast_u16_as_u32();
        cannot_cast_u32_as_u64();
        cannot_cast_u64_as_i8();
        cannot_cast_i8_as_i16();
        cannot_cast_i16_as_i32();
        cannot_cast_i32_as_i64();
        cannot_cast_i64_as_u8();
        cannot_cast_bool_as_char();
        cannot_cast_char_as_bool();
        cannot_cast_f32_as_64();
        cannot_cast_f64_as_32();
    }
}
