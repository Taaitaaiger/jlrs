#[cfg(feature = "sync-rt")]
mod tests {
    use super::super::super::util::JULIA;
    use jlrs::{convert::as_unrooted::AsUnrooted, prelude::*, wrappers::ptr::function::Function};

    #[test]
    fn return_value_as_unrooted() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, frame| {
                let v = frame.value_scope(|output, frame| {
                    let v = Module::base(global).function(&mut *frame, "+")?;
                    let output = output.into_scope(frame);
                    Ok(v.as_value().as_unrooted(output))
                })?;

                assert!(v.is::<Function>());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn return_result_as_unrooted() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, frame| {
                let v = frame
                    .result_scope(|output, frame| unsafe {
                        let zero = Value::new(&mut *frame, 0usize)?;

                        let v = Module::base(global).function(&mut *frame, "+")?.call2(
                            &mut *frame,
                            zero,
                            zero,
                        )?;

                        let output = output.into_scope(frame);
                        Ok(v.as_unrooted(output))
                    })?
                    .into_jlrs_result()?;

                assert!(v.is::<usize>());
                Ok(())
            })
            .unwrap();
        });
    }
}
