#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use std::sync::atomic::Ordering;

    use jlrs::{data::managed::array::data::accessor::AccessorMut, prelude::*};

    use crate::util::JULIA;

    fn managed_data_mut() {
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

                        let accessor = arr.managed_data_mut();
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

    fn managed_data_mut_set_value() {
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

                        let mut accessor = arr.managed_data_mut();

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

    fn managed_data_mut_set() {
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

                        let mut accessor = arr.managed_data_mut();

                        let v = Symbol::new(&frame, "s");
                        accessor
                            .set_value(&mut frame, [0, 0], v.as_value())
                            .unwrap()
                            .unwrap();

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

    fn managed_data_mut_set_unchecked() {
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

                        let mut accessor = arr.managed_data_mut();

                        let v = Symbol::new(&frame, "s");
                        accessor.set_value_unchecked([0, 0], v.as_value());

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

    fn managed_data_mut_set_value_unchecked() {
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

                        let mut accessor = arr.managed_data_mut();

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

    fn try_managed_data_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let mut arr = Value::eval_string(&mut frame, "Symbol[:foo :bar]")
                            .unwrap()
                            .cast::<Array>()
                            .unwrap();

                        let accessor = arr.try_managed_data_mut::<Symbol>();
                        assert!(accessor.is_ok());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_managed_data_mut_err() {
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

                        let accessor = arr.try_managed_data_mut::<Symbol>();
                        assert!(accessor.is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn managed_data_mut_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let mut arr = Value::eval_string(&mut frame, "Symbol[:foo :bar]")
                            .unwrap()
                            .cast::<Array>()
                            .unwrap();

                        let accessor = arr.managed_data_mut_unchecked::<Symbol>();
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

    pub(crate) fn array_managed_data_mut_tests() {
        managed_data_mut();
        managed_data_mut_set();
        managed_data_mut_set_unchecked();
        managed_data_mut_set_value();
        managed_data_mut_set_value_unchecked();
        try_managed_data_mut();
        try_managed_data_mut_err();
        managed_data_mut_unchecked();
    }
}
