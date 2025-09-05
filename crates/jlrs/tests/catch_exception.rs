mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{catch::catch_exceptions, prelude::*, weak_handle_unchecked};

    use super::util::JULIA;

    fn call0_exception_is_caught() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_managed();

                    let mut f = || func.call_unchecked(&mut frame, []);

                    let mut exc = false;
                    let res = catch_exceptions(&mut f, |_e| {
                        exc = true;
                    });
                    assert!(exc);
                    assert!(res.is_err());
                });
            });
        });
    }

    unsafe fn throwing_inner(func: Value) {
        unsafe {
            let f2 = || {
                let handle = weak_handle_unchecked!();
                func.call_unchecked(&handle, []);
                assert!(false);
            };

            let _ = catch_exceptions(f2, |e| {
                e.rethrow();
            });

            assert!(false);
        }
    }

    unsafe fn catching_outer(func: Value) {
        unsafe {
            let f2 = || {
                throwing_inner(func);
                assert!(false);
            };

            let mut exc = false;
            let _ = catch_exceptions(f2, |_e| {
                exc = true;
            });

            assert!(exc);
        }
    }

    fn rethrow_exception() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|frame| unsafe {
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "throws_exception")
                        .unwrap()
                        .as_managed();

                    catching_outer(func);
                });
            });
        });
    }

    #[test]
    fn call_exception_tests() {
        call0_exception_is_caught();
        rethrow_exception();
    }
}
