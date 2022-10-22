#[cfg(feature = "sync-rt")]
mod tests {
    use crate::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn extend_lifetime() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let output = frame.output();

                    frame
                        .scope(|frame| {
                            let func = unsafe {
                                Module::base(&frame)
                                    .function(&frame, "+")?
                                    .wrapper_unchecked()
                            };
                            Ok(func.root(output))
                        })
                        .unwrap();

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn has_datatype() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| {
                    let func_ty = unsafe {
                        Module::base(&frame)
                            .function(&frame, "+")?
                            .wrapper_unchecked()
                            .datatype()
                    };

                    assert_eq!(func_ty.name(), "#+");

                    Ok(())
                })
                .unwrap();
        })
    }
}
