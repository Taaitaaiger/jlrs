#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::{
            managed::array::data::accessor::Accessor, types::construct_type::UnionTypeConstructor,
        },
        prelude::*,
    };

    use crate::util::JULIA;

    fn union_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let accessor = arr.union_data();

                        assert_eq!(accessor.get::<isize, _>([0, 0]).unwrap().unwrap(), 1);
                        assert_eq!(accessor.get::<f64, _>([0, 1]).unwrap().unwrap(), 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_get_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let accessor = arr.union_data();

                        assert_eq!(accessor.get_unchecked::<isize, _>([0, 0]), 1);
                        assert_eq!(accessor.get_unchecked::<f64, _>([0, 1]), 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_get_value() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let accessor = arr.union_data();

                        assert_eq!(
                            accessor
                                .get_value(&mut frame, [0, 0])
                                .unwrap()
                                .unwrap()
                                .unbox::<isize>()
                                .unwrap(),
                            1
                        );
                        assert_eq!(
                            accessor
                                .get_value(&mut frame, [0, 1])
                                .unwrap()
                                .unwrap()
                                .unbox::<f64>()
                                .unwrap(),
                            2.0
                        );
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_get_value_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let accessor = arr.union_data();

                        assert_eq!(
                            accessor
                                .get_value_unchecked(&mut frame, [0, 0])
                                .unbox::<isize>()
                                .unwrap(),
                            1
                        );
                        assert_eq!(
                            accessor
                                .get_value_unchecked(&mut frame, [0, 1])
                                .unbox::<f64>()
                                .unwrap(),
                            2.0
                        );
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_union_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let arr = arr
                            .cast::<TypedArray<UnionTypeConstructor<isize, f64>>>()
                            .unwrap();
                        let accessor = arr.try_union_data();
                        assert!(accessor.is_ok());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_union_data_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Int[1 2]").unwrap();
                        let arr = arr.cast::<Array>().unwrap();
                        let accessor = arr.try_union_data();
                        assert!(accessor.is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn union_data_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "Union{Int, Float64}[1 2.0]").unwrap();
                        let arr = arr.cast::<Array>().unwrap();
                        let accessor = arr.union_data_unchecked();

                        assert_eq!(accessor.get::<isize, _>([0, 0]).unwrap().unwrap(), 1);
                        assert_eq!(accessor.get::<f64, _>([0, 1]).unwrap().unwrap(), 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_union_data_tests() {
        union_data();
        union_data_get_unchecked();

        union_data_get_value();
        union_data_get_value_unchecked();

        try_union_data();
        try_union_data_err();
        union_data_unchecked();
    }
}
