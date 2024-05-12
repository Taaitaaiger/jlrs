#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use std::sync::atomic::Ordering;

    use jlrs::{
        data::{
            managed::array::data::accessor::Accessor, types::construct_type::UnionTypeConstructor,
        },
        prelude::*,
    };

    use crate::util::JULIA;

    fn value_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Value>>()
                            .unwrap();

                        let accessor = arr.value_data();
                        let s = accessor[[0, 0]]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_value()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_symbol_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Symbol[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Symbol>>()
                            .unwrap();

                        let accessor = arr.value_data();
                        let s = accessor[[0, 0]]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_value()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_get() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Value>>()
                            .unwrap();

                        let accessor = arr.value_data();
                        let s = accessor
                            .get(&mut frame, [0, 0])
                            .unwrap()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_get_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .local_scope::<_, 2>(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Value>>()
                            .unwrap();

                        let accessor = arr.value_data();
                        let s = accessor
                            .get_unchecked(&mut frame, [0, 0])
                            .unwrap()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_get_value() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Value>>()
                            .unwrap();

                        let accessor = arr.value_data();
                        let s = accessor
                            .get_value(&mut frame, [0, 0])
                            .unwrap()
                            .unwrap()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_get_value_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Value>>()
                            .unwrap();

                        let accessor = arr.value_data();
                        let s = accessor
                            .get_value_unchecked(&mut frame, [0, 0])
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_as_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Value>>()
                            .unwrap();

                        let accessor = arr.value_data();
                        let slice = accessor.as_slice();
                        let s = slice[0]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_value()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_into_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Value>>()
                            .unwrap();

                        let accessor = arr.value_data();
                        let slice = accessor.into_slice();
                        let s = slice[0]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_value()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_value_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<Array>()
                            .unwrap();

                        let accessor = arr.try_value_data();
                        assert!(accessor.is_ok());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_value_data_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Int[1 2]")
                            .unwrap()
                            .cast::<Array>()
                            .unwrap();

                        let accessor = arr.try_value_data();
                        assert!(accessor.is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<Array>()
                            .unwrap();

                        let accessor = arr.value_data_unchecked();
                        let s = accessor[[0, 0]]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_value()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_value_union_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Union{Symbol, Int}[:foo 1]")
                            .unwrap()
                            .cast::<Array>()
                            .unwrap();

                        let accessor = arr.try_value_data();
                        assert!(accessor.is_ok());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_union_data_get() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Union{Symbol, Int}[:foo 1]")
                            .unwrap()
                            .cast::<TypedArray<UnionTypeConstructor<Symbol, isize>>>()
                            .unwrap();

                        let accessor = arr.try_value_data().unwrap();
                        let s = accessor
                            .get(&mut frame, [0, 0])
                            .unwrap()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_value_data_tests() {
        value_data();
        value_data_symbol_array();
        value_data_get();
        value_data_get_unchecked();
        value_data_get_value();
        value_data_get_value_unchecked();
        value_data_as_slice();
        value_data_into_slice();
        try_value_data();
        try_value_data_err();
        value_data_unchecked();
        try_value_union_data();
        value_union_data_get();
    }
}
