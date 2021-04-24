mod example {
    use crossbeam_channel::Sender;
    use jlrs::prelude::*;

    struct MyTask {
        dims: isize,
        iters: isize,
        sender: Sender<JlrsResult<f64>>,
    }

    #[async_trait(?Send)]
    impl JuliaTask for MyTask {
        type T = f64;
        type R = Sender<JlrsResult<Self::T>>;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::T> {
            let dims = Value::new(&mut *frame, self.dims)?;
            let iters = Value::new(&mut *frame, self.iters)?;

            let v = Module::main(global)
                .submodule("MyModule")?
                .function("complexfunc")?
                .call_async(&mut *frame, &mut [dims, iters])
                .await?
                .unwrap()
                .cast::<f64>()?;

            Ok(v)
        }

        fn return_channel(&self) -> Option<&Sender<JlrsResult<Self::T>>> {
            Some(&self.sender)
        }
    }

    struct NestingTaskAsyncFrame {
        dims: isize,
        iters: isize,
        sender: Sender<JlrsResult<f64>>,
    }

    #[async_trait(?Send)]
    impl JuliaTask for NestingTaskAsyncFrame {
        type T = f64;
        type R = Sender<JlrsResult<Self::T>>;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::T> {
            let dims = Value::new(&mut *frame, self.dims)?;
            let iters = Value::new(&mut *frame, self.iters)?;

            let v = frame
                .async_scope_with_slots(1, |frame| async move {
                    Module::main(global)
                        .submodule("MyModule")?
                        .function("complexfunc")?
                        .call_async(&mut *frame, &mut [dims, iters])
                        .await?
                        .unwrap()
                        .cast::<f64>()
                })
                .await?;

            Ok(v)
        }

        fn return_channel(&self) -> Option<&Sender<JlrsResult<Self::T>>> {
            Some(&self.sender)
        }
    }

    struct NestingTaskAsyncValueFrame {
        dims: isize,
        iters: isize,
        sender: Sender<JlrsResult<f64>>,
    }

    #[async_trait(?Send)]
    impl JuliaTask for NestingTaskAsyncValueFrame {
        type T = f64;
        type R = Sender<JlrsResult<Self::T>>;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::T> {
            let v = unsafe {
                frame
                    .async_value_scope_with_slots(3, |output, frame| async move {
                        let iters = Value::new(&mut *frame, self.iters)?;
                        let dims = Value::new(&mut *frame, self.dims)?;

                        let out = Module::main(global)
                            .submodule("MyModule")?
                            .function("complexfunc")?
                            .call_async(&mut *frame, &mut [dims, iters])
                            .await?
                            .unwrap();

                        let output = output.into_scope(frame);
                        Ok(out.as_unrooted(output))
                    })
                    .await?
                    .cast::<f64>()?
            };

            Ok(v)
        }

        fn return_channel(&self) -> Option<&Sender<JlrsResult<Self::T>>> {
            Some(&self.sender)
        }
    }

    struct NestingTaskAsyncCallFrame {
        dims: isize,
        iters: isize,
        sender: Sender<JlrsResult<f64>>,
    }

    #[async_trait(?Send)]
    impl JuliaTask for NestingTaskAsyncCallFrame {
        type T = f64;
        type R = Sender<JlrsResult<Self::T>>;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::T> {
            let v = unsafe {
                frame
                    .async_result_scope_with_slots(3, |output, frame| async move {
                        let iters = Value::new(&mut *frame, self.iters)?;
                        let dims = Value::new(&mut *frame, self.dims)?;

                        let out = Module::main(global)
                            .submodule("MyModule")?
                            .function("complexfunc")?
                            .call_async(&mut *frame, &mut [dims, iters])
                            .await?;

                        let output = output.into_scope(frame);
                        Ok(out.as_unrooted(output))
                    })
                    .await?
                    .unwrap()
                    .cast::<f64>()?
            };

            Ok(v)
        }

        fn return_channel(&self) -> Option<&Sender<JlrsResult<Self::T>>> {
            Some(&self.sender)
        }
    }

    struct NestingTaskAsyncGcFrame {
        dims: isize,
        iters: isize,
        sender: Sender<JlrsResult<f64>>,
    }

    #[async_trait(?Send)]
    impl JuliaTask for NestingTaskAsyncGcFrame {
        type T = f64;
        type R = Sender<JlrsResult<Self::T>>;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::T> {
            let dims = Value::new(&mut *frame, self.dims)?;
            let iters = Value::new(&mut *frame, self.iters)?;

            let v = frame
                .async_scope(|frame| async move {
                    Module::main(global)
                        .submodule("MyModule")?
                        .function("complexfunc")?
                        .call_async(&mut *frame, &mut [dims, iters])
                        .await?
                        .unwrap()
                        .cast::<f64>()
                })
                .await?;

            Ok(v)
        }

        fn return_channel(&self) -> Option<&Sender<JlrsResult<Self::T>>> {
            Some(&self.sender)
        }
    }

    struct NestingTaskAsyncDynamicValueFrame {
        dims: isize,
        iters: isize,
        sender: Sender<JlrsResult<f64>>,
    }

    #[async_trait(?Send)]
    impl JuliaTask for NestingTaskAsyncDynamicValueFrame {
        type T = f64;
        type R = Sender<JlrsResult<Self::T>>;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::T> {
            let v = unsafe {
                frame
                    .async_value_scope(|output, frame| async move {
                        let iters = Value::new(&mut *frame, self.iters)?;
                        let dims = Value::new(&mut *frame, self.dims)?;

                        let out = Module::main(global)
                            .submodule("MyModule")?
                            .function("complexfunc")?
                            .call_async(&mut *frame, &mut [dims, iters])
                            .await?
                            .unwrap();

                        let output = output.into_scope(frame);
                        Ok(out.as_unrooted(output))
                    })
                    .await?
                    .cast::<f64>()?
            };

            Ok(v)
        }

        fn return_channel(&self) -> Option<&Sender<JlrsResult<Self::T>>> {
            Some(&self.sender)
        }
    }

    struct NestingTaskAsyncDynamicCallFrame {
        dims: isize,
        iters: isize,
        sender: Sender<JlrsResult<f64>>,
    }

    #[async_trait(?Send)]
    impl JuliaTask for NestingTaskAsyncDynamicCallFrame {
        type T = f64;
        type R = Sender<JlrsResult<Self::T>>;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::T> {
            let v = unsafe {
                frame
                    .async_result_scope(|output, frame| async move {
                        let iters = Value::new(&mut *frame, self.iters)?;
                        let dims = Value::new(&mut *frame, self.dims)?;

                        let out = Module::main(global)
                            .submodule("MyModule")?
                            .function("complexfunc")?
                            .call_async(&mut *frame, &mut [dims, iters])
                            .await?;

                        let output = output.into_scope(frame);
                        Ok(out.as_unrooted(output))
                    })
                    .await?
                    .unwrap()
                    .cast::<f64>()?
            };

            Ok(v)
        }

        fn return_channel(&self) -> Option<&Sender<JlrsResult<Self::T>>> {
            Some(&self.sender)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::cell::RefCell;

        thread_local! {
            pub static JULIA: RefCell<AsyncJulia<f64, Sender<JlrsResult<f64>>>> = {
                let r = RefCell::new(unsafe {  AsyncJulia::init(16, 1).expect("Could not init Julia").0 });
                r.borrow_mut().try_include("MyModule.jl").unwrap();
                r
            };
        }

        #[test]
        fn test_task() {
            JULIA.with(|j| {
                let julia = j.borrow_mut();

                let (sender, receiver) = crossbeam_channel::bounded(1);

                julia
                    .try_task(MyTask {
                        dims: 4,
                        iters: 5_000_000,
                        sender: sender,
                    })
                    .unwrap();

                assert_eq!(receiver.recv().unwrap().unwrap(), 20_000_004.0);
            });
        }

        #[test]
        fn test_nesting_static_task() {
            JULIA.with(|j| {
                let julia = j.borrow_mut();

                let (sender, receiver) = crossbeam_channel::bounded(1);

                julia
                    .try_task(NestingTaskAsyncFrame {
                        dims: 6,
                        iters: 5_000_000,
                        sender: sender,
                    })
                    .unwrap();

                assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
            });
        }

        #[test]
        fn test_nesting_value_static_task() {
            JULIA.with(|j| {
                let julia = j.borrow_mut();

                let (sender, receiver) = crossbeam_channel::bounded(1);

                julia
                    .try_task(NestingTaskAsyncValueFrame {
                        dims: 6,
                        iters: 5_000_000,
                        sender: sender,
                    })
                    .unwrap();

                assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
            });
        }

        #[test]
        fn test_nesting_call_static_task() {
            JULIA.with(|j| {
                let julia = j.borrow_mut();

                let (sender, receiver) = crossbeam_channel::bounded(1);

                julia
                    .try_task(NestingTaskAsyncCallFrame {
                        dims: 6,
                        iters: 5_000_000,
                        sender: sender,
                    })
                    .unwrap();

                assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
            });
        }

        #[test]
        fn test_nesting_dynamic_task() {
            JULIA.with(|j| {
                let julia = j.borrow_mut();

                let (sender, receiver) = crossbeam_channel::bounded(1);

                julia
                    .try_task(NestingTaskAsyncGcFrame {
                        dims: 6,
                        iters: 5_000_000,
                        sender: sender,
                    })
                    .unwrap();

                assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
            });
        }

        #[test]
        fn test_nesting_value_dynamic_task() {
            JULIA.with(|j| {
                let julia = j.borrow_mut();

                let (sender, receiver) = crossbeam_channel::bounded(1);

                julia
                    .try_task(NestingTaskAsyncDynamicValueFrame {
                        dims: 6,
                        iters: 5_000_000,
                        sender: sender,
                    })
                    .unwrap();

                assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
            });
        }

        #[test]
        fn test_nesting_call_dynamic_task() {
            JULIA.with(|j| {
                let julia = j.borrow_mut();

                let (sender, receiver) = crossbeam_channel::bounded(1);

                julia
                    .try_task(NestingTaskAsyncDynamicCallFrame {
                        dims: 6,
                        iters: 5_000_000,
                        sender: sender,
                    })
                    .unwrap();

                assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
            });
        }
    }
}
