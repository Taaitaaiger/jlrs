mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn return_value_from_scope() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();

                    frame
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| frame.scope(|_| Ok(Value::new(output, 1usize))))?
                        .unbox::<usize>()
                });

            assert_eq!(out.unwrap(), 1);
        });
    }

    fn return_result_from_scope() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let out = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let output = frame.output();

                    frame
                        .scope(|mut frame| {
                            frame
                                .returning::<JlrsResult<_>>()
                                .scope(|mut frame| unsafe {
                                    let func =
                                        Module::base(&frame).function(&frame, "+")?.as_managed();
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

    #[test]
    fn output_frame_tests() {
        return_value_from_scope();
        return_result_from_scope();
    }
}
