#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::prelude::*;

    use crate::util::JULIA;

    fn array_new_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let arr = Array::new_for(&mut frame, dt, (1, 2));
                    assert!(arr.is_ok());

                    let arr = arr.unwrap();
                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_new_for_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let arr = unsafe { Array::new_for_unchecked(&mut frame, dt, (1, 2)) };
                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_slice_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = Array::from_slice_for(&mut frame, dt, data, (1, 2));
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

    fn array_from_slice_for_type_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float64_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = Array::from_slice_for(&mut frame, dt, data, (1, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_slice_for_size_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = Array::from_slice_for(&mut frame, dt, data, (2, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_slice_for_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr =
                        unsafe { Array::from_slice_for_unchecked(&mut frame, dt, data, (1, 2)) };

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_vec_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let data = vec![1f32, 2f32];
                    let arr = Array::from_vec_for(&mut frame, dt, data, (1, 2));
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

    fn array_from_vec_for_type_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float64_type(&frame).as_value();
                    let data = vec![1f32, 2f32];
                    let arr = Array::from_vec_for(&mut frame, dt, data, (1, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_vec_for_size_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let data = vec![1f32, 2f32];
                    let arr = Array::from_vec_for(&mut frame, dt, data, (2, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_vec_for_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let data = vec![1f32, 2f32];
                    let arr =
                        unsafe { Array::from_vec_for_unchecked(&mut frame, dt, data, (1, 2)) };

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_slice_cloned_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = Array::from_slice_cloned_for(&mut frame, dt, data, (1, 2));
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

    fn array_from_slice_cloned_for_type_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float64_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = Array::from_slice_cloned_for(&mut frame, dt, data, (1, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_slice_cloned_for_size_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = Array::from_slice_cloned_for(&mut frame, dt, data, (2, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_slice_cloned_for_unchecked() {
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
                        Array::from_slice_cloned_for_unchecked(&mut frame, dt, data, (1, 2))
                    };

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_slice_copied_for() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = Array::from_slice_copied_for(&mut frame, dt, data, (1, 2));
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

    fn array_from_slice_copied_for_type_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float64_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = Array::from_slice_copied_for(&mut frame, dt, data, (1, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_slice_copied_for_size_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = Array::from_slice_copied_for(&mut frame, dt, data, (2, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_from_slice_copied_for_unchecked() {
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
                        Array::from_slice_copied_for_unchecked(&mut frame, dt, data, (1, 2))
                    };

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    // TODO: Test 1D, 2D, 3D, and 4D cases to cover all ctors
    // TODO: Test that exception is thrown if the number of elements >= isize::MAX
    pub(crate) fn array_constructor_tests() {
        array_new_for();
        array_new_for_unchecked();

        array_from_slice_for();
        array_from_slice_for_size_err();
        array_from_slice_for_type_err();
        array_from_slice_for_unchecked();

        array_from_vec_for();
        array_from_vec_for_size_err();
        array_from_vec_for_type_err();
        array_from_vec_for_unchecked();

        array_from_slice_cloned_for();
        array_from_slice_cloned_for_size_err();
        array_from_slice_cloned_for_type_err();
        array_from_slice_cloned_for_unchecked();

        array_from_slice_copied_for();
        array_from_slice_copied_for_size_err();
        array_from_slice_copied_for_type_err();
        array_from_slice_copied_for_unchecked();
    }
}
