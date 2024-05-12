#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use std::sync::atomic::Ordering;

    use jlrs::{data::managed::array::data::accessor::Accessor, prelude::*};

    use crate::util::JULIA;

    fn managed_data() {
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

                        let accessor = arr.managed_data();
                        let s = accessor[[0, 0]]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_managed()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn managed_data_symbol_array() {
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

                        let accessor = arr.managed_data();
                        let s = accessor[[0, 0]]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_managed()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn managed_data_get() {
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

                        let accessor = arr.managed_data();
                        let s = accessor.get(&mut frame, [0, 0]).unwrap().as_str().unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn managed_data_get_unchecked() {
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

                        let accessor = arr.managed_data();
                        let s = accessor
                            .get_unchecked(&mut frame, [0, 0])
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

    fn managed_data_get_value() {
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

                        let accessor = arr.managed_data();
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

    fn managed_data_get_value_unchecked() {
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

                        let accessor = arr.managed_data();
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

    fn managed_data_as_slice() {
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

                        let accessor = arr.managed_data();
                        let slice = accessor.as_slice();
                        let s = slice[0]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_managed()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn managed_data_into_slice() {
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

                        let accessor = arr.managed_data();
                        let slice = accessor.into_slice();
                        let s = slice[0]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_managed()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_managed_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Symbol[:foo :bar]")
                            .unwrap()
                            .cast::<Array>()
                            .unwrap();

                        let accessor = arr.try_managed_data::<Symbol>();
                        assert!(accessor.is_ok());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_managed_data_err() {
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

                        let accessor = arr.try_managed_data::<Symbol>();
                        assert!(accessor.is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn managed_data_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(&mut frame, "Symbol[:foo :bar]")
                            .unwrap()
                            .cast::<Array>()
                            .unwrap();

                        let accessor = arr.managed_data_unchecked::<Symbol>();
                        let s = accessor[[0, 0]]
                            .load(Ordering::Relaxed)
                            .unwrap()
                            .as_managed()
                            .as_str()
                            .unwrap();

                        assert_eq!(s, "foo");
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_managed_data_tests() {
        managed_data();
        managed_data_symbol_array();
        managed_data_get();
        managed_data_get_unchecked();
        managed_data_get_value();
        managed_data_get_value_unchecked();
        managed_data_as_slice();
        managed_data_into_slice();
        try_managed_data();
        try_managed_data_err();
        managed_data_unchecked();
    }
}
