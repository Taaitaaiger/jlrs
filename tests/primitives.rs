use jlrs::prelude::*;
mod util;
use util::JULIA;

#[test]
fn create_and_unbox_uints() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(5, |_, frame| {
            let p1 = Value::new(frame, 1u8)?;
            let p2 = Value::new(frame, 2u16)?;
            let p3 = Value::new(frame, 3u32)?;
            let p4 = Value::new(frame, 4u64)?;
            let p5 = Value::new(frame, 5usize)?;

            let u1 = p1.try_unbox::<u8>()?;
            let u2 = p2.try_unbox::<u16>()?;
            let u3 = p3.try_unbox::<u32>()?;
            let u4 = p4.try_unbox::<u64>()?;
            let u5 = p5.try_unbox::<usize>()?;

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
fn create_and_unbox_uints_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|_, frame| {
            let p1 = Value::new(frame, 1u8)?;
            let p2 = Value::new(frame, 2u16)?;
            let p3 = Value::new(frame, 3u32)?;
            let p4 = Value::new(frame, 4u64)?;
            let p5 = Value::new(frame, 5usize)?;

            let u1 = p1.try_unbox::<u8>()?;
            let u2 = p2.try_unbox::<u16>()?;
            let u3 = p3.try_unbox::<u32>()?;
            let u4 = p4.try_unbox::<u64>()?;
            let u5 = p5.try_unbox::<usize>()?;

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
fn create_and_unbox_ints() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(5, |_, frame| {
            let p1 = Value::new(frame, 1i8)?;
            let p2 = Value::new(frame, 2i16)?;
            let p3 = Value::new(frame, 3i32)?;
            let p4 = Value::new(frame, 4i64)?;
            let p5 = Value::new(frame, 5isize)?;

            let u1 = p1.try_unbox::<i8>()?;
            let u2 = p2.try_unbox::<i16>()?;
            let u3 = p3.try_unbox::<i32>()?;
            let u4 = p4.try_unbox::<i64>()?;
            let u5 = p5.try_unbox::<isize>()?;

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
fn create_and_unbox_ints_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|_, frame| {
            let p1 = Value::new(frame, 1i8)?;
            let p2 = Value::new(frame, 2i16)?;
            let p3 = Value::new(frame, 3i32)?;
            let p4 = Value::new(frame, 4i64)?;
            let p5 = Value::new(frame, 5isize)?;

            let u1 = p1.try_unbox::<i8>()?;
            let u2 = p2.try_unbox::<i16>()?;
            let u3 = p3.try_unbox::<i32>()?;
            let u4 = p4.try_unbox::<i64>()?;
            let u5 = p5.try_unbox::<isize>()?;

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
fn create_and_unbox_floats() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(5, |_, frame| {
            let p1 = Value::new(frame, 1f32)?;
            let p2 = Value::new(frame, 2f64)?;

            let u1 = p1.try_unbox::<f32>()?;
            let u2 = p2.try_unbox::<f64>()?;

            assert_eq!(u1, 1.);
            assert_eq!(u2, 2.);

            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_unbox_floats_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|_, frame| {
            let p1 = Value::new(frame, 1f32)?;
            let p2 = Value::new(frame, 2f64)?;

            let u1 = p1.try_unbox::<f32>()?;
            let u2 = p2.try_unbox::<f64>()?;

            assert_eq!(u1, 1.);
            assert_eq!(u2, 2.);

            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_unbox_bool() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(5, |_, frame| {
            let p1 = Value::new(frame, true)?;
            let u1 = p1.try_unbox::<bool>()?;
            assert_eq!(u1, true);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_unbox_bool_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|_, frame| {
            let p1 = Value::new(frame, false)?;
            let u1 = p1.try_unbox::<bool>()?;
            assert_eq!(u1, false);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_unbox_char() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(5, |_, frame| {
            let p1 = Value::new(frame, 'a')?;
            let u1 = p1.try_unbox::<char>()?;
            assert_eq!(u1, 'a');
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_unbox_char_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|_, frame| {
            let p1 = Value::new(frame, 'a')?;
            let u1 = p1.try_unbox::<char>()?;
            assert_eq!(u1, 'a');
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_unbox_values() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(2, |_, frame| {
            let p1 = Values::new(frame, ['a', 'b'])?;
            let u1 = p1.value(0)?.try_unbox::<char>()?;
            let u2 = p1.value(1)?.try_unbox::<char>()?;
            assert_eq!(u1, 'a');
            assert_eq!(u2, 'b');
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_unbox_values_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|_, frame| {
            let p1 = Values::new(frame, ['a', 'b'])?;
            let u1 = p1.value(0)?.try_unbox::<char>()?;
            let u2 = p1.value(1)?.try_unbox::<char>()?;
            assert_eq!(u1, 'a');
            assert_eq!(u2, 'b');
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_unbox_dyn_values() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(2, |_, frame| {
            let p1 = Values::new_dyn(frame, [&1u32 as _, &2u64 as _])?;
            let u1 = p1.value(0)?.try_unbox::<u32>()?;
            let u2 = p1.value(1)?.try_unbox::<u64>()?;
            assert_eq!(u1, 1u32);
            assert_eq!(u2, 2u64);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_and_unbox_dyn_values_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|_, frame| {
            let p1 = Values::new_dyn(frame, [&1u32 as _, &2u64 as _])?;
            let u1 = p1.value(0)?.try_unbox::<u32>()?;
            let u2 = p1.value(1)?.try_unbox::<u64>()?;
            assert_eq!(u1, 1u32);
            assert_eq!(u2, 2u64);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_values_get_out_of_bounds() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(2, |_, frame| {
            let p1 = Values::new(frame, ['a'])?;
            let u2 = p1.value(1);
            assert!(u2.is_err());
            Ok(())
        })
        .unwrap();
    });
}
