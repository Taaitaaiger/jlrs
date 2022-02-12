#[cfg(feature = "sync-rt")]
mod tests {
    use super::super::super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn ok_to_jlrs_result() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_global, frame| unsafe {
                Value::eval_string(&mut *frame, "1 + 1")?.into_jlrs_result()?;

                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn exc_to_jlrs_result() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_global, frame| unsafe {
                Value::eval_string(&mut *frame, "1 + \"a\"")?.into_jlrs_result()?;

                Ok(())
            })
            .unwrap_err();
        });
    }
}
