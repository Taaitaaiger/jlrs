#[cfg(feature = "sync-rt")]
mod tests {
    use super::super::super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn output_can_be_created_and_used() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let (output, frame) = frame.split()?;
                frame
                    .scope(|frame| {
                        let output = output.into_scope(frame);
                        Value::new(output, 0usize)
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 1);

                let (output, frame) = frame.split()?;
                frame
                    .scope_with_capacity(0, |frame| {
                        let output = output.into_scope(frame);
                        Value::new(output, 0usize)
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 2);

                let (output, frame) = frame.split()?;
                let _: Module = frame
                    .scope(|frame| {
                        let m = Module::core(frame.global());
                        let output = output.into_scope(frame);
                        unsafe { m.as_ref().root(output) }
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 3);

                let (output, frame) = frame.split()?;
                let _: Module = frame
                    .scope_with_capacity(0, |frame| {
                        let m = Module::core(global);
                        let output = output.into_scope(frame);
                        unsafe { m.as_ref().root(output) }
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 4);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn output_can_be_propagated() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_global, frame| {
                let (output, frame) = frame.split()?;
                frame
                    .scope(|frame| {
                        frame.scope(|frame| {
                            let output = output.into_scope(frame);
                            Value::new(output, 0usize)
                        })
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 1);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn output_not_created_if_frame_full() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_global, frame| {
                while frame.n_roots() != frame.capacity() {
                    Value::new(&mut *frame, 0u8)?;
                }

                assert!(frame.split().is_err());

                Ok(())
            })
            .unwrap()
        })
    }
}
