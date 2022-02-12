#[cfg(feature = "sync-rt")]
mod tests {
    use super::super::super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn frame_starts_with_no_roots() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_slots(0, |_global, frame| {
                assert_eq!(frame.n_roots(), 0);
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn allocation_creates_root() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_slots(0, |_global, frame| {
                Value::new(&mut *frame, 0usize)?;
                assert_eq!(frame.n_roots(), 1);
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn allocation_fails_if_capacity_exceeded() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_slots(0, |_global, frame| {
                for _ in 0..frame.capacity() {
                    Value::new(&mut *frame, 0usize)?;
                }

                assert_eq!(frame.n_roots(), frame.capacity());
                assert!(Value::new(&mut *frame, 0usize).is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn frames_can_be_nested() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_slots(0, |_global, frame| {
                let cap = frame.capacity();
                frame
                    .scope(|frame| {
                        let inner_cap = frame.capacity();
                        assert_eq!(inner_cap, cap - 2);
                        assert_eq!(frame.n_roots(), 0);
                        Ok(())
                    })
                    .unwrap();

                assert_eq!(frame.capacity(), cap);
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn new_page_is_allocated_if_necessary() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_slots(0, |_global, frame| {
                let cap = frame.capacity();

                for _ in 0..frame.capacity() {
                    Value::new(&mut *frame, 0usize)?;
                }

                frame
                    .scope(|frame| {
                        let inner_cap = frame.capacity();
                        assert_eq!(inner_cap, cap);
                        Ok(())
                    })
                    .unwrap();

                assert_eq!(frame.capacity(), cap);
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn new_page_is_reused() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_slots(0, |_global, frame| {
                let cap = frame.capacity();

                for _ in 0..frame.capacity() {
                    Value::new(&mut *frame, 0usize)?;
                }

                frame
                    .scope_with_slots(128, |frame| {
                        let inner_cap = frame.capacity();
                        assert_eq!(inner_cap, 128);
                        Ok(())
                    })
                    .unwrap();

                frame
                    .scope_with_slots(64, |frame| {
                        let inner_cap = frame.capacity();
                        assert_eq!(inner_cap, 128);
                        Ok(())
                    })
                    .unwrap();

                assert_eq!(frame.capacity(), cap);
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn new_page_realloacated_if_necessary() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_slots(0, |_global, frame| {
                let cap = frame.capacity();

                for _ in 0..frame.capacity() {
                    Value::new(&mut *frame, 0usize)?;
                }

                frame
                    .scope_with_slots(128, |frame| {
                        let inner_cap = frame.capacity();
                        assert_eq!(inner_cap, 128);
                        Ok(())
                    })
                    .unwrap();

                frame
                    .scope_with_slots(129, |frame| {
                        let inner_cap = frame.capacity();
                        assert_eq!(inner_cap, 129);
                        Ok(())
                    })
                    .unwrap();

                assert_eq!(frame.capacity(), cap);
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    #[cfg(feature = "ccall")]
    fn create_null_frame() {
        let mut ccall = unsafe { CCall::new() };
        ccall.null_scope(|_frame| Ok(())).unwrap();
    }

    #[test]
    #[cfg(feature = "ccall")]
    fn null_frame_is_empty() {
        let mut ccall = unsafe { CCall::new() };

        ccall
            .null_scope(|frame| {
                assert_eq!(frame.n_roots(), 0);
                assert_eq!(frame.capacity(), 0);
                assert!(Value::new(&mut *frame, 0usize).is_err());
                Ok(())
            })
            .unwrap();
    }

    #[test]
    #[cfg(feature = "ccall")]
    fn cannot_nest_null_frame() {
        let mut ccall = unsafe { CCall::new() };

        ccall
            .null_scope(|frame| {
                assert!(frame.scope(|_| { Ok(()) }).is_err());
                Ok(())
            })
            .unwrap();

        ccall
            .null_scope(|frame| {
                assert!(frame.scope_with_slots(0, |_| { Ok(()) }).is_err());
                Ok(())
            })
            .unwrap();

        ccall
            .null_scope(|frame| {
                assert!(frame.value_scope(|_, _| { unreachable!() }).is_err());
                Ok(())
            })
            .unwrap();

        ccall
            .null_scope(|frame| {
                assert!(frame
                    .value_scope_with_slots(0, |_, _| { unreachable!() })
                    .is_err());
                Ok(())
            })
            .unwrap();

        ccall
            .null_scope(|frame| {
                assert!(frame.result_scope(|_, _| { unreachable!() }).is_err());
                Ok(())
            })
            .unwrap();

        ccall
            .null_scope(|frame| {
                assert!(frame
                    .result_scope_with_slots(0, |_, _| { unreachable!() })
                    .is_err());
                Ok(())
            })
            .unwrap();
    }
}
