#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use crate::util::JULIA;
    use jlrs::{prelude::*, wrappers::ptr::array::dimensions::Dims};

    #[test]
    fn move_array_1d() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope_with_capacity(1, |_, frame| {
                    let data = vec![1.0f32, 2., 3.];
                    let array = Array::from_vec(&mut *frame, data, 3)?.into_jlrs_result()?;
                    array.copy_inline_data::<f32, _>(frame)
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1., 2., 3.]);
        });
    }

    #[test]
    fn move_array_1d_output() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope_with_capacity(1, |_, frame| {
                    let (output, frame) = frame.split()?;
                    let data = vec![1.0f32, 2., 3.];
                    let array = frame
                        .scope_with_capacity(0, |frame| {
                            let output = output.into_scope(frame);
                            Array::from_vec(output, data, 3)
                        })?
                        .into_jlrs_result()?;
                    array.copy_inline_data::<f32, _>(frame)
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1., 2., 3.]);
        });
    }

    #[test]
    fn move_array_1d_nested() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope_with_capacity(0, |_, frame| {
                    frame.scope_with_capacity(1, |frame| {
                        let data = vec![1.0f64, 2., 3.];
                        let array = Array::from_vec(&mut *frame, data, 3)?.into_jlrs_result()?;
                        array.copy_inline_data::<f64, _>(frame)
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1., 2., 3.]);
        });
    }

    #[test]
    fn move_array_1d_nested_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope_with_capacity(0, |_, frame| {
                    frame.scope(|frame| {
                        let data = vec![1i8, 2, 3];
                        let array = Array::from_vec(&mut *frame, data, 3)?.into_jlrs_result()?;
                        array.copy_inline_data::<i8, _>(frame)
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1, 2, 3]);
        });
    }

    #[test]
    fn move_array_1d_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope(|_, frame| {
                    let data = vec![1i16, 2, 3];
                    let array = Array::from_vec(&mut *frame, data, 3)?.into_jlrs_result()?;
                    array.copy_inline_data::<i16, _>(frame)
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1, 2, 3]);
        });
    }

    #[test]
    fn move_array_1d_dynamic_nested() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope(|_, frame| {
                    frame.scope_with_capacity(1, |frame| {
                        let data = vec![1i32, 2, 3];
                        let array = Array::from_vec(&mut *frame, data, 3)?.into_jlrs_result()?;
                        array.copy_inline_data::<i32, _>(frame)
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1, 2, 3]);
        });
    }

    #[test]
    fn move_array_1d_dynamic_nested_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope(|_, frame| {
                    frame.scope(|frame| {
                        let data = vec![1i64, 2, 3];
                        let array = Array::from_vec(&mut *frame, data, 3)?.into_jlrs_result()?;
                        array.copy_inline_data::<i64, _>(frame)
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1, 2, 3]);
        });
    }

    #[test]
    fn move_array_2d() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope_with_capacity(1, |_, frame| {
                    let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                    let array = Array::from_vec(&mut *frame, data, (3, 4))?.into_jlrs_result()?;
                    array.copy_inline_data::<u8, _>(frame)
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.to_vec(), vec![1u8, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
        });
    }

    #[test]
    fn move_array_2d_nested() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope_with_capacity(0, |_, frame| {
                    frame.scope_with_capacity(1, |frame| {
                        let data = vec![1u16, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                        let array =
                            Array::from_vec(&mut *frame, data, (3, 4))?.into_jlrs_result()?;
                        array.copy_inline_data::<u16, _>(frame)
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.to_vec(), vec![1u16, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
        });
    }

    #[test]
    fn move_array_2d_nested_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope_with_capacity(0, |_, frame| {
                    frame.scope(|frame| {
                        let data = vec![1u32, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                        let array =
                            Array::from_vec(&mut *frame, data, (3, 4))?.into_jlrs_result()?;
                        array.copy_inline_data::<u32, _>(frame)
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.to_vec(), vec![1u32, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
        });
    }

    #[test]
    fn move_array_2d_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope(|_, frame| {
                    let data = vec![1u64, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                    let array = Array::from_vec(&mut *frame, data, (3, 4))?.into_jlrs_result()?;
                    array.copy_inline_data::<u64, _>(frame)
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.to_vec(), vec![1u64, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
        });
    }

    #[test]
    fn move_array_2d_dynamic_nested() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope(|_, frame| {
                    frame.scope_with_capacity(1, |frame| {
                        let data = vec![1usize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                        let array =
                            Array::from_vec(&mut *frame, data, (3, 4))?.into_jlrs_result()?;
                        array.copy_inline_data::<usize, _>(frame)
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.to_vec(), vec![1usize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
        });
    }

    #[test]
    fn move_array_2d_dynamic_nested_dynamic() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .scope(|_, frame| {
                    frame.scope(|frame| {
                        let data = vec![1isize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                        let array =
                            Array::from_vec(&mut *frame, data, (3, 4))?.into_jlrs_result()?;
                        array.copy_inline_data::<isize, _>(frame)
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.to_vec(), vec![1isize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
        });
    }
}
