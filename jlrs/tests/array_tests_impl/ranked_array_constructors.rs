#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{data::managed::array::RankedArray, prelude::*};

    use crate::util::JULIA;

    fn ranked_array_new_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let arr = RankedArray::<2>::new_for(&mut frame, dt, (1, 2));
                    assert!(arr.is_ok());

                    let arr = arr.unwrap();
                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_new_for_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let arr =
                        unsafe { RankedArray::<2>::new_for_unchecked(&mut frame, dt, (1, 2)) };
                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_slice_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = RankedArray::<2>::from_slice_for(&mut frame, dt, data, (1, 2));
                    assert!(arr.is_ok());
                    let arr = arr.unwrap();
                    assert!(arr.is_ok());
                    let arr = arr.unwrap();

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_slice_for_type_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float64_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = RankedArray::<2>::from_slice_for(&mut frame, dt, data, (1, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_slice_for_size_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = RankedArray::<2>::from_slice_for(&mut frame, dt, data, (2, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_slice_for_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = unsafe {
                        RankedArray::<2>::from_slice_for_unchecked(&mut frame, dt, data, (1, 2))
                    };

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_vec_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let data = vec![1f32, 2f32];
                    let arr = RankedArray::<2>::from_vec_for(&mut frame, dt, data, (1, 2));
                    assert!(arr.is_ok());
                    let arr = arr.unwrap();
                    assert!(arr.is_ok());
                    let arr = arr.unwrap();

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_vec_for_type_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float64_type(&frame).as_value();
                    let data = vec![1f32, 2f32];
                    let arr = RankedArray::<2>::from_vec_for(&mut frame, dt, data, (1, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_vec_for_size_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let data = vec![1f32, 2f32];
                    let arr = RankedArray::<2>::from_vec_for(&mut frame, dt, data, (2, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_vec_for_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let data = vec![1f32, 2f32];
                    let arr = unsafe {
                        RankedArray::<2>::from_vec_for_unchecked(&mut frame, dt, data, (1, 2))
                    };

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_slice_cloned_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = RankedArray::<2>::from_slice_cloned_for(&mut frame, dt, data, (1, 2));
                    assert!(arr.is_ok());
                    let arr = arr.unwrap();
                    assert!(arr.is_ok());
                    let arr = arr.unwrap();

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_slice_cloned_for_type_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float64_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = RankedArray::<2>::from_slice_cloned_for(&mut frame, dt, data, (1, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_slice_cloned_for_size_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = RankedArray::<2>::from_slice_cloned_for(&mut frame, dt, data, (2, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn ranked_array_from_slice_cloned_for_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = unsafe {
                        RankedArray::<2>::from_slice_cloned_for_unchecked(
                            &mut frame,
                            dt,
                            data,
                            (1, 2),
                        )
                    };

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn ranked_array_constructors_tests() {
        ranked_array_new_for();
        ranked_array_new_for_unchecked();

        ranked_array_from_slice_for();
        ranked_array_from_slice_for_size_err();
        ranked_array_from_slice_for_type_err();
        ranked_array_from_slice_for_unchecked();

        ranked_array_from_vec_for();
        ranked_array_from_vec_for_size_err();
        ranked_array_from_vec_for_type_err();
        ranked_array_from_vec_for_unchecked();

        ranked_array_from_slice_cloned_for();
        ranked_array_from_slice_cloned_for_size_err();
        ranked_array_from_slice_cloned_for_type_err();
        ranked_array_from_slice_cloned_for_unchecked();
    }
}
