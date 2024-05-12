mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use std::{ffi::c_void, ptr::null_mut};

    use jlrs::{
        convert::{into_julia::IntoJulia, unbox::Unbox},
        prelude::*,
    };

    use super::util::JULIA;

    macro_rules! impl_test {
        ($type:ty, $test_name:ident, $failing_test_name:ident, $val:expr) => {
            fn $test_name() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();

                    jlrs.instance(&mut frame)
                        .returning::<JlrsResult<_>>()
                        .scope(|frame| unsafe {
                            let val: $type = $val;
                            assert_eq!(
                                <$type as Unbox>::unbox(val.into_julia(&frame).as_value()),
                                $val
                            );
                            Ok(())
                        })
                        .unwrap();
                });
            }
        };
    }

    impl_test!(u8, unbox_u8, unbox_u8_as_bool, 0u8);
    impl_test!(u16, unbox_u16, unbox_u16_as_bool, 0u16);
    impl_test!(u32, unbox_u32, unbox_u32_as_bool, 0u32);
    impl_test!(u64, unbox_u64, unbox_u64_as_bool, 0u64);

    #[cfg(target_pointer_width = "64")]
    impl_test!(usize, unbox_usize, unbox_usize_as_bool, 0usize);
    #[cfg(target_pointer_width = "32")]
    impl_test!(usize, unbox_usize, unbox_usize_as_bool, 0usize);

    impl_test!(i8, unbox_i8, unbox_i8_as_bool, 0i8);
    impl_test!(i16, unbox_i16, unbox_i16_as_bool, 0i16);
    impl_test!(i32, unbox_i32, unbox_i32_as_bool, 0i32);
    impl_test!(i64, unbox_i64, unbox_i64_as_bool, 0i64);

    #[cfg(target_pointer_width = "64")]
    impl_test!(isize, unbox_isize, unbox_isize_as_bool, 0isize);
    #[cfg(target_pointer_width = "32")]
    impl_test!(isize, unbox_isize, unbox_isize_as_bool, 0isize);

    impl_test!(f32, unbox_f32, unbox_f32_as_bool, 0.0f32);
    impl_test!(f64, unbox_f64, unbox_f64_as_bool, 0.0f64);

    impl_test!(*mut u8, unbox_u8_ptr, unbox_u8_ptr_as_bool, null_mut());
    impl_test!(*mut u16, unbox_u16_ptr, unbox_u16_ptr_as_bool, null_mut());
    impl_test!(*mut u32, unbox_u32_ptr, unbox_u32_ptr_as_bool, null_mut());
    impl_test!(*mut u64, unbox_u64_ptr, unbox_u64_ptr_as_bool, null_mut());

    #[cfg(target_pointer_width = "64")]
    impl_test!(
        *mut usize,
        unbox_usize_ptr,
        unbox_usize_ptr_as_bool,
        null_mut()
    );
    #[cfg(target_pointer_width = "32")]
    impl_test!(
        *mut usize,
        unbox_usize_ptr,
        unbox_usize_ptr_as_bool,
        null_mut()
    );

    impl_test!(*mut i8, unbox_i8_ptr, unbox_i8_ptr_as_bool, null_mut());
    impl_test!(*mut i16, unbox_i16_ptr, unbox_i16_ptr_as_bool, null_mut());
    impl_test!(*mut i32, unbox_i32_ptr, unbox_i32_ptr_as_bool, null_mut());
    impl_test!(*mut i64, unbox_i64_ptr, unbox_i64_ptr_as_bool, null_mut());

    #[cfg(target_pointer_width = "64")]
    impl_test!(
        *mut isize,
        unbox_isize_ptr,
        unbox_isize_ptr_as_bool,
        null_mut()
    );
    #[cfg(target_pointer_width = "32")]
    impl_test!(
        *mut isize,
        unbox_isize_ptr,
        unbox_isize_ptr_as_bool,
        null_mut()
    );

    impl_test!(*mut f32, unbox_f32_ptr, unbox_f32_ptr_as_bool, null_mut());
    impl_test!(*mut f64, unbox_f64_ptr, unbox_f64_ptr_as_bool, null_mut());

    impl_test!(
        *mut c_void,
        unbox_void_ptr,
        unbox_void_ptr_as_bool,
        null_mut()
    );

    #[test]
    fn unbox_tests() {
        unbox_u8();
        unbox_u16();
        unbox_u32();
        unbox_u64();
        unbox_usize();
        unbox_i8();
        unbox_i16();
        unbox_i32();
        unbox_i64();
        unbox_isize();
        unbox_f32();
        unbox_f64();
        unbox_u8_ptr();
        unbox_u16_ptr();
        unbox_u32_ptr();
        unbox_u64_ptr();
        unbox_usize_ptr();
        unbox_usize_ptr();
        unbox_i8_ptr();
        unbox_i16_ptr();
        unbox_i32_ptr();
        unbox_i64_ptr();
        unbox_isize_ptr();
        unbox_f32_ptr();
        unbox_f64_ptr();
        unbox_void_ptr();
    }
}
