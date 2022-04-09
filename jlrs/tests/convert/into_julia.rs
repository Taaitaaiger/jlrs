#[cfg(feature = "sync-rt")]
mod tests {
    use super::super::super::util::JULIA;
    use jlrs::{convert::into_julia::IntoJulia, prelude::*, wrappers::ptr::union_all::UnionAll};
    use std::{ffi::c_void, ptr::null_mut};

    macro_rules! impl_test {
        ($type:ty, $type_test_name:ident, $into_test_name:ident, $val:expr, $assoc_ty:ident) => {
            #[test]
            fn $type_test_name() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();

                    jlrs.scope(|global, _frame| unsafe {
                        let ty = <$type as IntoJulia>::julia_type(global).value_unchecked();
                        assert_eq!(ty, DataType::$assoc_ty(global).as_value());
                        assert!(ty.cast::<DataType>()?.is::<$type>());

                        Ok(())
                    })
                    .unwrap();
                });
            }

            #[test]
            fn $into_test_name() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();

                    jlrs.scope(|global, _frame| unsafe {
                        let val = $val.into_julia(global).value_unchecked();
                        assert!(val.is::<$type>());

                        Ok(())
                    })
                    .unwrap();
                });
            }
        };
    }

    macro_rules! impl_ptr_test {
        ($type:ty, $type_test_name:ident, $into_test_name:ident, $assoc_ty:ident) => {
            #[test]
            fn $type_test_name() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();

                    jlrs.scope(|global, frame| unsafe {
                        let ty = <*mut $type as IntoJulia>::julia_type(global).value_unchecked();

                        let applied = UnionAll::pointer_type(global)
                            .as_value()
                            .apply_type_unchecked(&mut *frame, &mut [DataType::$assoc_ty(global).as_value()])?;

                        assert_eq!(ty, applied);
                        assert!(applied.cast::<DataType>()?.is::<*mut $type>());

                        Ok(())
                    })
                    .unwrap();
                });
            }

            #[test]
            fn $into_test_name() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();

                    jlrs.scope(|global, _frame| unsafe {
                        let val = null_mut::<$type>().into_julia(global).value_unchecked();
                        assert!(val.is::<*mut $type>());
                        Ok(())
                    })
                    .unwrap();
                });
            }
        };
    }

    impl_test!(bool, bool_julia_type, bool_into_julia, true, bool_type);
    impl_test!(char, char_julia_type, char_into_julia, 'a', char_type);

    impl_test!(u8, u8_julia_type, u8_into_julia, 1u8, uint8_type);
    impl_test!(u16, u16_julia_type, u16_into_julia, 1u16, uint16_type);
    impl_test!(u32, u32_julia_type, u32_into_julia, 1u32, uint32_type);
    impl_test!(u64, u64_julia_type, u64_into_julia, 1u64, uint64_type);

    #[cfg(target_pointer_width = "64")]
    impl_test!(
        usize,
        usize_julia_type,
        usize_into_julia,
        1usize,
        uint64_type
    );
    #[cfg(target_pointer_width = "32")]
    impl_test!(
        usize,
        usize_julia_type,
        usize_into_julia,
        1usize,
        uint32_type
    );

    impl_test!(i8, i8_julia_type, i8_into_julia, 1i8, int8_type);
    impl_test!(i16, i16_julia_type, i16_into_julia, 1i16, int16_type);
    impl_test!(i32, i32_julia_type, i32_into_julia, 1i32, int32_type);
    impl_test!(i64, i64_julia_type, i64_into_julia, 1i64, int64_type);

    #[cfg(target_pointer_width = "64")]
    impl_test!(
        isize,
        isize_julia_type,
        isize_into_julia,
        1isize,
        int64_type
    );
    #[cfg(target_pointer_width = "32")]
    impl_test!(
        isize,
        isize_julia_type,
        isize_into_julia,
        1isize,
        int32_type
    );

    impl_test!(f32, f32_julia_type, f32_into_julia, 1.0f32, float32_type);
    impl_test!(f64, f64_julia_type, f64_into_julia, 1.0f64, float64_type);

    impl_ptr_test!(bool, bool_ptr_julia_type, bool_ptr_into_julia, bool_type);
    impl_ptr_test!(char, char_ptr_julia_type, char_ptr_into_julia, char_type);

    impl_ptr_test!(u8, u8_ptr_julia_type, u8_ptr_into_julia, uint8_type);
    impl_ptr_test!(u16, u16_ptr_julia_type, u16_ptr_into_julia, uint16_type);
    impl_ptr_test!(u32, u32_ptr_julia_type, u32_ptr_into_julia, uint32_type);
    impl_ptr_test!(u64, u64_ptr_julia_type, u64_ptr_into_julia, uint64_type);

    #[cfg(target_pointer_width = "64")]
    impl_ptr_test!(
        usize,
        usize_ptr_julia_type,
        usize_ptr_into_julia,
        uint64_type
    );
    #[cfg(target_pointer_width = "32")]
    impl_ptr_test!(
        usize,
        usize_ptr_julia_type,
        usize_ptr_into_julia,
        uint32_type
    );

    impl_ptr_test!(i8, i8_ptr_julia_type, i8_ptr_into_julia, int8_type);
    impl_ptr_test!(i16, i16_ptr_julia_type, i16_ptr_into_julia, int16_type);
    impl_ptr_test!(i32, i32_ptr_julia_type, i32_ptr_into_julia, int32_type);
    impl_ptr_test!(i64, i64_ptr_julia_type, i64_ptr_into_julia, int64_type);

    #[cfg(target_pointer_width = "64")]
    impl_ptr_test!(
        isize,
        isize_ptr_julia_type,
        isize_ptr_into_julia,
        int64_type
    );
    #[cfg(target_pointer_width = "32")]
    impl_ptr_test!(
        isize,
        isize_ptr_julia_type,
        isize_ptr_into_julia,
        int32_type
    );

    impl_ptr_test!(f32, f32_ptr_julia_type, f32_ptr_into_julia, float32_type);
    impl_ptr_test!(f64, f64_ptr_julia_type, f64_ptr_into_julia, float64_type);

    #[test]
    fn void_ptr_julia_type() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, _frame| unsafe {
                let ty = <*mut c_void as IntoJulia>::julia_type(global).value_unchecked();
                assert_eq!(ty, DataType::voidpointer_type(global).as_value());
                assert!(ty.cast::<DataType>()?.is::<*mut c_void>());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn void_ptr_into_julia() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, _frame| unsafe {
                let val = null_mut::<c_void>().into_julia(global).value_unchecked();
                assert!(val.is::<*mut c_void>());
                Ok(())
            })
            .unwrap();
        });
    }
}
