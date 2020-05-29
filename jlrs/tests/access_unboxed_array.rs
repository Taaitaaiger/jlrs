use jlrs::prelude::*;
use jlrs::util::JULIA;

macro_rules! impl_test {
    ($name:ident, $name_mut:ident, $value_type:ty) => {
        #[test]
        fn $name() {
            JULIA.with(|j| {
                let mut jlrs = j.borrow_mut();

                jlrs.frame(1, |_, frame| {
                    let data: Vec<$value_type> = (1..=24).map(|x| x as $value_type).collect();

                    let array = Value::move_array(frame, data, (2, 3, 4))?;
                    let d = array.cast::<Array>()?.copy_inline_data::<$value_type>()?;

                    let mut out = 1 as $value_type;
                    for third in &[0, 1, 2, 3] {
                        for second in &[0, 1, 2] {
                            for first in &[0, 1] {
                                assert_eq!(d[(*first, *second, *third)], out);
                                assert_eq!(*d.get((*first, *second, *third)).unwrap(), out);
                                out += 1 as $value_type;
                            }
                        }
                    }

                    assert!(d.get((7, 7, 7)).is_none());

                    Ok(())
                })
                .unwrap();
            });
        }

        #[test]
        fn $name_mut() {
            JULIA.with(|j| {
                let mut jlrs = j.borrow_mut();

                jlrs.frame(1, |_, frame| {
                    let data: Vec<$value_type> = (1..=24).map(|x| x as $value_type).collect();

                    let array = Value::move_array(frame, data, (2, 3, 4))?;
                    let mut d = array.cast::<Array>()?.copy_inline_data::<$value_type>()?;

                    let mut out = 2 as $value_type;
                    for third in &[0, 1, 2, 3] {
                        for second in &[0, 1, 2] {
                            for first in &[0, 1] {
                                d[(*first, *second, *third)] += 1 as $value_type;
                                assert_eq!(d[(*first, *second, *third)], out);
                                let e = d.get_mut((*first, *second, *third)).unwrap();
                                *e = *e + 1 as $value_type;
                                assert_eq!(d[(*first, *second, *third)], out + 1 as $value_type);
                                out += 1 as $value_type;
                            }
                        }
                    }

                    assert!(d.get_mut((7, 7, 7)).is_none());

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
