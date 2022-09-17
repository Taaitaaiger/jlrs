#[cfg(feature = "sync-rt")]
mod tests {
    use super::super::super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn frame_starts_with_no_roots() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_global, frame| {
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
            jlrs.scope(|_global, mut frame| {
                Value::new(&mut frame, 0usize);
                assert_eq!(frame.n_roots(), 1);
                Ok(())
            })
            .unwrap();
        })
    }


    /*
    #[test]
    #[cfg(feature = "ccall")]
    fn create_null_frame() {
        let mut ccall = unsafe { CCall::new() };
        ccall.null_scope(|_mut frame| Ok(())).unwrap();
    }

    #[test]
    #[cfg(feature = "ccall")]
    fn null_frame_is_empty() {
        let mut ccall = unsafe { CCall::new() };

        ccall
            .null_scope(|mut frame| {
                assert_eq!(frame.n_roots(), 0);
                assert_eq!(frame.capacity(), 0);
                assert!(Value::new(&mut frame, 0usize).is_err());
                Ok(())
            })
            .unwrap();
    }

    #[test]
    #[cfg(feature = "ccall")]
    fn cannot_nest_null_frame() {
        let mut ccall = unsafe { CCall::new() };

        ccall
            .null_scope(|mut frame| {
                assert!(frame.scope(|_| { Ok(()) }).is_err());
                Ok(())
            })
            .unwrap();

        ccall
            .null_scope(|mut frame| {
                assert!(frame.scope(|_| { Ok(()) }).is_err());
                Ok(())
            })
            .unwrap();

        ccall
            .null_scope(|mut frame| {
                assert!(frame.value_scope(|_, _| { unreachable!() }).is_err());
                Ok(())
            })
            .unwrap();

        ccall
            .null_scope(|mut frame| {
                assert!(frame
                    .value_scope_with_slots(0, |_, _| { unreachable!() })
                    .is_err());
                Ok(())
            })
            .unwrap();

        ccall
            .null_scope(|mut frame| {
                assert!(frame.result_scope(|_, _| { unreachable!() }).is_err());
                Ok(())
            })
            .unwrap();

        ccall
            .null_scope(|mut frame| {
                assert!(frame
                    .result_scope_with_slots(0, |_, _| { unreachable!() })
                    .is_err());
                Ok(())
            })
            .unwrap();
    }
     */
}
