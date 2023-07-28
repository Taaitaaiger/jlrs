mod util;

#[cfg(feature = "sync-rt")]
mod tests {
    use jlrs::{data::managed::array::dimensions::Dims, prelude::*};

    use crate::util::JULIA;

    fn array_1d() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let new_array = Array::new::<f32, _, _>(&mut frame, 3).into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<f32>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let output = frame.output();
                    let array = frame
                        .scope(|_frame| Ok(Array::new::<f32, _, _>(output, 3)))?
                        .into_jlrs_result()?;
                    unsafe { array.copy_inline_data::<f32>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<f64, _, _>(&mut frame, 3).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<f64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array = Array::new::<i8, _, _>(&mut frame, 3).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<i8>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let new_array = Array::new::<i16, _, _>(&mut frame, 3).into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<i16>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<i32, _, _>(&mut frame, 3).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<i32>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<i64, _, _>(&mut frame, 3).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<i64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_2d() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let new_array =
                        Array::new::<u8, _, _>(&mut frame, (3, 4)).into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u8>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<u16, _, _>(&mut frame, (3, 4)).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u16>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<u32, _, _>(&mut frame, (3, 4)).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u32>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let new_array =
                        Array::new::<u64, _, _>(&mut frame, (3, 4)).into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u64>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<usize, _, _>(&mut frame, (3, 4)).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<usize>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<isize, _, _>(&mut frame, (3, 4)).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<isize>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_3d() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let new_array =
                        Array::new::<u8, _, _>(&mut frame, (3, 4, 5)).into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u8>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<u16, _, _>(&mut frame, (3, 4, 5)).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u16>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<u32, _, _>(&mut frame, (3, 4, 5)).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u32>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let new_array =
                        Array::new::<u64, _, _>(&mut frame, (3, 4, 5)).into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u64>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<usize, _, _>(&mut frame, (3, 4, 5)).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<usize>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<isize, _, _>(&mut frame, (3, 4, 5)).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<isize>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_4d() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let new_array =
                        Array::new::<u8, _, _>(&mut frame, (3, 4, 5, 6)).into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u8>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<u16, _, _>(&mut frame, (3, 4, 5, 6)).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u16>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new::<u32, _, _>(&mut frame, (3, 4, 5, 6)).into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u32>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let new_array =
                        Array::new::<u64, _, _>(&mut frame, (3, 4, 5, 6)).into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u64>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic_nested() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array = Array::new::<usize, _, _>(&mut frame, (3, 4, 5, 6))
                            .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<usize>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic_nested_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array = Array::new::<isize, _, _>(&mut frame, (3, 4, 5, 6))
                            .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<isize>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_of_bools() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array = Array::new::<bool, _, _>(&mut frame, (3, 4, 5, 6))
                            .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<bool>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_of_chars() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let new_array = Array::new::<char, _, _>(&mut frame, (3, 4, 5, 6))
                            .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<char>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_1d_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let new_array = { Array::new_unchecked::<f32, _, _>(&mut frame, 3) };
                    {
                        new_array.copy_inline_data::<f32>()
                    }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_output_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let output = frame.output();
                    let array = frame.scope(|_frame| unsafe {
                        Ok(Array::new_unchecked::<f32, _, _>(output, 3))
                    })?;
                    unsafe { array.copy_inline_data::<f32>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_nested_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array = { Array::new_unchecked::<f64, _, _>(&mut frame, 3) };
                        {
                            new_array.copy_inline_data::<f64>()
                        }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_nested_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array = { Array::new_unchecked::<i8, _, _>(&mut frame, 3) };
                        {
                            new_array.copy_inline_data::<i8>()
                        }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let new_array = { Array::new_unchecked::<i16, _, _>(&mut frame, 3) };
                    {
                        new_array.copy_inline_data::<i16>()
                    }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic_nested_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array = { Array::new_unchecked::<i32, _, _>(&mut frame, 3) };
                        {
                            new_array.copy_inline_data::<i32>()
                        }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic_nested_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array = { Array::new_unchecked::<i64, _, _>(&mut frame, 3) };
                        {
                            new_array.copy_inline_data::<i64>()
                        }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_2d_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let new_array = { Array::new_unchecked::<u8, _, _>(&mut frame, (3, 4)) };
                    {
                        new_array.copy_inline_data::<u8>()
                    }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_nested_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array = { Array::new_unchecked::<u16, _, _>(&mut frame, (3, 4)) };
                        {
                            new_array.copy_inline_data::<u16>()
                        }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_nested_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array = { Array::new_unchecked::<u32, _, _>(&mut frame, (3, 4)) };
                        {
                            new_array.copy_inline_data::<u32>()
                        }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let new_array = { Array::new_unchecked::<u64, _, _>(&mut frame, (3, 4)) };
                    {
                        new_array.copy_inline_data::<u64>()
                    }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic_nested_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array = { Array::new_unchecked::<usize, _, _>(&mut frame, (3, 4)) };
                        new_array.copy_inline_data::<usize>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic_nested_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array = { Array::new_unchecked::<isize, _, _>(&mut frame, (3, 4)) };
                        new_array.copy_inline_data::<isize>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_3d_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let new_array = { Array::new_unchecked::<u8, _, _>(&mut frame, (3, 4, 5)) };
                    new_array.copy_inline_data::<u8>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_nested_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array =
                            { Array::new_unchecked::<u16, _, _>(&mut frame, (3, 4, 5)) };
                        new_array.copy_inline_data::<u16>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_nested_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array =
                            { Array::new_unchecked::<u32, _, _>(&mut frame, (3, 4, 5)) };
                        new_array.copy_inline_data::<u32>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let new_array = { Array::new_unchecked::<u64, _, _>(&mut frame, (3, 4, 5)) };
                    new_array.copy_inline_data::<u64>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic_nested_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array =
                            { Array::new_unchecked::<usize, _, _>(&mut frame, (3, 4, 5)) };

                        new_array.copy_inline_data::<usize>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic_nested_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array =
                            { Array::new_unchecked::<isize, _, _>(&mut frame, (3, 4, 5)) };
                        new_array.copy_inline_data::<isize>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_4d_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let new_array = { Array::new_unchecked::<u8, _, _>(&mut frame, (3, 4, 5, 6)) };
                    new_array.copy_inline_data::<u8>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_nested_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array =
                            { Array::new_unchecked::<u16, _, _>(&mut frame, (3, 4, 5, 6)) };
                        new_array.copy_inline_data::<u16>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_nested_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array =
                            { Array::new_unchecked::<u32, _, _>(&mut frame, (3, 4, 5, 6)) };
                        new_array.copy_inline_data::<u32>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let new_array = { Array::new_unchecked::<u64, _, _>(&mut frame, (3, 4, 5, 6)) };
                    new_array.copy_inline_data::<u64>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic_nested_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array =
                            { Array::new_unchecked::<usize, _, _>(&mut frame, (3, 4, 5, 6)) };
                        new_array.copy_inline_data::<usize>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic_nested_dynamic_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array =
                            { Array::new_unchecked::<isize, _, _>(&mut frame, (3, 4, 5, 6)) };
                        new_array.copy_inline_data::<isize>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_of_bools_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array =
                            { Array::new_unchecked::<bool, _, _>(&mut frame, (3, 4, 5, 6)) };
                        new_array.copy_inline_data::<bool>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_of_chars_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let new_array =
                            { Array::new_unchecked::<char, _, _>(&mut frame, (3, 4, 5, 6)) };

                        new_array.copy_inline_data::<char>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_1d_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let new_array =
                        Array::new_for(&mut frame, 3, DataType::float32_type(&global).as_value())
                            .into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<f32>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_output_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let output = frame.output();
                    let array = frame
                        .scope(|_frame| {
                            Ok(Array::new_for(
                                output,
                                3,
                                DataType::float32_type(&global).as_value(),
                            ))
                        })?
                        .into_jlrs_result()?;
                    unsafe { array.copy_inline_data::<f32>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_nested_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let global = frame.unrooted();
                        let new_array = Array::new_for(
                            &mut frame,
                            3,
                            DataType::float64_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<f64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_nested_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new_for(&mut frame, 3, DataType::int8_type(&global).as_value())
                                .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<i8>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let new_array =
                        Array::new_for(&mut frame, 3, DataType::int16_type(&global).as_value())
                            .into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<i16>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic_nested_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new_for(&mut frame, 3, DataType::int32_type(&global).as_value())
                                .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<i32>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic_nested_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| {
                        let new_array =
                            Array::new_for(&mut frame, 3, DataType::int64_type(&global).as_value())
                                .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<i64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_2d_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let new_array = Array::new_for(
                        &mut frame,
                        (3, 4),
                        DataType::uint8_type(&global).as_value(),
                    )
                    .into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u8>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_nested_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let global = frame.unrooted();
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4),
                            DataType::uint16_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u16>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_nested_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| {
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4),
                            DataType::uint32_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u32>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let new_array = Array::new_for(
                        &mut frame,
                        (3, 4),
                        DataType::uint64_type(&global).as_value(),
                    )
                    .into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u64>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic_nested_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let global = frame.unrooted();
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4),
                            DataType::uint64_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic_nested_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let global = frame.unrooted();
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4),
                            DataType::int64_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<i64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_3d_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let new_array = Array::new_for(
                        &mut frame,
                        (3, 4, 5),
                        DataType::uint8_type(&global).as_value(),
                    )
                    .into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u8>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_nested_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let global = frame.unrooted();
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4, 5),
                            DataType::uint16_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u16>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_nested_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| {
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4, 5),
                            DataType::uint32_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u32>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let new_array = Array::new_for(
                        &mut frame,
                        (3, 4, 5),
                        DataType::uint64_type(&global).as_value(),
                    )
                    .into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u64>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic_nested_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| {
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4, 5),
                            DataType::uint64_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic_nested_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let global = frame.unrooted();
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4, 5),
                            DataType::int64_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<i64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_4d_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let new_array = Array::new_for(
                        &mut frame,
                        (3, 4, 5, 6),
                        DataType::uint8_type(&global).as_value(),
                    )
                    .into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u8>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_nested_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| {
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::uint16_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u16>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_nested_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let global = frame.unrooted();
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::uint32_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u32>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let new_array = Array::new_for(
                        &mut frame,
                        (3, 4, 5, 6),
                        DataType::uint64_type(&global).as_value(),
                    )
                    .into_jlrs_result()?;
                    unsafe { new_array.copy_inline_data::<u64>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic_nested_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let global = frame.unrooted();
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::uint64_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<u64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic_nested_dynamic_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let global = frame.unrooted();
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::int64_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<i64>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_of_bools_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let global = frame.unrooted();
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::bool_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<bool>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_of_chars_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| {
                        let global = frame.unrooted();
                        let new_array = Array::new_for(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::char_type(&global).as_value(),
                        )
                        .into_jlrs_result()?;
                        unsafe { new_array.copy_inline_data::<char>() }
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_1d_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let global = frame.unrooted();
                    let new_array = Array::new_for_unchecked(
                        &mut frame,
                        3,
                        DataType::float32_type(&global).as_value(),
                    );
                    new_array.copy_inline_data::<f32>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_output_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let output = frame.output();
                    let array = frame.scope(|_frame| unsafe {
                        Ok(Array::new_for_unchecked(
                            output,
                            3,
                            DataType::float32_type(&global).as_value(),
                        ))
                    })?;
                    unsafe { array.copy_inline_data::<f32>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_nested_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let global = frame.unrooted();
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            3,
                            DataType::float64_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<f64>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_nested_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let global = frame.unrooted();
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            3,
                            DataType::int8_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<i8>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let global = frame.unrooted();
                    let new_array = Array::new_for_unchecked(
                        &mut frame,
                        3,
                        DataType::int16_type(&global).as_value(),
                    );
                    new_array.copy_inline_data::<i16>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic_nested_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let global = frame.unrooted();
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            3,
                            DataType::int32_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<i32>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_1d_dynamic_nested_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| unsafe {
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            3,
                            DataType::int64_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<i64>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(data.len(), 3);
        });
    }

    fn array_2d_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let global = frame.unrooted();
                    let new_array = Array::new_for_unchecked(
                        &mut frame,
                        (3, 4),
                        DataType::uint8_type(&global).as_value(),
                    );
                    new_array.copy_inline_data::<u8>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_nested_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let global = frame.unrooted();
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4),
                            DataType::uint16_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<u16>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_nested_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let global = frame.unrooted();
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4),
                            DataType::uint32_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<u32>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let global = frame.unrooted();
                    let new_array = Array::new_for_unchecked(
                        &mut frame,
                        (3, 4),
                        DataType::uint64_type(&global).as_value(),
                    );
                    new_array.copy_inline_data::<u64>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic_nested_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| unsafe {
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4),
                            DataType::uint64_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<u64>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_2d_dynamic_nested_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| unsafe {
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4),
                            DataType::int64_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<i64>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 2);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(data.len(), 12);
        });
    }

    fn array_3d_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let global = frame.unrooted();
                    let new_array = Array::new_for_unchecked(
                        &mut frame,
                        (3, 4, 5),
                        DataType::uint8_type(&global).as_value(),
                    );
                    new_array.copy_inline_data::<u8>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_nested_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| unsafe {
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4, 5),
                            DataType::uint16_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<u16>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_nested_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let global = frame.unrooted();
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4, 5),
                            DataType::uint32_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<u32>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let global = frame.unrooted();
                    let new_array = Array::new_for_unchecked(
                        &mut frame,
                        (3, 4, 5),
                        DataType::uint64_type(&global).as_value(),
                    );
                    new_array.copy_inline_data::<u64>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic_nested_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let global = frame.unrooted();
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4, 5),
                            DataType::uint64_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<u64>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_3d_dynamic_nested_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let global = frame.unrooted();
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4, 5),
                            DataType::int64_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<i64>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 3);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(data.len(), 60);
        });
    }

    fn array_4d_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let global = frame.unrooted();
                    let new_array = Array::new_for_unchecked(
                        &mut frame,
                        (3, 4, 5, 6),
                        DataType::uint8_type(&global).as_value(),
                    );
                    new_array.copy_inline_data::<u8>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_nested_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| unsafe {
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::uint16_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<u16>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_nested_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| unsafe {
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::uint32_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<u32>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let global = frame.unrooted();
                    let new_array = Array::new_for_unchecked(
                        &mut frame,
                        (3, 4, 5, 6),
                        DataType::uint64_type(&global).as_value(),
                    );
                    new_array.copy_inline_data::<u64>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic_nested_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| unsafe {
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::uint64_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<u64>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_4d_dynamic_nested_dynamic_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| unsafe {
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::int64_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<i64>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_of_bools_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    frame.scope(|mut frame| unsafe {
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::bool_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<bool>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    fn array_of_chars_for_unchecked_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    frame.scope(|mut frame| unsafe {
                        let global = frame.unrooted();
                        let new_array = Array::new_for_unchecked(
                            &mut frame,
                            (3, 4, 5, 6),
                            DataType::char_type(&global).as_value(),
                        );
                        new_array.copy_inline_data::<char>()
                    })
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 4);
            assert_eq!(dims.n_elements(0), 3);
            assert_eq!(dims.n_elements(1), 4);
            assert_eq!(dims.n_elements(2), 5);
            assert_eq!(dims.n_elements(3), 6);
            assert_eq!(data.len(), 360);
        });
    }

    #[test]
    fn managed_array_tests() {
        array_1d();
        array_1d_output();
        array_1d_nested();
        array_1d_nested_dynamic();
        array_1d_dynamic();
        array_1d_dynamic_nested();
        array_1d_dynamic_nested_dynamic();
        array_2d();
        array_2d_nested();
        array_2d_nested_dynamic();
        array_2d_dynamic();
        array_2d_dynamic_nested();
        array_2d_dynamic_nested_dynamic();
        array_3d();
        array_3d_nested();
        array_3d_nested_dynamic();
        array_3d_dynamic();
        array_3d_dynamic_nested();
        array_3d_dynamic_nested_dynamic();
        array_4d();
        array_4d_nested();
        array_4d_nested_dynamic();
        array_4d_dynamic();
        array_4d_dynamic_nested();
        array_4d_dynamic_nested_dynamic();
        array_of_bools();
        array_of_chars();
        array_1d_unchecked();
        array_1d_output_unchecked();
        array_1d_nested_unchecked();
        array_1d_nested_dynamic_unchecked();
        array_1d_dynamic_unchecked();
        array_1d_dynamic_nested_unchecked();
        array_1d_dynamic_nested_dynamic_unchecked();
        array_2d_unchecked();
        array_2d_nested_unchecked();
        array_2d_nested_dynamic_unchecked();
        array_2d_dynamic_unchecked();
        array_2d_dynamic_nested_unchecked();
        array_2d_dynamic_nested_dynamic_unchecked();
        array_3d_unchecked();
        array_3d_nested_unchecked();
        array_3d_nested_dynamic_unchecked();
        array_3d_dynamic_unchecked();
        array_3d_dynamic_nested_unchecked();
        array_3d_dynamic_nested_dynamic_unchecked();
        array_4d_unchecked();
        array_4d_nested_unchecked();
        array_4d_nested_dynamic_unchecked();
        array_4d_dynamic_unchecked();
        array_4d_dynamic_nested_unchecked();
        array_4d_dynamic_nested_dynamic_unchecked();
        array_of_bools_unchecked();
        array_of_chars_unchecked();
        array_1d_for();
        array_1d_output_for();
        array_1d_nested_for();
        array_1d_nested_dynamic_for();
        array_1d_dynamic_for();
        array_1d_dynamic_nested_for();
        array_1d_dynamic_nested_dynamic_for();
        array_2d_for();
        array_2d_nested_for();
        array_2d_nested_dynamic_for();
        array_2d_dynamic_for();
        array_2d_dynamic_nested_for();
        array_2d_dynamic_nested_dynamic_for();
        array_3d_for();
        array_3d_nested_for();
        array_3d_nested_dynamic_for();
        array_3d_dynamic_for();
        array_3d_dynamic_nested_for();
        array_3d_dynamic_nested_dynamic_for();
        array_4d_for();
        array_4d_nested_for();
        array_4d_nested_dynamic_for();
        array_4d_dynamic_for();
        array_4d_dynamic_nested_for();
        array_4d_dynamic_nested_dynamic_for();
        array_of_bools_for();
        array_of_chars_for();
        array_1d_for_unchecked_unchecked();
        array_1d_output_for_unchecked_unchecked();
        array_1d_nested_for_unchecked_unchecked();
        array_1d_nested_dynamic_for_unchecked_unchecked();
        array_1d_dynamic_for_unchecked_unchecked();
        array_1d_dynamic_nested_for_unchecked_unchecked();
        array_1d_dynamic_nested_dynamic_for_unchecked_unchecked();
        array_2d_for_unchecked_unchecked();
        array_2d_nested_for_unchecked_unchecked();
        array_2d_nested_dynamic_for_unchecked_unchecked();
        array_2d_dynamic_for_unchecked_unchecked();
        array_2d_dynamic_nested_for_unchecked_unchecked();
        array_2d_dynamic_nested_dynamic_for_unchecked_unchecked();
        array_3d_for_unchecked_unchecked();
        array_3d_nested_for_unchecked_unchecked();
        array_3d_nested_dynamic_for_unchecked_unchecked();
        array_3d_dynamic_for_unchecked_unchecked();
        array_3d_dynamic_nested_for_unchecked_unchecked();
        array_3d_dynamic_nested_dynamic_for_unchecked_unchecked();
        array_4d_for_unchecked_unchecked();
        array_4d_nested_for_unchecked_unchecked();
        array_4d_nested_dynamic_for_unchecked_unchecked();
        array_4d_dynamic_for_unchecked_unchecked();
        array_4d_dynamic_nested_for_unchecked_unchecked();
        array_4d_dynamic_nested_dynamic_for_unchecked_unchecked();
        array_of_bools_for_unchecked_unchecked();
        array_of_chars_for_unchecked_unchecked();
    }
}
