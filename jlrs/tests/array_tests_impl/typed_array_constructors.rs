#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::prelude::*;

    use crate::util::JULIA;

    fn typed_array_new_rank0() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let arr = TypedArray::<f32>::new(&mut frame, []);
                    assert!(arr.is_ok());

                    let arr = arr.unwrap();
                    assert_eq!(arr.n_dims(), 0);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn typed_array_new() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let arr = TypedArray::<f32>::new(&mut frame, (1, 2));
                    assert!(arr.is_ok());

                    let arr = arr.unwrap();
                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn typed_array_new_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let arr = unsafe { TypedArray::<f32>::new_unchecked(&mut frame, (1, 2)) };
                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn typed_array_from_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = TypedArray::<f32>::from_slice(&mut frame, data, (1, 2));
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

    fn typed_array_from_slice_rank0() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32];
                    let data = data.as_mut_slice();
                    let arr = TypedArray::<f32>::from_slice(&mut frame, data, []);
                    assert!(arr.is_ok());
                    let arr = arr.unwrap();
                    assert!(arr.is_ok());
                    let arr = arr.unwrap();

                    assert_eq!(arr.n_dims(), 0);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn typed_array_from_slice_size_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = TypedArray::<f32>::from_slice(&mut frame, data, (2, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn typed_array_from_slice_unchecked() {
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
                        TypedArray::<f32>::from_slice_unchecked(&mut frame, data, (1, 2))
                    };

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn typed_array_from_vec() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let data = vec![1f32, 2f32];
                    let arr = TypedArray::<f32>::from_vec(&mut frame, data, (1, 2));
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

    fn typed_array_from_vec_size_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let data = vec![1f32, 2f32];
                    let arr = TypedArray::<f32>::from_vec(&mut frame, data, (2, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn typed_array_from_vec_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let data = vec![1f32, 2f32];
                    let arr =
                        unsafe { TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2)) };

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn typed_array_from_slice_cloned() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = TypedArray::<f32>::from_slice_cloned(&mut frame, data, (1, 2));
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

    fn typed_array_from_slice_cloned_size_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let mut data = vec![1f32, 2f32];
                    let data = data.as_mut_slice();
                    let arr = TypedArray::<f32>::from_slice_cloned(&mut frame, data, (2, 2));
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn typed_array_from_slice_cloned_unchecked() {
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
                        TypedArray::<f32>::from_slice_cloned_unchecked(&mut frame, data, (1, 2))
                    };

                    assert_eq!(arr.n_dims(), 2);
                    assert_eq!(arr.element_type(), dt);
                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn typed_array_constructors_tests() {
        typed_array_new_rank0();

        typed_array_new();
        typed_array_new_unchecked();

        typed_array_from_slice();
        typed_array_from_slice_rank0();
        typed_array_from_slice_size_err();
        typed_array_from_slice_unchecked();

        typed_array_from_vec();
        typed_array_from_vec_size_err();
        typed_array_from_vec_unchecked();

        typed_array_from_slice_cloned();
        typed_array_from_slice_cloned_size_err();
        typed_array_from_slice_cloned_unchecked();
    }
}
