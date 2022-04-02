#[cfg(feature = "sync-rt")]
mod tests {
    use super::super::super::util::JULIA;
    use jlrs::layout::valid_layout::ValidLayout;
    use jlrs::prelude::*;
    use std::ptr::null_mut;

    macro_rules! impl_valid_layout_test {
        ($name:ident, $invalid_name:ident, $t:ty, $val:expr) => {
            #[test]
            fn $name() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();
                    jlrs.scope_with_capacity(1, |_global, frame| {
                        let val: $t = $val;
                        let v = Value::new(frame, val)?;
                        assert!(<$t>::valid_layout(v.datatype().as_value()));
                        Ok(())
                    })
                    .unwrap();
                })
            }

            #[test]
            fn $invalid_name() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();
                    jlrs.scope_with_capacity(1, |_global, frame| {
                        let v = Value::new(frame, null_mut::<$t>())?;
                        assert!(!<$t>::valid_layout(v.datatype().as_value()));
                        Ok(())
                    })
                    .unwrap();
                })
            }
        };
    }

    impl_valid_layout_test!(valid_layout_u8, invalid_layout_u8, u8, 1u8);
    impl_valid_layout_test!(valid_layout_u16, invalid_layout_u16, u16, 1u16);
    impl_valid_layout_test!(valid_layout_u32, invalid_layout_u32, u32, 1u32);
    impl_valid_layout_test!(valid_layout_u64, invalid_layout_u64, u64, 1u64);
    impl_valid_layout_test!(valid_layout_usize, invalid_layout_usize, usize, 1usize);
    impl_valid_layout_test!(valid_layout_i8, invalid_layout_i8, i8, 1i8);
    impl_valid_layout_test!(valid_layout_i16, invalid_layout_i16, i16, 1i16);
    impl_valid_layout_test!(valid_layout_i32, invalid_layout_i32, i32, 1i32);
    impl_valid_layout_test!(valid_layout_i64, invalid_layout_i64, i64, 1i64);
    impl_valid_layout_test!(valid_layout_isize, invalid_layout_isize, isize, 1isize);
    impl_valid_layout_test!(valid_layout_f32, invalid_layout_f32, f32, 1.0f32);
    impl_valid_layout_test!(valid_layout_f64, invalid_layout_f64, f64, 1.0f64);
    impl_valid_layout_test!(valid_layout_bool, invalid_layout_bool, bool, true);
    impl_valid_layout_test!(valid_layout_char, invalid_layout_char, char, 'a');

    impl_valid_layout_test!(
        valid_layout_u8_ptr,
        invalid_layout_u8_ptr,
        *mut u8,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_u16_ptr,
        invalid_layout_u16_ptr,
        *mut u16,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_u32_ptr,
        invalid_layout_u32_ptr,
        *mut u32,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_u64_ptr,
        invalid_layout_u64_ptr,
        *mut u64,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_usize_ptr,
        invalid_layout_usize_ptr,
        *mut usize,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_i8_ptr,
        invalid_layout_i8_ptr,
        *mut i8,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_i16_ptr,
        invalid_layout_i16_ptr,
        *mut i16,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_i32_ptr,
        invalid_layout_i32_ptr,
        *mut i32,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_i64_ptr,
        invalid_layout_i64_ptr,
        *mut i64,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_isize_ptr,
        invalid_layout_isize_ptr,
        *mut isize,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_f32_ptr,
        invalid_layout_f32_ptr,
        *mut f32,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_f64_ptr,
        invalid_layout_f64_ptr,
        *mut f64,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_bool_ptr,
        invalid_layout_bool_ptr,
        *mut bool,
        null_mut()
    );
    impl_valid_layout_test!(
        valid_layout_char_ptr,
        invalid_layout_char_ptr,
        *mut char,
        null_mut()
    );

    #[test]
    fn invalid_ptr_layout() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_capacity(1, |_global, frame| {
                let v = Value::new(frame, null_mut::<u8>())?;
                assert!(!<u8>::valid_layout(v.datatype().as_value()));
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    fn valid_layout_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                unsafe {
                    let v = Array::new::<i32, _, _, _>(frame, (2, 2))?
                        .into_jlrs_result()?
                        .as_value();
                    assert!(Array::valid_layout(v.datatype().as_value()));

                    let ua = Module::base(global)
                        .global_ref("Array")?
                        .wrapper_unchecked();

                    assert!(Array::valid_layout(ua));
                }
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    fn invalid_layout_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_global, frame| {
                let v = Array::new::<i32, _, _, _>(frame, (2, 2))?
                    .into_jlrs_result()?
                    .as_value();
                assert!(!bool::valid_layout(v));
                Ok(())
            })
            .unwrap();
        })
    }
}
