mod example {
    use jlrs::prelude::*;

    struct MyTask {
        dims: isize,
        iters: isize,
    }

    #[async_trait(?Send)]
    impl AsyncTask for MyTask {
        type Output = f64;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::Output> {
            let dims = Value::new(&mut *frame, self.dims)?;
            let iters = Value::new(&mut *frame, self.iters)?;

            let v = unsafe {
                Module::main(global)
                    .submodule_ref("MyModule")?
                    .wrapper_unchecked()
                    .function_ref("complexfunc")?
                    .wrapper_unchecked()
                    .as_value()
                    .call_async(&mut *frame, &mut [dims, iters])
                    .await?
                    .unwrap()
                    .unbox::<f64>()?
            };

            Ok(v)
        }
    }

    struct OtherRetTypeTask {
        dims: isize,
        iters: isize,
    }

    #[async_trait(?Send)]
    impl AsyncTask for OtherRetTypeTask {
        type Output = f32;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::Output> {
            let dims = Value::new(&mut *frame, self.dims)?;
            let iters = Value::new(&mut *frame, self.iters)?;

            let v = unsafe {
                Module::main(global)
                    .submodule_ref("MyModule")?
                    .wrapper_unchecked()
                    .function_ref("complexfunc")?
                    .wrapper_unchecked()
                    .as_value()
                    .call_async(&mut *frame, &mut [dims, iters])
                    .await?
                    .unwrap()
                    .unbox::<f64>()? as f32
            };

            Ok(v)
        }
    }

    struct NestingTaskAsyncFrame {
        dims: isize,
        iters: isize,
    }

    #[async_trait(?Send)]
    impl AsyncTask for NestingTaskAsyncFrame {
        type Output = f64;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::Output> {
            let dims = Value::new(&mut *frame, self.dims)?;
            let iters = Value::new(&mut *frame, self.iters)?;

            let v = frame
                .async_scope_with_slots(1, |frame| async move {
                    unsafe {
                        Module::main(global)
                            .submodule_ref("MyModule")?
                            .wrapper_unchecked()
                            .function_ref("complexfunc")?
                            .wrapper_unchecked()
                            .as_value()
                            .call_async(&mut *frame, &mut [dims, iters])
                            .await?
                            .unwrap()
                            .unbox::<f64>()
                    }
                })
                .await?;

            Ok(v)
        }
    }

    struct NestingTaskAsyncValueFrame {
        dims: isize,
        iters: isize,
    }

    #[async_trait(?Send)]
    impl AsyncTask for NestingTaskAsyncValueFrame {
        type Output = f64;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::Output> {
            let v = frame
                .async_value_scope_with_slots(3, |output, frame| async move {
                    let iters = Value::new(&mut *frame, self.iters)?;
                    let dims = Value::new(&mut *frame, self.dims)?;

                    let out = unsafe {
                        Module::main(global)
                            .submodule_ref("MyModule")?
                            .wrapper_unchecked()
                            .function_ref("complexfunc")?
                            .wrapper_unchecked()
                            .as_value()
                            .call_async(&mut *frame, &mut [dims, iters])
                            .await?
                            .unwrap()
                    };

                    let output = output.into_scope(frame);
                    Ok(out.as_unrooted(output))
                })
                .await?
                .unbox::<f64>()?;

            Ok(v)
        }
    }

    struct NestingTaskAsyncCallFrame {
        dims: isize,
        iters: isize,
    }

    #[async_trait(?Send)]
    impl AsyncTask for NestingTaskAsyncCallFrame {
        type Output = f64;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::Output> {
            let v = frame
                .async_result_scope_with_slots(3, |output, frame| async move {
                    let iters = Value::new(&mut *frame, self.iters)?;
                    let dims = Value::new(&mut *frame, self.dims)?;

                    let out = unsafe {
                        Module::main(global)
                            .submodule_ref("MyModule")?
                            .wrapper_unchecked()
                            .function_ref("complexfunc")?
                            .wrapper_unchecked()
                            .as_value()
                            .call_async(&mut *frame, &mut [dims, iters])
                            .await?
                    };

                    let output = output.into_scope(frame);
                    Ok(out.as_unrooted(output))
                })
                .await?
                .unwrap()
                .unbox::<f64>()?;

            Ok(v)
        }
    }

