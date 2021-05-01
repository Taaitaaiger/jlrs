use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn create_and_cast_uints() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(5, |_, frame| {
            let p1 = Value::new(&mut *frame, 1u8)?;
            let p2 = Value::new(&mut *frame, 2u16)?;
            let p3 = Value::new(&mut *frame, 3u32)?;
            let p4 = Value::new(&mut *frame, 4u64)?;
            let p5 = Value::new(&mut *frame, 5usize)?;

            let u1 = p1.cast::<u8>()?;
            let u2 = p2.cast::<u16>()?;
            let u3 = p3.cast::<u32>()?;
            let u4 = p4.cast::<u64>()?;
            let u5 = p5.cast::<usize>()?;

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

#[test]
fn create_and_cast_uints_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|_, frame| {
            let p1 = Value::new(&mut *frame, 1u8)?;
            let p2 = Value::new(&mut *frame, 2u16)?;
            let p3 = Value::new(&mut *frame, 3u32)?;
            let p4 = Value::new(&mut *frame, 4u64)?;
            let p5 = Value::new(&mut *frame, 5usize)?;

            let u1 = p1.cast::<u8>()?;
            let u2 = p2.cast::<u16>()?;
            let u3 = p3.cast::<u32>()?;
            let u4 = p4.cast::<u64>()?;
            let u5 = p5.cast::<usize>()?;

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

#[test]
fn create_and_cast_ints() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(5, |_, frame| {
            let p1 = Value::new(&mut *frame, 1i8)?;
            let p2 = Value::new(&mut *frame, 2i16)?;
            let p3 = Value::new(&mut *frame, 3i32)?;
            let p4 = Value::new(&mut *frame, 4i64)?;
            let p5 = Value::new(&mut *frame, 5isize)?;

            let u1 = p1.cast::<i8>()?;
            let u2 = p2.cast::<i16>()?;
            let u3 = p3.cast::<i32>()?;
            let u4 = p4.cast::<i64>()?;
            let u5 = p5.cast::<isize>()?;

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

#[test]
fn create_and_cast_ints_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|_, frame| {
            let p1 = Value::new(&mut *frame, 1i8)?;
            let p2 = Value::new(&mut *frame, 2i16)?;
            let p3 = Value::new(&mut *frame, 3i32)?;
            let p4 = Value::new(&mut *frame, 4i64)?;
            let p5 = Value::new(&mut *frame, 5isize)?;

            let u1 = p1.cast::<i8>()?;
            let u2 = p2.cast::<i16>()?;
            let u3 = p3.cast::<i32>()?;
            let u4 = p4.cast::<i64>()?;
            let u5 = p5.cast::<isize>()?;

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

#[test]
fn create_and_cast_floats() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(5, |_, frame| {
            let p1 = Value::new(&mut *frame, 1f32)?;
            let p2 = Value::new(&mut *frame, 2f64)?;

            let u1 = p1.cast::<f32>()?;
            let u2 = p2.cast::<f64>()?;

            assert_eq!(u1, 1.);
            assert_eq!(u2, 2.);

            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_cast_floats_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|_, frame| {
            let p1 = Value::new(&mut *frame, 1f32)?;
            let p2 = Value::new(&mut *frame, 2f64)?;

            let u1 = p1.cast::<f32>()?;
            let u2 = p2.cast::<f64>()?;

            assert_eq!(u1, 1.);
            assert_eq!(u2, 2.);

            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_cast_bool() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(5, |_, frame| {
            let p1 = Value::new(&mut *frame, true)?;
            let u1 = p1.cast::<bool>()?;
            assert_eq!(u1, true);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_cast_bool_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|_, frame| {
            let p1 = Value::new(&mut *frame, false)?;
            let u1 = p1.cast::<bool>()?;
            assert_eq!(u1, false);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_cast_char() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(5, |_, frame| {
            let p1 = Value::new(&mut *frame, 'a')?;
            let u1 = p1.cast::<char>()?;
            assert_eq!(u1, 'a');
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_cast_char_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|_, frame| {
            let p1 = Value::new(&mut *frame, 'a')?;
            let u1 = p1.cast::<char>()?;
            assert_eq!(u1, 'a');
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_nothing() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(0, |global, _frame| {
            let nothing = Value::nothing(global);
            assert!(nothing.is::<Nothing>());
            assert!(!nothing.is::<f32>());
            assert!(nothing.datatype().is::<Nothing>());
            assert_eq!(nothing.type_name().unwrap(), "Nothing");
            assert!(!nothing.is_array_of::<f32>());
            assert_eq!(nothing.field_names().len(), 0);
            assert_eq!(nothing.n_fields(), 0);

            Ok(())
        })
        .unwrap();
    });
}

macro_rules! cannot_cast_wrong_type {
    ($name:ident, $val:expr, $from:ty, $to:ty) => {
        #[test]
        fn $name() {
            JULIA.with(|j| {
                let mut jlrs = j.borrow_mut();

                jlrs.scope_with_slots(1, |_global, frame| {
                    let val = Value::new(&mut *frame, $val)?;
                    assert!(val.is::<$from>());
                    assert!(val.cast::<$to>().is_err());
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

#[test]
fn function_pointer() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(2, |global, frame| {
            let val = Value::new(&mut *frame, func as *mut std::ffi::c_void)?;
            assert!(val.is::<*mut std::ffi::c_void>());

            let res = Module::main(global)
                .submodule("JlrsTests")?
                .function("callrust")?
                .call1(&mut *frame, val)?
                .unwrap()
                .cast::<bool>()?;

            assert!(res);
            val.cast::<*mut std::ffi::c_void>()?;

            Ok(())
        })
        .unwrap();
    });
}
