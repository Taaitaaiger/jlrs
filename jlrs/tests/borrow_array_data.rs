mod util;

#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use jlrs::{
        memory::gc::{Gc, GcCollection},
        prelude::*,
        wrappers::ptr::array::dimensions::Dims,
    };

    use crate::util::JULIA;

    macro_rules! impl_test {
        ($name:ident, $name_mut:ident, $name_slice:ident, $name_slice_mut:ident, $value_type:ty) => {
            fn $name() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();
                    let mut jlrs = jlrs.instance(&mut frame);

                    jlrs.scope(|mut frame| unsafe {
                        let data: Vec<$value_type> = (1..=24).map(|x| x as $value_type).collect();

                        let array = Array::from_vec(frame.as_extended_target(), data, (2, 3, 4))?
                            .into_jlrs_result()?;
                        let d = array.inline_data::<$value_type>()?;

                        let mut out = 1 as $value_type;
                        for third in &[0, 1, 2, 3] {
                            for second in &[0, 1, 2] {
                                for first in &[0, 1] {
                                    assert_eq!(d[[*first, *second, *third]], out);
                                    out += 1 as $value_type;
                                }
                            }
                        }

                        let gi = Module::base(&frame).function(&frame, "getindex")?.wrapper();
                        let one = Value::new(&mut frame, 1usize);
                        let two = Value::new(&mut frame, 2usize);
                        let three = Value::new(&mut frame, 3usize);
                        let four = Value::new(&mut frame, 4usize);

                        out = 1 as $value_type;
                        for third in &[one, two, three, four] {
                            for second in &[one, two, three] {
                                for first in &[one, two] {
                                    frame.scope(|mut frame| {
                                        let v = gi
                                            .call(
                                                &mut frame,
                                                &mut [array.as_value(), *first, *second, *third],
                                            )
                                            .unwrap();
                                        assert_eq!(v.unbox::<$value_type>()?, out);
                                        out += 1 as $value_type;
                                        Ok(())
                                    })?;
                                }
                            }
                        }

                        Ok(())
                    })
                    .unwrap();

                    jlrs.gc_collect(GcCollection::Full);
                });
            }

            fn $name_mut() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();

                    jlrs.instance(&mut frame)
                        .scope(|mut frame| unsafe {
                            let data: Vec<$value_type> =
                                (1..=24).map(|x| x as $value_type).collect();

                            let mut array =
                                Array::from_vec(frame.as_extended_target(), data, (2, 3, 4))?
                                    .into_jlrs_result()?;
                            let mut d = array.bits_data_mut::<$value_type>()?;

                            for third in &[0, 1, 2, 3] {
                                for second in &[0, 1, 2] {
                                    for first in &[0, 1] {
                                        d[(*first, *second, *third)] += 1 as $value_type;
                                    }
                                }
                            }
                            let gi = Module::base(&frame).function(&frame, "getindex")?.wrapper();
                            let one = Value::new(&mut frame, 1usize);
                            let two = Value::new(&mut frame, 2usize);
                            let three = Value::new(&mut frame, 3usize);
                            let four = Value::new(&mut frame, 4usize);

                            let mut out = 2 as $value_type;
                            for third in &[one, two, three, four] {
                                for second in &[one, two, three] {
                                    for first in &[one, two] {
                                        frame.scope(|mut frame| {
                                            let v = gi
                                                .call(
                                                    &mut frame,
                                                    &mut [
                                                        array.as_value(),
                                                        *first,
                                                        *second,
                                                        *third,
                                                    ],
                                                )
                                                .unwrap();
                                            assert_eq!(v.unbox::<$value_type>()?, out);
                                            out += 1 as $value_type;
                                            Ok(())
                                        })?;
                                    }
                                }
                            }

                            Ok(())
                        })
                        .unwrap();
                });
            }

            fn $name_slice() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();

                    jlrs.instance(&mut frame)
                        .scope(|mut frame| unsafe {
                            let data: Vec<$value_type> =
                                (1..=24).map(|x| x as $value_type).collect();

                            let array = Array::from_vec(
                                frame.as_extended_target(),
                                data.clone(),
                                (2, 3, 4),
                            )?
                            .into_jlrs_result()?;
                            let d = array.inline_data::<$value_type>()?;

                            for (a, b) in data.iter().zip(d.as_slice()) {
                                assert_eq!(a, b)
                            }

                            Ok(())
                        })
                        .unwrap();
                });
            }

            fn $name_slice_mut() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();

                    jlrs.instance(&mut frame)
                        .scope(|mut frame| unsafe {
                            let data: Vec<$value_type> =
                                (1..=24).map(|x| x as $value_type).collect();

                            let mut array = Array::from_vec(
                                frame.as_extended_target(),
                                data.clone(),
                                (2, 3, 4),
                            )?
                            .into_jlrs_result()?;
                            let mut d = array.bits_data_mut::<$value_type>()?;

                            for (a, b) in data.iter().zip(d.as_mut_slice()) {
                                assert_eq!(a, b)
                            }

                            Ok(())
                        })
                        .unwrap();
                });
            }
        };
    }

    impl_test!(
        array_data_3d_u8,
        array_data_3d_u8_mut,
        array_data_3d_u8_slice,
        array_data_3d_u8_mut_slice,
        u8
    );
    impl_test!(
        array_data_3d_u16,
        array_data_3d_u16_mut,
        array_data_3d_u16_slice,
        array_data_3d_u16_mut_slice,
        u16
    );
    impl_test!(
        array_data_3d_u32,
        array_data_3d_u32_mut,
        array_data_3d_u32_slice,
        array_data_3d_u32_mut_slice,
        u32
    );
    impl_test!(
        array_data_3d_u64,
        array_data_3d_u64_mut,
        array_data_3d_u64_slice,
        array_data_3d_u64_mut_slice,
        u64
    );
    impl_test!(
        array_data_3d_i8,
        array_data_3d_i8_mut,
        array_data_3d_i8_slice,
        array_data_3d_i8_mut_slice,
        i8
    );
    impl_test!(
        array_data_3d_i16,
        array_data_3d_i16_mut,
        array_data_3d_i16_slice,
        array_data_3d_i16_mut_slice,
        i16
    );
    impl_test!(
        array_data_3d_i32,
        array_data_3d_i32_mut,
        array_data_3d_i32_slice,
        array_data_3d_i32_mut_slice,
        i32
    );
    impl_test!(
        array_data_3d_i64,
        array_data_3d_i64_mut,
        array_data_3d_i64_slice,
        array_data_3d_i64_mut_slice,
        i64
    );
    impl_test!(
        array_data_3d_f32,
        array_data_3d_f32_mut,
        array_data_3d_f32_slice,
        array_data_3d_f32_mut_slice,
        f32
    );
    impl_test!(
        array_data_3d_f64,
        array_data_3d_f64_mut,
        array_data_3d_f64_slice,
        array_data_3d_f64_mut_slice,
        f64
    );

    fn borrow_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let data: Vec<u8> = (1..=24).map(|x| x as u8).collect();

                    let array = Array::from_vec(frame.as_extended_target(), data, (2, 3, 4))?
                        .into_jlrs_result()?;

                    frame.scope(|mut frame| {
                        let d = { array.inline_data::<u8>()? };

                        let mut out = 1 as u8;
                        for third in &[0, 1, 2, 3] {
                            for second in &[0, 1, 2] {
                                for first in &[0, 1] {
                                    assert_eq!(d[(*first, *second, *third)], out);
                                    out += 1 as u8;
                                }
                            }
                        }

                        let gi = Module::base(&frame).function(&frame, "getindex")?.wrapper();
                        let one = Value::new(&mut frame, 1usize);
                        let two = Value::new(&mut frame, 2usize);
                        let three = Value::new(&mut frame, 3usize);
                        let four = Value::new(&mut frame, 4usize);

                        out = 1 as u8;
                        for third in &[one, two, three, four] {
                            for second in &[one, two, three] {
                                for first in &[one, two] {
                                    frame.scope(|mut frame| {
                                        let v = gi
                                            .call(
                                                &mut frame,
                                                &mut [array.as_value(), *first, *second, *third],
                                            )
                                            .unwrap();
                                        assert_eq!(v.unbox::<u8>()?, out);
                                        out += 1 as u8;
                                        Ok(())
                                    })?;
                                }
                            }
                        }

                        Ok(())
                    })
                })
                .unwrap();
        });
    }

    fn access_borrowed_array_dimensions() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let arr = arr_val;

                    let data = unsafe { arr.inline_data::<f32>()? };
                    assert_eq!(data.dimensions().into_dimensions().as_slice(), &[1, 2]);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_mutable_borrowed_array_dimensions() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let mut arr = arr_val;

                    let data = unsafe { arr.inline_data_mut::<f32>()? };
                    assert_eq!(data.dimensions().into_dimensions().as_slice(), &[1, 2]);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn value_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    unsafe {
                        let arr = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .function(&frame, "vecofmodules")?
                            .wrapper()
                            .call0(&mut frame)
                            .unwrap()
                            .cast::<Array>()?;
                        let data = { arr.value_data()? };

                        assert!(data[0].unwrap().wrapper().is::<Module>());
                    }
                    Ok(())
                })
                .unwrap();
        })
    }

    fn value_data_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    unsafe {
                        let submod = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper();
                        let mut arr = submod
                            .function(&frame, "vecofmodules")?
                            .wrapper()
                            .call0(&mut frame)
                            .unwrap()
                            .cast::<Array>()?;
                        let mut data = { arr.value_data_mut()? };
                        data.set(0, Some(submod.as_value()))?;

                        let getindex = Module::base(&frame).function(&frame, "getindex")?.wrapper();
                        let idx = Value::new(&mut frame, 1usize);
                        let entry = getindex
                            .call2(&mut frame, arr.as_value(), idx)
                            .unwrap()
                            .cast::<Module>()?;

                        assert_eq!(entry.name().hash(), submod.name().hash());
                    }
                    Ok(())
                })
                .unwrap();
        })
    }

    fn typed_array_value_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    unsafe {
                        let arr = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .function(&frame, "vecofmodules")?
                            .wrapper()
                            .call0(&mut frame)
                            .unwrap()
                            .cast::<TypedArray<Option<ModuleRef>>>()?;
                        let data = { arr.value_data()? };

                        assert!(data[0].unwrap().wrapper().is::<Module>());
                    }
                    Ok(())
                })
                .unwrap();
        })
    }

    fn typed_array_value_data_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    unsafe {
                        let submod = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper();
                        let mut arr = submod
                            .function(&frame, "vecofmodules")?
                            .wrapper()
                            .call0(&mut frame)
                            .unwrap()
                            .cast::<TypedArray<Option<ModuleRef>>>()?;
                        let mut data = { arr.value_data_mut()? };
                        data.set(0, Some(submod.as_value()))?;

                        let getindex = Module::base(&frame).function(&frame, "getindex")?.wrapper();
                        let idx = Value::new(&mut frame, 1usize);
                        let entry = getindex
                            .call2(&mut frame, arr.as_value(), idx)
                            .unwrap()
                            .cast::<Module>()?;

                        assert_eq!(entry.name().hash(), submod.name().hash());
                    }
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn borrow_array_data_tests() {
        array_data_3d_u8();
        array_data_3d_u8_mut();
        array_data_3d_u8_slice();
        array_data_3d_u8_mut_slice();
        array_data_3d_u16();
        array_data_3d_u16_mut();
        array_data_3d_u16_slice();
        array_data_3d_u16_mut_slice();
        array_data_3d_u32();
        array_data_3d_u32_mut();
        array_data_3d_u32_slice();
        array_data_3d_u32_mut_slice();
        array_data_3d_u64();
        array_data_3d_u64_mut();
        array_data_3d_u64_slice();
        array_data_3d_u64_mut_slice();
        array_data_3d_i8();
        array_data_3d_i8_mut();
        array_data_3d_i8_slice();
        array_data_3d_i8_mut_slice();
        array_data_3d_i16();
        array_data_3d_i16_mut();
        array_data_3d_i16_slice();
        array_data_3d_i16_mut_slice();
        array_data_3d_i32();
        array_data_3d_i32_mut();
        array_data_3d_i32_slice();
        array_data_3d_i32_mut_slice();
        array_data_3d_i64();
        array_data_3d_i64_mut();
        array_data_3d_i64_slice();
        array_data_3d_i64_mut_slice();
        array_data_3d_f32();
        array_data_3d_f32_mut();
        array_data_3d_f32_slice();
        array_data_3d_f32_mut_slice();
        array_data_3d_f64();
        array_data_3d_f64_mut();
        array_data_3d_f64_slice();
        array_data_3d_f64_mut_slice();
        borrow_nested();
        access_borrowed_array_dimensions();
        access_mutable_borrowed_array_dimensions();
        value_data();
        value_data_mut();
        typed_array_value_data();
        typed_array_value_data_mut();
    }
}
