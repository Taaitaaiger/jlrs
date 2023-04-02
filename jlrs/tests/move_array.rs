mod util;

#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
mod tests {
    use jlrs::{data::managed::array::dimensions::Dims, prelude::*};

    use crate::util::JULIA;

    fn move_array_1d() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1.0f32, 2., 3.];
                    let array =
                        Array::from_vec(frame.as_extended_target(), data, 3)?.into_jlrs_result()?;
                    unsafe { array.copy_inline_data::<f32>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1., 2., 3.]);
        });
    }

    fn move_array_1d_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let output = frame.output();
                    let data = vec![1.0f32, 2., 3.];
                    let array = frame
                        .scope(|mut frame| {
                            let output = output.into_extended_target(&mut frame);
                            Array::from_vec(output, data, 3)
                        })?
                        .into_jlrs_result()?;
                    unsafe { array.copy_inline_data::<f32>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1., 2., 3.]);
        });
    }

    fn move_array_1d_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let data = vec![1.0f64, 2., 3.];
                        let array = Array::from_vec(frame.as_extended_target(), data, 3)?
                            .into_jlrs_result()?;
                        unsafe { array.copy_inline_data::<f64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1., 2., 3.]);
        });
    }

    fn move_array_1d_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let data = vec![1i8, 2, 3];
                        let array = Array::from_vec(frame.as_extended_target(), data, 3)?
                            .into_jlrs_result()?;
                        unsafe { array.copy_inline_data::<i8>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1, 2, 3]);
        });
    }

    fn move_array_1d_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1i16, 2, 3];
                    let array =
                        Array::from_vec(frame.as_extended_target(), data, 3)?.into_jlrs_result()?;
                    unsafe { array.copy_inline_data::<i16>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1, 2, 3]);
        });
    }

    fn move_array_1d_dynamic_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let data = vec![1i32, 2, 3];
                        let array = Array::from_vec(frame.as_extended_target(), data, 3)?
                            .into_jlrs_result()?;
                        unsafe { array.copy_inline_data::<i32>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1, 2, 3]);
        });
    }

    fn move_array_1d_dynamic_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let data = vec![1i64, 2, 3];
                        let array = Array::from_vec(frame.as_extended_target(), data, 3)?
                            .into_jlrs_result()?;
                        unsafe { array.copy_inline_data::<i64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.to_vec(), vec![1, 2, 3]);
        });
    }

    fn move_array_2d() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                    let array = Array::from_vec(frame.as_extended_target(), data, (3, 4))?
                        .into_jlrs_result()?;
                    unsafe { array.copy_inline_data::<u8>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.to_vec(), vec![1u8, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
        });
    }

    fn move_array_2d_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let data = vec![1u16, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                        let array = Array::from_vec(frame.as_extended_target(), data, (3, 4))?
                            .into_jlrs_result()?;
                        unsafe { array.copy_inline_data::<u16>() }
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

    fn move_array_2d_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let data = vec![1u32, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                        let array = Array::from_vec(frame.as_extended_target(), data, (3, 4))?
                            .into_jlrs_result()?;
                        unsafe { array.copy_inline_data::<u32>() }
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

    fn move_array_2d_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1u64, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                    let array = Array::from_vec(frame.as_extended_target(), data, (3, 4))?
                        .into_jlrs_result()?;
                    unsafe { array.copy_inline_data::<u64>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.to_vec(), vec![1u64, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
        });
    }

    fn move_array_2d_dynamic_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let data = vec![1usize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                        let array = Array::from_vec(frame.as_extended_target(), data, (3, 4))?
                            .into_jlrs_result()?;
                        unsafe { array.copy_inline_data::<usize>() }
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

    fn move_array_2d_dynamic_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let data = vec![1isize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                        let array = Array::from_vec(frame.as_extended_target(), data, (3, 4))?
                            .into_jlrs_result()?;
                        unsafe { array.copy_inline_data::<isize>() }
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

    #[test]
    fn move_array_tests() {
        move_array_1d();
        move_array_1d_output();
        move_array_1d_nested();
        move_array_1d_nested_dynamic();
        move_array_1d_dynamic();
        move_array_1d_dynamic_nested();
        move_array_1d_dynamic_nested_dynamic();
        move_array_2d();
        move_array_2d_nested();
        move_array_2d_nested_dynamic();
        move_array_2d_dynamic();
        move_array_2d_dynamic_nested();
        move_array_2d_dynamic_nested_dynamic();
    }
}
