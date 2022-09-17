#[cfg(feature = "sync-rt")]
mod tests {
    use super::super::super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn output_can_be_created_and_used() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let (output, frame) = frame.split();
                frame
                    .scope(|mut frame| {
                        let output = output.into_scope(&mut frame);
                        Ok(Value::new(output, 0usize))
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 1);

                let (output, frame) = frame.split();
                frame
                    .scope(|mut frame| {
                        let output = output.into_scope(&mut frame);
                        Ok(Value::new(output, 0usize))
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 2);

                let (output, frame) = frame.split();
                let _: Module = frame
                    .scope(|mut frame| {
                        let m = Module::core(frame.as_scope().global());
                        let output = output.into_scope(&mut frame);
                        unsafe { m.as_ref().root(output) }
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 3);

                let (output, frame) = frame.split();
                let _: Module = frame
                    .scope(|mut frame| {
                        let m = Module::core(global);
                        let output = output.into_scope(&mut frame);
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
            jlrs.scope(|_global, mut frame| {
                let (output, frame) = frame.split();
                frame
                    .scope(|mut frame| {
                        frame.scope(|mut frame| {
                            let output = output.into_scope(&mut frame);
                            Ok(Value::new(output, 0usize))
                        })
                    })
                    .unwrap();

                assert_eq!(frame.n_roots(), 1);

                Ok(())
            })
            .unwrap();
        })
    }
}
