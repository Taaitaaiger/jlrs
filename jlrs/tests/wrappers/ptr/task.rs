#[cfg(feature = "sync-rt")]
mod tests {
    use crate::util::JULIA;
    use jlrs::{prelude::*, wrappers::ptr::task::Task};

    #[test]
    fn extend_lifetime() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_, mut frame| {
                let output = frame.output();

                frame
                    .scope(|mut frame| {
                        let task = unsafe {
                            Value::eval_string(&mut frame, "@task 1 + 2")
                                .into_jlrs_result()?
                                .cast::<Task>()?
                                .clone()
                        };
                        Ok(task.root(output))
                    })
                    .unwrap();
                Ok(())
            })
            .unwrap();
        })
    }
}
