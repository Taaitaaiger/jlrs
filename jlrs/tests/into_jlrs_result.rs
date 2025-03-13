mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn ok_to_jlrs_result() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| unsafe {
                        Value::eval_string(&mut frame, "1 + 1").into_jlrs_result()?;

                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn exc_to_jlrs_result() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| unsafe {
                        Value::eval_string(&mut frame, "1 + \"a\"").into_jlrs_result()?;

                        Ok(())
                    })
                    .unwrap_err();
            });
        });
    }

    #[test]
    fn test_into_jlrs_result() {
        ok_to_jlrs_result();
        exc_to_jlrs_result();
    }
}
