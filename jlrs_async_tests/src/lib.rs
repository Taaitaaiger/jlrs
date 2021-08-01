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

    struct AccGeneratorTask {
        start: usize,
    }

    #[async_trait(?Send)]
    impl GeneratorTask for AccGeneratorTask {
        type InitData = UnborrowedValue;
        type CallData = usize;
        type Output = usize;

        fn init(
            &mut self,
            global: Global<'static>,
            frame: &mut GcFrame<'static, Async<'static>>,
        ) -> JlrsResult<Self::InitData> {
            unsafe {
                Value::eval_string(&mut *frame, "mutable struct MutUInt v::UInt end")
                    .unwrap()
                    .unwrap();

                let init_v = frame
                    .result_scope_with_slots(1, |output, frame| {
                        let func = Module::main(global)
                            .global_ref("MutUInt")
                            .unwrap()
                            .value_unchecked();
                        let init_v = Value::new(&mut *frame, self.start)?;

                        let os = output.into_scope(frame);

                        func.call1(os, init_v)
                    })?
                    .unwrap();

                Ok(init_v.as_unborrowed())
            }
        }

        async fn run<'nested>(
            &mut self,
            _global: Global<'nested>,
            frame: &mut AsyncGcFrame<'nested>,
            init_data: Self::InitData,
            call_data: Self::CallData,
        ) -> JlrsResult<Self::Output> {
            let acc = init_data.as_value();
            let value = acc.get_raw_field::<usize, _>("v")? + call_data;
            let jlvalue = Value::new(&mut *frame, value)?;

            unsafe {
                acc.set_field(frame, "v", jlvalue)?.into_jlrs_result()?;
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

                let (sender, receiver) = crossbeam_channel::bounded(1);

                let handle = julia.try_generator(AccGeneratorTask { start: 5 }).unwrap();

                handle.try_call(7, sender.clone()).unwrap();
                assert_eq!(receiver.recv().unwrap().unwrap(), 12);

                handle.try_call(12, sender).unwrap();
                assert_eq!(receiver.recv().unwrap().unwrap(), 24);
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
