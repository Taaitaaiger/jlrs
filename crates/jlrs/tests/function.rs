mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use crate::util::JULIA;

    fn extend_lifetime() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let output = frame.output();

                    frame
                        .scope(|frame| {
                            let func = unsafe {
                                Module::base(&frame)
                                    .global(&frame, "+")
                                    .unwrap()
                                    .as_managed()
                            };
                            JlrsResult::Ok(func.root(output))
                        })
                        .unwrap();
                })
            })
        })
    }

    fn has_datatype() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|frame| {
                    let func_ty = unsafe {
                        Module::base(&frame)
                            .global(&frame, "+")
                            .unwrap()
                            .as_managed()
                            .datatype()
                    };

                    assert_eq!(func_ty.name(), "#+");
                })
            })
        })
    }

    #[test]
    fn function_tests() {
        extend_lifetime();
        has_datatype();
    }
}
