mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use crate::util::JULIA;

    fn extend_lifetime() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let output = frame.output();

                        frame
                            .scope(|frame| {
                                let func = unsafe {
                                    Module::base(&frame).function(&frame, "+")?.as_managed()
                                };
                                JlrsResult::Ok(func.root(output))
                            })
                            .unwrap();

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn has_datatype() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let func_ty = unsafe {
                            Module::base(&frame)
                                .function(&frame, "+")?
                                .as_managed()
                                .datatype()
                        };

                        assert_eq!(func_ty.name(), "#+");

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    #[test]
    fn function_tests() {
        extend_lifetime();
        has_datatype();
    }
}
