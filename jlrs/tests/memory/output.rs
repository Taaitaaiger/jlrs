#[cfg(feature = "sync-rt")]
mod tests {
    use super::super::super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn output_can_be_created_and_used() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_global, frame| {
                frame
                    .value_scope(|output, frame| {
                        let output = output.into_scope(frame);
                        Value::new(output, 0usize)
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 1);

                frame
                    .value_scope_with_slots(0, |output, frame| {
                        let output = output.into_scope(frame);
                        Value::new(output, 0usize)
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 2);

                frame
                    .wrapper_scope::<Module, _>(|output, frame| {
                        let m = Module::core(frame.global());
                        let output = output.into_scope(frame);
                        Ok(m.as_value().as_unrooted(output))
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 3);

                frame
                    .wrapper_scope_with_slots::<Module, _>(0, |output, frame| {
                        let m = Module::core(frame.global());
                        let output = output.into_scope(frame);
                        Ok(m.as_value().as_unrooted(output))
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 4);

                frame
                    .wrapper_result_scope::<Module, _>(|output, frame| unsafe {
                        let output = output.into_scope(frame);
                        Value::eval_string(output, "Base")
                    })
                    .unwrap()
                    .unwrap();

                assert_eq!(frame.n_roots(), 5);

                frame
                    .wrapper_result_scope_with_slots::<Module, _>(0, |output, frame| unsafe {
                        let output = output.into_scope(frame);
                        Value::eval_string(output, "Base")
                    })
                    .unwrap()
                    .unwrap();

                assert_eq!(frame.n_roots(), 6);

                frame
                    .result_scope(|output, frame| unsafe {
                        let output = output.into_scope(frame);
                        Value::eval_string(output, "0")
                    })
                    .unwrap()
                    .unwrap();

                assert_eq!(frame.n_roots(), 7);

                frame
                    .result_scope_with_slots(0, |output, frame| unsafe {
                        let output = output.into_scope(frame);
                        Value::eval_string(output, "0")
                    })
                    .unwrap()
                    .unwrap();

                assert_eq!(frame.n_roots(), 8);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn output_can_be_propagated() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_global, frame| {
                frame
                    .value_scope(|output, frame| {
                        let output = output.into_scope(frame);
                        output.value_scope(|output, frame| {
                            let output = output.into_scope(frame);
                            Value::new(output, 0usize)
                        })
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 1);

                frame
                    .value_scope_with_slots(0, |output, frame| {
                        let output = output.into_scope(frame);
                        output.value_scope_with_slots(0, |output, frame| {
                            let output = output.into_scope(frame);
                            Value::new(output, 0usize)
                        })
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 2);

                frame
                    .result_scope(|output, frame| {
                        let output = output.into_scope(frame);
                        output.result_scope(|output, frame| unsafe {
                            let output = output.into_scope(frame);
                            Value::eval_string(output, "0")
                        })
                    })
                    .unwrap()
                    .unwrap();

                assert_eq!(frame.n_roots(), 3);

                frame
                    .result_scope_with_slots(0, |output, frame| {
                        let output = output.into_scope(frame);
                        output.result_scope_with_slots(0, |output, frame| unsafe {
                            let output = output.into_scope(frame);
                            Value::eval_string(output, "0")
                        })
                    })
                    .unwrap()
                    .unwrap();

                assert_eq!(frame.n_roots(), 4);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn output_propagates_error() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_global, frame| {
                frame
                    .wrapper_scope::<JuliaString, _>(|output, frame| {
                        let m = Module::core(frame.global());
                        let output = output.into_scope(frame);
                        Ok(m.as_value().as_unrooted(output))
                    })
                    .unwrap_err();

                frame
                    .wrapper_scope_with_slots::<JuliaString, _>(0, |output, frame| {
                        let m = Module::core(frame.global());
                        let output = output.into_scope(frame);
                        Ok(m.as_value().as_unrooted(output))
                    })
                    .unwrap_err();

                frame
                    .wrapper_result_scope::<JuliaString, _>(|output, frame| unsafe {
                        let output = output.into_scope(frame);
                        Value::eval_string(output, "throw(ErrorException(\"1\"))")
                    })
                    .unwrap()
                    .unwrap_err();

                frame
                    .wrapper_result_scope_with_slots::<JuliaString, _>(0, |output, frame| unsafe {
                        let output = output.into_scope(frame);
                        Value::eval_string(output, "throw(ErrorException(\"1\"))")
                    })
                    .unwrap()
                    .unwrap_err();

                frame
                    .wrapper_result_scope::<JuliaString, _>(|output, frame| unsafe {
                        let output = output.into_scope(frame);
                        Value::eval_string(output, "Base")
                    })
                    .unwrap_err();

                frame
                    .wrapper_result_scope_with_slots::<JuliaString, _>(0, |output, frame| unsafe {
                        let output = output.into_scope(frame);
                        Value::eval_string(output, "Base")
                    })
                    .unwrap_err();

                frame
                    .result_scope(|output, frame| unsafe {
                        let output = output.into_scope(frame);
                        Value::eval_string(output, "throw(ErrorException(\"1\"))")
                    })
                    .unwrap()
                    .unwrap_err();

                frame
                    .result_scope_with_slots(0, |output, frame| unsafe {
                        let output = output.into_scope(frame);
                        Value::eval_string(output, "throw(ErrorException(\"1\"))")
                    })
                    .unwrap()
                    .unwrap_err();

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn propagated_output_propagates_exception() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_global, frame| {
                frame
                    .result_scope(|output, frame| unsafe {
                        let output = output.into_scope(frame);
                        output.result_scope(|output, frame| {
                            let output = output.into_scope(frame);
                            Value::eval_string(output, "throw(ErrorException(\"1\"))")
                        })
                    })
                    .unwrap()
                    .unwrap_err();

                frame
                    .result_scope_with_slots(0, |output, frame| unsafe {
                        let output = output.into_scope(frame);
                        output.result_scope_with_slots(0, |output, frame| {
                            let output = output.into_scope(frame);
                            let v = Value::eval_string(output, "throw(ErrorException(\"1\"))");

                            v.map(|e| {
                                assert!(e.is_exception());
                                e
                            })
                        })
                    })
                    .unwrap()
                    .unwrap_err();

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn output_not_created_if_frame_full() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_global, frame| {
                while frame.n_roots() != frame.capacity() {
                    Value::new(&mut *frame, 0u8)?;
                }

                let err = frame.value_scope(|_output, _frame| unreachable!());
                assert!(err.is_err());

                let err = frame.value_scope_with_slots(0, |_output, _frame| unreachable!());
                assert!(err.is_err());

                let err = frame.wrapper_scope::<Module, _>(|_output, _frame| unreachable!());
                assert!(err.is_err());

                let err = frame
                    .wrapper_scope_with_slots::<Module, _>(0, |_output, _frame| unreachable!());
                assert!(err.is_err());

                let err = frame.wrapper_result_scope::<Module, _>(|_output, _frame| unreachable!());
                assert!(err.is_err());

                let err = frame.wrapper_result_scope_with_slots::<Module, _>(
                    0,
                    |_output, _frame| unreachable!(),
                );
                assert!(err.is_err());

                let err = frame.result_scope(|_output, _frame| unreachable!());
                assert!(err.is_err());

                let err = frame.result_scope_with_slots(0, |_output, _frame| unreachable!());
                assert!(err.is_err());

                Ok(())
            })
            .unwrap()
        })
    }
}
