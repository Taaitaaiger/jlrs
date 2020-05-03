use jlrs::prelude::*;
mod util;
use util::JULIA;

macro_rules! impl_test {
    ($name:ident, $name_mut:ident, $value_type:ty) => {
        #[test]
        fn $name() {
            JULIA.with(|j| {
                let mut jlrs = j.borrow_mut();

                jlrs.frame(5, |global, frame| {
                    let data: Vec<$value_type> = (1..=24).map(|x| x as $value_type).collect();

                    let array = Value::move_array(frame, data, (2, 3, 4))?;
                    let d = array.array_data::<$value_type, _>(frame)?;

                    let mut out = 1 as $value_type;
                    for third in &[0, 1, 2, 3] {
                        for second in &[0, 1, 2] {
                            for first in &[0, 1] {
                                assert_eq!(d[(*first, *second, *third)], out);
                                out += 1 as $value_type;
                            }
                        }
                    }

                    let gi = Module::base(global).function("getindex")?;
                    let one = Value::new(frame, 1usize)?;
                    let two = Value::new(frame, 2usize)?;
                    let three = Value::new(frame, 3usize)?;
                    let four = Value::new(frame, 4usize)?;

                    out = 1 as $value_type;
                    for third in &[one, two, three, four] {
                        for second in &[one, two, three] {
                            for first in &[one, two] {
                                frame.frame(1, |frame| {
                                    let v =
                                        gi.call(frame, [array, *first, *second, *third])?.unwrap();
                                    assert_eq!(v.try_unbox::<$value_type>()?, out);
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

        #[test]
        fn $name_mut() {
            JULIA.with(|j| {
                let mut jlrs = j.borrow_mut();

                jlrs.frame(5, |global, frame| {
                    let data: Vec<$value_type> = (1..=24).map(|x| x as $value_type).collect();

                    let mut array = Value::move_array(frame, data, (2, 3, 4))?;
                    let mut d = array.array_data_mut::<$value_type, _>(frame)?;

                    for third in &[0, 1, 2, 3] {
                        for second in &[0, 1, 2] {
                            for first in &[0, 1] {
                                d[(*first, *second, *third)] += 1 as $value_type;
                            }
                        }
                    }
                    let gi = Module::base(global).function("getindex")?;
                    let one = Value::new(frame, 1usize)?;
                    let two = Value::new(frame, 2usize)?;
                    let three = Value::new(frame, 3usize)?;
                    let four = Value::new(frame, 4usize)?;

                    let mut out = 2 as $value_type;
                    for third in &[one, two, three, four] {
                        for second in &[one, two, three] {
                            for first in &[one, two] {
                                frame.frame(1, |frame| {
                                    let v =
                                        gi.call(frame, [array, *first, *second, *third])?.unwrap();
                                    assert_eq!(v.try_unbox::<$value_type>()?, out);
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
    };
}

impl_test!(array_data_3d_u8, array_data_3d_u8_mut, u8);
impl_test!(array_data_3d_u16, array_data_3d_u16_mut, u16);
impl_test!(array_data_3d_u32, array_data_3d_u32_mut, u32);
impl_test!(array_data_3d_u64, array_data_3d_u64_mut, u64);
impl_test!(array_data_3d_i8, array_data_3d_i8_mut, i8);
impl_test!(array_data_3d_i16, array_data_3d_i16_mut, i16);
impl_test!(array_data_3d_i32, array_data_3d_i32_mut, i32);
impl_test!(array_data_3d_i64, array_data_3d_i64_mut, i64);
impl_test!(array_data_3d_f32, array_data_3d_f32_mut, f32);
impl_test!(array_data_3d_f64, array_data_3d_f64_mut, f64);

#[test]
fn borrow_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(1, |global, frame| {
            let data: Vec<u8> = (1..=24).map(|x| x as u8).collect();

            let array = Value::move_array(frame, data, (2, 3, 4))?;

            frame.frame(4, |frame| {
                let d = array.array_data::<u8, _>(frame)?;

                let mut out = 1 as u8;
                for third in &[0, 1, 2, 3] {
                    for second in &[0, 1, 2] {
                        for first in &[0, 1] {
                            assert_eq!(d[(*first, *second, *third)], out);
                            out += 1 as u8;
                        }
                    }
                }

                let gi = Module::base(global).function("getindex")?;
                let one = Value::new(frame, 1usize)?;
                let two = Value::new(frame, 2usize)?;
                let three = Value::new(frame, 3usize)?;
                let four = Value::new(frame, 4usize)?;

                out = 1 as u8;
                for third in &[one, two, three, four] {
                    for second in &[one, two, three] {
                        for first in &[one, two] {
                            frame.frame(1, |frame| {
                                let v = gi.call(frame, [array, *first, *second, *third])?.unwrap();
                                assert_eq!(v.try_unbox::<u8>()?, out);
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
