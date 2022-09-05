#[cfg(feature = "sync-rt")]
mod tests {
    use crate::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn extend_lifetime() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_, mut frame| {
                let output = frame.output()?;

                frame
                    .scope(|mut frame| {
                        let global = frame.as_scope().global();
                        let func =
                            unsafe { Module::base(global).function_ref("+")?.wrapper_unchecked() };
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
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_, mut frame| {
                let global = frame.as_scope().global();
                let func_ty = unsafe {
                    Module::base(global)
                        .function_ref("+")?
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
