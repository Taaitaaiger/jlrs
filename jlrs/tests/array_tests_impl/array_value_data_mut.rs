#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use std::sync::atomic::Ordering;

    use jlrs::{data::managed::array::data::accessor::AccessorMut, prelude::*};

    use crate::util::JULIA;

    fn value_data_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let mut arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Value>>()
                            .unwrap();

                        let accessor = arr.value_data_mut();
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

    fn value_data_mut_set_value() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let mut arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Value>>()
                            .unwrap();

                        let mut accessor = arr.value_data_mut();

                        let v = Symbol::new(&frame, "s").as_value();
                        assert!(accessor.set_value(&mut frame, [0, 0], v).unwrap().is_ok());

                        let s = accessor[[0, 0]]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_value()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "s");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_mut_set_value_typed() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let mut arr = Value::eval_string(&mut frame, "Symbol[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Symbol>>()
                            .unwrap();

                        let mut accessor = arr.value_data_mut();

                        let v = Symbol::new(&frame, "s").as_value();
                        assert!(accessor.set_value(&mut frame, [0, 0], v).unwrap().is_ok());

                        let s = accessor[[0, 0]]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_value()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "s");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_mut_set_value_typed_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let mut arr = Value::eval_string(&mut frame, "Symbol[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Symbol>>()
                            .unwrap();

                        let mut accessor = arr.value_data_mut();

                        let v = Value::new(&mut frame, 1usize);
                        assert!(accessor.set_value(&mut frame, [0, 0], v).unwrap().is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_mut_set_value_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let mut arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<TypedArray<Value>>()
                            .unwrap();

                        let mut accessor = arr.value_data_mut();

                        let v = Symbol::new(&frame, "s").as_value();
                        accessor.set_value_unchecked([0, 0], v);

                        let s = accessor[[0, 0]]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_value()
                            .cast::<Symbol>()
                            .unwrap()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "s");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_value_data_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let mut arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<Array>()
                            .unwrap();

                        let accessor = arr.try_value_data_mut();
                        assert!(accessor.is_ok());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_value_data_mut_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let mut arr = Value::eval_string(&mut frame, "Int[1 2]")
                            .unwrap()
                            .cast::<Array>()
                            .unwrap();

                        let accessor = arr.try_value_data_mut();
                        assert!(accessor.is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_data_mut_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let mut arr = Value::eval_string(&mut frame, "Any[:foo :bar]")
                            .unwrap()
                            .cast::<Array>()
                            .unwrap();

                        let accessor = arr.value_data_mut_unchecked();
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

    pub(crate) fn array_value_data_mut_tests() {
        value_data_mut();
        value_data_mut_set_value();
        value_data_mut_set_value_typed();
        value_data_mut_set_value_typed_err();
        value_data_mut_set_value_unchecked();
        try_value_data_mut();
        try_value_data_mut_err();
        value_data_mut_unchecked();
    }
}
