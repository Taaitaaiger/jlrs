use jlrs::prelude::*;
use jlrs::util::JULIA;

macro_rules! impl_test {
    ($name:ident, $name_mut:ident, $name_slice:ident, $name_slice_mut:ident, $value_type:ty) => {
        #[test]
        fn $name() {
            JULIA.with(|j| {
                let mut jlrs = j.borrow_mut();

                jlrs.frame_with_slots(1, |_, frame| {
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

                jlrs.frame_with_slots(1, |_, frame| {
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

        #[test]
        fn $name_slice() {
            JULIA.with(|j| {
                let mut jlrs = j.borrow_mut();

                jlrs.frame_with_slots(1, |_, frame| {
                    let data: Vec<$value_type> = (1..=24).map(|x| x as $value_type).collect();

                    let array = Value::move_array(frame, data.clone(), (2, 3, 4))?;
                    let d = array.cast::<Array>()?.copy_inline_data::<$value_type>()?;

                    for (a, b) in data.iter().zip(d.as_slice()) {
                        assert_eq!(a, b)
                    }

                    Ok(())
                })
                .unwrap();
            });
        }

        #[test]
        fn $name_slice_mut() {
            JULIA.with(|j| {
                let mut jlrs = j.borrow_mut();

                jlrs.frame_with_slots(1, |_, frame| {
                    let data: Vec<$value_type> = (1..=24).map(|x| x as $value_type).collect();

                    let array = Value::move_array(frame, data.clone(), (2, 3, 4))?;
                    let mut d = array.cast::<Array>()?.copy_inline_data::<$value_type>()?;

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
    array_data_3d_u8_slice_mut,
    u8
);
impl_test!(
    array_data_3d_u16,
    array_data_3d_u16_mut,
    array_data_3d_u16_slice,
    array_data_3d_u16_slice_mut,
    u16
);
impl_test!(
    array_data_3d_u32,
    array_data_3d_u32_mut,
    array_data_3d_u32_slice,
    array_data_3d_u32_slice_mut,
    u32
);
impl_test!(
    array_data_3d_u64,
    array_data_3d_u64_mut,
    array_data_3d_u64_slice,
    array_data_3d_u64_slice_mut,
    u64
);
impl_test!(
    array_data_3d_i8,
    array_data_3d_i8_mut,
    array_data_3d_i8_slice,
    array_data_3d_i8_slice_mut,
    i8
);
impl_test!(
    array_data_3d_i16,
    array_data_3d_i16_mut,
    array_data_3d_i16_slice,
    array_data_3d_i16_slice_mut,
    i16
);
impl_test!(
    array_data_3d_i32,
    array_data_3d_i32_mut,
    array_data_3d_i32_slice,
    array_data_3d_i32_slice_mut,
    i32
);
impl_test!(
    array_data_3d_i64,
    array_data_3d_i64_mut,
    array_data_3d_i64_slice,
    array_data_3d_i64_slice_mut,
    i64
);
impl_test!(
    array_data_3d_f32,
    array_data_3d_f32_mut,
    array_data_3d_f32_slice,
    array_data_3d_f32_slice_mut,
    f32
);
impl_test!(
    array_data_3d_f64,
    array_data_3d_f64_mut,
    array_data_3d_f64_slice,
    array_data_3d_f64_slice_mut,
    f64
);

#[test]
fn access_copied_array_dimensions() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame_with_slots(1, |_, frame| {
            let arr_val = Value::new_array::<f32, _, _, _>(frame, (1, 2))?;
            let arr = arr_val.cast::<Array>()?;

            let data = arr.copy_inline_data::<f32>()?;
            assert_eq!(data.dimensions().as_slice(), &[1, 2]);

            Ok(())
        })
        .unwrap();
    })
}
