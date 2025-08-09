mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{catch::catch_exceptions, prelude::*};

    use super::util::JULIA;

    fn call0_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| unsafe {
                        let func = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .as_managed()
                            .global(&frame, "throws_exception")?
                            .as_managed();

                        let mut f = || func.call_unchecked(&mut frame, []);

                        let mut exc = false;
                        let res = catch_exceptions(&mut f, |_e| {
                            exc = true;
                        });
                        assert!(exc);
                        assert!(res.is_err());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    #[test]
    fn call_exception_tests() {
        call0_exception_is_caught();
    }
}