    struct NestingTaskAsyncGcFrame {
        dims: isize,
        iters: isize,
    }

    #[async_trait(?Send)]
    impl AsyncTask for NestingTaskAsyncGcFrame {
        type Output = f64;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::Output> {
            let dims = Value::new(&mut *frame, self.dims)?;
            let iters = Value::new(&mut *frame, self.iters)?;

            let v = frame
                .async_scope(|frame| async move {
                    unsafe {
                        Module::main(global)
                            .submodule_ref("MyModule")?
                            .wrapper_unchecked()
                            .function_ref("complexfunc")?
                            .wrapper_unchecked()
                            .as_value()
                            .call_async(&mut *frame, &mut [dims, iters])
                            .await?
                            .unwrap()
                            .unbox::<f64>()
                    }
                })
                .await?;

            Ok(v)
        }
    }

    struct NestingTaskAsyncDynamicValueFrame {
        dims: isize,
        iters: isize,
    }

    #[async_trait(?Send)]
    impl AsyncTask for NestingTaskAsyncDynamicValueFrame {
        type Output = f64;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::Output> {
            let v = frame
                .async_value_scope(|output, frame| async move {
                    let iters = Value::new(&mut *frame, self.iters)?;
                    let dims = Value::new(&mut *frame, self.dims)?;

                    let out = unsafe {
                        Module::main(global)
                            .submodule_ref("MyModule")?
                            .wrapper_unchecked()
                            .function_ref("complexfunc")?
                            .wrapper_unchecked()
                            .as_value()
                            .call_async(&mut *frame, &mut [dims, iters])
                            .await?
                            .unwrap()
                    };

                    let output = output.into_scope(frame);
                    Ok(out.as_unrooted(output))
                })
                .await?
                .unbox::<f64>()?;

            Ok(v)
        }
    }

    struct NestingTaskAsyncDynamicCallFrame {
        dims: isize,
        iters: isize,
    }

