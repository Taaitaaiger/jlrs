mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use std::ptr::null_mut;

    use jlrs::{
        data::layout::valid_layout::{ValidField, ValidLayout},
        prelude::*,
    };

    use super::util::JULIA;

    macro_rules! impl_valid_layout_test {
        ($name:ident, $invalid_name:ident, $t:ty, $val:expr_2021) => {
            fn $name() {
                JULIA.with(|handle| {
                    handle.borrow_mut().with_stack(|mut stack| {
                        stack.scope(|mut frame| {
                            let val: $t = $val;
                            let v = Value::new(&mut frame, val);
                            assert!(<$t>::valid_layout(v.datatype().as_value()));
                        });
                    })
                })
            }

            fn $invalid_name() {
                JULIA.with(|handle| {
                    handle.borrow_mut().with_stack(|mut stack| {
                        stack.scope(|mut frame| {
                            let v = Value::new(&mut frame, null_mut::<$t>());
                            assert!(!<$t>::valid_layout(v.datatype().as_value()));
                        });
                    })
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

    fn invalid_ptr_layout() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let v = Value::new(&mut frame, null_mut::<u8>());
                    assert!(!<u8>::valid_layout(v.datatype().as_value()));
                });
            })
        })
    }

    fn valid_layout_array() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let v = TypedArray::<i32>::new::<_, _>(&mut frame, [2, 2])
                        .unwrap()
                        .as_value();
                    assert!(Option::<WeakArray>::valid_field(v.datatype().as_value()));

                    let ua = Module::base(&frame)
                        .global(&frame, "Array")
                        .unwrap()
                        .as_managed();

                    assert!(Option::<WeakArray>::valid_field(ua));
                });
            })
        })
    }

    fn invalid_layout_array() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let v = TypedArray::<i32>::new::<_, _>(&mut frame, [2, 2])
                        .unwrap()
                        .as_value();

                    assert!(Option::<WeakTypedArray<i32>>::valid_field(
                        v.datatype().as_value()
                    ));
                    assert!(!Option::<WeakTypedArray<u32>>::valid_field(
                        v.datatype().as_value()
                    ));
                });
            })
        })
    }

    #[test]
    fn valid_layout_tests() {
        valid_layout_u8();
        invalid_layout_u8();
        valid_layout_u16();
        invalid_layout_u16();
        valid_layout_u32();
        invalid_layout_u32();
        valid_layout_u64();
        invalid_layout_u64();
        valid_layout_usize();
        invalid_layout_usize();
        valid_layout_i8();
        invalid_layout_i8();
        valid_layout_i16();
        invalid_layout_i16();
        valid_layout_i32();
        invalid_layout_i32();
        valid_layout_i64();
        invalid_layout_i64();
        valid_layout_isize();
        invalid_layout_isize();
        valid_layout_f32();
        invalid_layout_f32();
        valid_layout_f64();
        invalid_layout_f64();
        valid_layout_bool();
        invalid_layout_bool();
        valid_layout_char();
        invalid_layout_char();

        valid_layout_u8_ptr();
        invalid_layout_u8_ptr();
        valid_layout_u16_ptr();
        invalid_layout_u16_ptr();
        valid_layout_u32_ptr();
        invalid_layout_u32_ptr();
        valid_layout_u64_ptr();
        invalid_layout_u64_ptr();
        valid_layout_usize_ptr();
        invalid_layout_usize_ptr();
        valid_layout_i8_ptr();
        invalid_layout_i8_ptr();
        valid_layout_i16_ptr();
        invalid_layout_i16_ptr();
        valid_layout_i32_ptr();
        invalid_layout_i32_ptr();
        valid_layout_i64_ptr();
        invalid_layout_i64_ptr();
        valid_layout_isize_ptr();
        invalid_layout_isize_ptr();
        valid_layout_f32_ptr();
        invalid_layout_f32_ptr();
        valid_layout_f64_ptr();
        invalid_layout_f64_ptr();
        valid_layout_bool_ptr();
        invalid_layout_bool_ptr();
        valid_layout_char_ptr();
        invalid_layout_char_ptr();

        invalid_ptr_layout();
        valid_layout_array();
        invalid_layout_array();
    }
}
