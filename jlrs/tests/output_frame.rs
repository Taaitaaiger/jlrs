mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn nested_value_scope() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(1, |_global, frame| {
                let (output, frame) = frame.split()?;

                frame
                    .scope_with_capacity(0, |frame| {
                        frame.scope_with_capacity(0, |frame| {
                            let output = output.into_scope(frame);
                            Value::new(output, 1usize)
                        })
                    })?
                    .unbox::<usize>()
            });

            assert_eq!(out.unwrap(), 1);
        });
    }

    #[test]
    fn nested_result_scope() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            let out = jlrs.scope_with_capacity(1, |global, frame| {
                let (output, frame) = frame.split()?;

                frame
                    .scope_with_capacity(0, |frame| {
                        frame.scope_with_capacity(2, |frame| unsafe {
                            let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                            let v1 = Value::new(frame.as_scope(), 1usize)?;
                            let v2 = Value::new(frame.as_scope(), 2usize)?;
                            let output = output.into_scope(frame);
                            func.call2(output, v1, v2)
                        })
                    })?
                    .unwrap()
                    .unbox::<usize>()
            });

            assert_eq!(out.unwrap(), 3);
        });
    }
}