    #[async_trait(?Send)]
    impl AsyncTask for NestingTaskAsyncDynamicCallFrame {
        type Output = f64;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncGcFrame<'base>,
        ) -> JlrsResult<Self::Output> {
            let v = frame
                .async_result_scope(|output, frame| async move {
                    let iters = Value::new(&mut *frame, self.iters)?;
                    let dims = Value::new(&mut *frame, self.dims)?;

                    let out = unsafe {
                        Module::main(global)
                            .submodule_ref("MyModule")?
                            .wrapper_unchecked()
                            .function_ref("complexfunc")?
                            .wrapper_unchecked()
                            .as_value()
                            .call_async(&mut *frame, &mut [dims, iters])
                            .await?
                    };

                    let output = output.into_scope(frame);
                    Ok(out.as_unrooted(output))
                })
                .await?
                .unwrap()
                .unbox::<f64>()?;

            Ok(v)
        }
    }

    struct AccumulatorTask {
        init_value: f64,
    }

    #[async_trait(?Send)]
    impl GeneratorTask for AccumulatorTask {
        type State = Value<'static, 'static>;
        type Input = f64;
        type Output = f64;

        const REGISTER_SLOTS: usize = 1;
        const INIT_SLOTS: usize = 1;
        const RUN_SLOTS: usize = 1;
        const CHANNEL_CAPACITY: usize = 2;

        async fn register<'frame>(
            _global: Global<'frame>,
            frame: &mut AsyncGcFrame<'frame>,
        ) -> JlrsResult<()> {
            unsafe {
                Value::eval_string(&mut *frame, "mutable struct MutFloat64 v::Float64 end")?
                    .into_jlrs_result()?;
            }
            Ok(())
        }

        async fn init<'inner>(
            &'inner mut self,
            global: Global<'static>,
            frame: &'inner mut AsyncGcFrame<'static>,
        ) -> JlrsResult<Value<'static, 'static>> {
            unsafe {
                frame
                    .result_scope_with_slots(1, |output, frame| {
                        // A nested scope is used to only root a single value in the frame provided to
                        // init, rather than two.
                        let func = Module::main(global)
                            .global_ref("MutFloat64")?
                            .value_unchecked();
                        let init_v = Value::new(&mut *frame, self.init_value)?;

                        let os = output.into_scope(frame);

                        func.call1(os, init_v)
                    })?
                    .into_jlrs_result()
            }
        }

        async fn run<'inner, 'frame>(
            &'inner mut self,
            _global: Global<'frame>,
            frame: &'inner mut AsyncGcFrame<'frame>,
            state: &'inner mut Self::State,
            input: Self::Input,
        ) -> JlrsResult<Self::Output> {
            let value = state.get_raw_field::<f64, _>("v")? + input;
            let new_value = Value::new(&mut *frame, value)?;

            unsafe {
                state.set_field(frame, "v", new_value)?.into_jlrs_result()?;
            }

            Ok(value)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::cell::RefCell;

        thread_local! {
            pub static JULIA: RefCell<AsyncJulia> = {
                let r = RefCell::new(unsafe {  AsyncJulia::init(4, 16, 1).expect("Could not init Julia").0 });
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
                    .try_task(
                        MyTask {
                            dims: 4,
                            iters: 5_000_000,
                        },
                        sender,
                    )
                    .unwrap();

                assert_eq!(receiver.recv().unwrap().unwrap(), 20_000_004.0);
            });
        }

        #[test]
        fn test_generator() {
            JULIA.with(|j| {
                let julia = j.borrow_mut();

                let (is, ir) = crossbeam_channel::bounded(1);
                julia
                    .try_register_generator::<AccumulatorTask, _>(is)
                    .unwrap();
                ir.recv().unwrap().unwrap();

                let (sender, receiver) = crossbeam_channel::bounded(1);

                let handle = julia
                    .try_generator(AccumulatorTask { init_value: 5.0 })
                    .unwrap();

                handle.try_call(7.0, sender.clone()).unwrap();
                assert_eq!(receiver.recv().unwrap().unwrap(), 12.0);

                handle.try_call(12.0, sender).unwrap();
                assert_eq!(receiver.recv().unwrap().unwrap(), 24.0);
            });
        }

        #[test]
        fn test_other_ret_type_task() {
            JULIA.with(|j| {
                let julia = j.borrow_mut();

                let (sender, receiver) = crossbeam_channel::bounded(1);

                julia
                    .try_task(
                        OtherRetTypeTask {
                            dims: 4,
                            iters: 5_000_000,
                        },
                        sender,
                    )
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
                    .try_task(
                        NestingTaskAsyncFrame {
                            dims: 6,
                            iters: 5_000_000,
                        },
                        sender,
                    )
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
                    .try_task(
                        NestingTaskAsyncValueFrame {
                            dims: 6,
                            iters: 5_000_000,
                        },
                        sender,
                    )
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
                    .try_task(
                        NestingTaskAsyncCallFrame {
                            dims: 6,
                            iters: 5_000_000,
                        },
                        sender,
                    )
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
                    .try_task(
                        NestingTaskAsyncGcFrame {
                            dims: 6,
                            iters: 5_000_000,
                        },
                        sender,
                    )
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
                    .try_task(
                        NestingTaskAsyncDynamicValueFrame {
                            dims: 6,
                            iters: 5_000_000,
                        },
                        sender,
                    )
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
                    .try_task(
                        NestingTaskAsyncDynamicCallFrame {
                            dims: 6,
                            iters: 5_000_000,
                        },
                        sender,
                    )
                    .unwrap();

                assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
            });
        }
    }
}
