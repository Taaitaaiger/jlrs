use jlrs::prelude::*;
use jlrs::util::JULIA;

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
fn create_and_unbox_output() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(1, |_, frame| {
            let output = frame.output()?;
            let p1 = Value::new_output(frame, output, 1u8);
            let u1 = p1.cast::<u8>()?;
            assert_eq!(u1, 1);

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
fn create_and_unbox_ints() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(5, |_, frame| {
            let p1 = Value::new(frame, 1i8)?;
            let p2 = Value::new(frame, 2i16)?;
            let p3 = Value::new(frame, 3i32)?;
            let p4 = Value::new(frame, 4i64)?;
            let p5 = Value::new(frame, 5isize)?;

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
fn create_and_unbox_ints_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|_, frame| {
            let p1 = Value::new(frame, 1i8)?;
            let p2 = Value::new(frame, 2i16)?;
            let p3 = Value::new(frame, 3i32)?;
            let p4 = Value::new(frame, 4i64)?;
            let p5 = Value::new(frame, 5isize)?;

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
fn create_and_unbox_floats() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(5, |_, frame| {
            let p1 = Value::new(frame, 1f32)?;
            let p2 = Value::new(frame, 2f64)?;

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
fn create_and_unbox_floats_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|_, frame| {
            let p1 = Value::new(frame, 1f32)?;
            let p2 = Value::new(frame, 2f64)?;

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
fn create_and_unbox_bool() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(5, |_, frame| {
            let p1 = Value::new(frame, true)?;
            let u1 = p1.cast::<bool>()?;
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
            let u1 = p1.cast::<bool>()?;
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
            let u1 = p1.cast::<char>()?;
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
            let u1 = p1.cast::<char>()?;
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
            let u1 = p1.value(0)?.cast::<char>()?;
            let u2 = p1.value(1)?.cast::<char>()?;
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
            let u1 = p1.value(0)?.cast::<char>()?;
            let u2 = p1.value(1)?.cast::<char>()?;
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
            let u1 = p1.value(0)?.cast::<u32>()?;
            let u2 = p1.value(1)?.cast::<u64>()?;
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
            let u1 = p1.value(0)?.cast::<u32>()?;
            let u2 = p1.value(1)?.cast::<u64>()?;
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
            assert_eq!(p1.len(), 1);
            let u2 = p1.value(1);
            assert!(u2.is_err());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_values_too_many() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(2, |_, frame| {
            let p1 = Values::new(frame, ['a', 'b', 'c']);
            assert!(p1.is_err());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_values_too_many_dyn() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(2, |_, frame| {
            let p1 = Values::new_dyn(frame, [&'a' as _, &1usize as _, &1isize as _]);
            assert!(p1.is_err());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_nothing() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(0, |global, frame| {
            let nothing = Value::nothing(frame);
            assert!(nothing.is_nothing());
            assert!(!nothing.is::<f32>());
            assert!(nothing.datatype().is_none());
            assert_eq!(nothing.type_name(), "Nothing");
            assert!(!nothing.is_array_of::<f32>());
            assert_eq!(nothing.field_names(global).len(), 0);
            assert_eq!(nothing.n_fields(), 0);

            Ok(())
        })
        .unwrap();
    });
}
