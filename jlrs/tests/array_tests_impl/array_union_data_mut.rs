#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::{
            managed::array::data::accessor::AccessorMut,
            types::construct_type::UnionTypeConstructor,
        },
        prelude::*,
    };

    use crate::util::JULIA;

    fn union_data_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let accessor = arr.union_data_mut();

                        assert_eq!(accessor.get::<isize, _>([0, 0]).unwrap().unwrap(), 1);
                        assert_eq!(accessor.get::<f64, _>([0, 1]).unwrap().unwrap(), 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_mut_set() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let mut accessor = arr.union_data_mut();

                        assert!(accessor
                            .set([0, 0], DataType::float64_type(&frame), 1.0f64)
                            .is_ok());
                        assert!(accessor
                            .set([0, 1], DataType::int64_type(&frame), 2isize)
                            .is_ok());

                        assert_eq!(accessor.get::<f64, _>([0, 0]).unwrap().unwrap(), 1.0);
                        assert_eq!(accessor.get::<isize, _>([0, 1]).unwrap().unwrap(), 2);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }
    fn union_data_mut_set_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let mut accessor = arr.union_data_mut();

                        assert!(accessor
                            .set([0, 0], DataType::float32_type(&frame), 1.0f32)
                            .unwrap()
                            .is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_mut_set_typed() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let mut accessor = arr.union_data_mut();

                        assert!(accessor.set_typed::<f64, _>([0, 0], 1.0).is_ok());
                        assert!(accessor.set_typed::<isize, _>([0, 1], 2).is_ok());

                        assert_eq!(accessor.get::<f64, _>([0, 0]).unwrap().unwrap(), 1.0);
                        assert_eq!(accessor.get::<isize, _>([0, 1]).unwrap().unwrap(), 2);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_mut_set_typed_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let mut accessor = arr.union_data_mut();

                        assert!(accessor.set_typed::<f32, _>([0, 0], 1.0).unwrap().is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_mut_set_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let mut accessor = arr.union_data_mut();

                        accessor.set_unchecked([0, 0], DataType::float64_type(&frame), 1.0f64);
                        accessor.set_unchecked([0, 1], DataType::int64_type(&frame), 2isize);

                        assert_eq!(accessor.get::<f64, _>([0, 0]).unwrap().unwrap(), 1.0);
                        assert_eq!(accessor.get::<isize, _>([0, 1]).unwrap().unwrap(), 2);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_mut_set_typed_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let mut accessor = arr.union_data_mut();

                        accessor.set_typed_unchecked([0, 0], 1.0f64);
                        accessor.set_typed_unchecked([0, 1], 2isize);

                        assert_eq!(accessor.get::<f64, _>([0, 0]).unwrap().unwrap(), 1.0);
                        assert_eq!(accessor.get::<isize, _>([0, 1]).unwrap().unwrap(), 2);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_mut_set_value() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let mut accessor = arr.union_data_mut();

                        let v = Value::new(&mut frame, 1.0f64);
                        assert!(accessor.set_value(&mut frame, [0, 0], v).unwrap().is_ok());
                        assert_eq!(accessor.get::<f64, _>([0, 0]).unwrap().unwrap(), 1.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_mut_set_value_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let mut accessor = arr.union_data_mut();

                        let v = Value::new(&mut frame, 1.0f32);
                        assert!(accessor.set_value(&mut frame, [0, 0], v).unwrap().is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_mut_set_value_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let mut accessor = arr.union_data_mut();

                        let v = Value::new(&mut frame, 1.0f64);
                        accessor.set_value_unchecked([0, 0], v);
                        assert_eq!(accessor.get::<f64, _>([0, 0]).unwrap().unwrap(), 1.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_union_data_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr.cast::<Array>().unwrap();
                        let accessor = arr.try_union_data_mut();
                        assert!(accessor.is_ok());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_union_data_mut_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Int[1 2]").unwrap();
                        let mut arr = arr.cast::<Array>().unwrap();
                        let accessor = arr.try_union_data_mut();
                        assert!(accessor.is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_mut_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let mut arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let accessor = arr.union_data_mut_unchecked();

                        assert_eq!(accessor.get::<isize, _>([0, 0]).unwrap().unwrap(), 1);
                        assert_eq!(accessor.get::<f64, _>([0, 1]).unwrap().unwrap(), 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_union_data_mut_tests() {
        union_data_mut();

        union_data_mut_set();
        union_data_mut_set_err();
        union_data_mut_set_unchecked();

        union_data_mut_set_typed();
        union_data_mut_set_typed_err();
        union_data_mut_set_typed_unchecked();

        union_data_mut_set_value();
        union_data_mut_set_value_err();
        union_data_mut_set_value_unchecked();

        try_union_data_mut();
        try_union_data_mut_err();
        union_data_mut_unchecked();
    }
}
