mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn local_scope() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let out = stack.returning::<JlrsResult<_>>().scope(|mut frame| {
                    let output = frame.output();

                    frame
                        .local_scope::<1>(|mut frame| {
                            assert_eq!(frame.frame_size(), 1);
                            assert_eq!(frame.n_roots(), 0);
                            let v = Value::new(&mut frame, 1usize);
                            assert_eq!(frame.n_roots(), 1);
                            v.root(output)
                        })
                        .unbox::<usize>()
                });

                assert_eq!(out.unwrap(), 1);
            });
        });
    }

    fn unsized_local_scope() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let out = stack.returning::<JlrsResult<_>>().scope(|mut frame| {
                    let output = frame.output();

                    frame
                        .unsized_local_scope(1, |mut frame| {
                            assert_eq!(frame.frame_size(), 1);
                            assert_eq!(frame.n_roots(), 0);
                            let v = Value::new(&mut frame, 1usize);
                            assert_eq!(frame.n_roots(), 1);
                            v.root(output)
                        })
                        .unbox::<usize>()
                });

                assert_eq!(out.unwrap(), 1);
            });
        });
    }

    #[test]
    fn output_frame_tests() {
        local_scope();
        unsized_local_scope();
    }
}
