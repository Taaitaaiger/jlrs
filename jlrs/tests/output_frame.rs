mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn nested_value_scope() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs.instance(&mut frame).scope(|mut frame| {
                let output = frame.output();

                frame
                    .scope(|mut frame| frame.scope(|_| Ok(Value::new(output, 1usize))))?
                    .unbox::<usize>()
            });

            assert_eq!(out.unwrap(), 1);
        });
    }

    #[test]
    fn nested_result_scope() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs.instance(&mut frame).scope(|mut frame| {
                let output = frame.output();

                frame
                    .scope(|mut frame| {
                        frame.scope(|mut frame| unsafe {
                            let func = Module::base(&frame)
                                .function(&frame, "+")?
                                .wrapper_unchecked();
                            let v1 = Value::new(frame.as_mut(), 1usize);
                            let v2 = Value::new(frame.as_mut(), 2usize);
                            Ok(func.call2(output, v1, v2))
                        })
                    })?
                    .unwrap()
                    .unbox::<usize>()
            });

            assert_eq!(out.unwrap(), 3);
        });
    }
}
