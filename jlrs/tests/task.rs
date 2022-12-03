mod util;

#[cfg(feature = "sync-rt")]
mod tests {
    use jlrs::{data::managed::task::Task, prelude::*};

    use super::util::JULIA;

    #[test]
    fn extend_lifetime() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
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
