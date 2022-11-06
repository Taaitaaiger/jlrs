use jlrs::prelude::*;

pub struct MyTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for MyTask {
    type Output = f64;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "complexfunc")?
                .wrapper_unchecked()
                .as_value()
                .call_async(&mut frame, &mut [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()?
        };

        Ok(v)
    }
}

pub struct OtherRetTypeTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for OtherRetTypeTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "complexfunc")?
                .wrapper_unchecked()
                .as_value()
                .call_async(&mut frame, &mut [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct KwTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for KwTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);
        let kw = Value::new(&mut frame, 5.0f64);
        let nt = named_tuple!(frame.as_extended_target(), "kw" => kw);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "kwfunc")?
                .wrapper_unchecked()
                .provide_keywords(nt)?
                .call_async(&mut frame, &mut [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct ThrowingTask;

#[async_trait(?Send)]
impl AsyncTask for ThrowingTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "throwingfunc")?
                .wrapper_unchecked()
                .call_async(&mut frame, [])
                .await
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct NestingTaskAsyncFrame {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for NestingTaskAsyncFrame {
    type Output = f64;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = frame
            .async_scope(|mut frame| async move {
                unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .wrapper_unchecked()
                        .function(&frame, "complexfunc")?
                        .wrapper_unchecked()
                        .as_value()
                        .call_async(&mut frame, &mut [dims, iters])
                        .await
                        .unwrap()
                        .unbox::<f64>()
                }
            })
            .await?;

        Ok(v)
    }
}

pub struct NestingTaskAsyncValueFrame {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for NestingTaskAsyncValueFrame {
    type Output = f64;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let output = frame.output();
        let v = (&mut frame)
            .async_scope(|mut frame| async move {
                let iters = Value::new(&mut frame, self.iters);
                let dims = Value::new(&mut frame, self.dims);

                let out = unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .wrapper_unchecked()
                        .function(&frame, "complexfunc")?
                        .wrapper_unchecked()
                        .as_value()
                        .call_async(&mut frame, &mut [dims, iters])
                        .await
                        .unwrap()
                };

                Ok(out.root(output))
            })
            .await?
            .unbox::<f64>()?;

        Ok(v)
    }
}

pub struct NestingTaskAsyncCallFrame {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for NestingTaskAsyncCallFrame {
    type Output = f64;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let output = frame.output();
        let v = frame
            .async_scope(|mut frame| async move {
                let iters = Value::new(&mut frame, self.iters);
                let dims = Value::new(&mut frame, self.dims);

                let out = unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .wrapper_unchecked()
                        .function(&frame, "complexfunc")?
                        .wrapper_unchecked()
                        .as_value()
                        .call_async(&mut frame, &mut [dims, iters])
                        .await
                };

                let out = unsafe {
                    match out {
                        Ok(v) => Ok(v.as_ref().root(output)?),
                        Err(e) => Err(e.as_ref().root(output)?),
                    }
                };

                Ok(out)
            })
            .await?
            .unwrap()
            .unbox::<f64>()?;

        Ok(v)
    }
}

pub struct NestingTaskAsyncGcFrame {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for NestingTaskAsyncGcFrame {
    type Output = f64;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = frame
            .async_scope(|mut frame| async move {
                unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .wrapper_unchecked()
                        .function(&frame, "complexfunc")?
                        .wrapper_unchecked()
                        .as_value()
                        .call_async(&mut frame, &mut [dims, iters])
                        .await
                        .unwrap()
                        .unbox::<f64>()
                }
            })
            .await?;

        Ok(v)
    }
}

pub struct NestingTaskAsyncDynamicValueFrame {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for NestingTaskAsyncDynamicValueFrame {
    type Output = f64;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let output = frame.output();
        let v = frame
            .async_scope(|mut frame| async move {
                let iters = Value::new(&mut frame, self.iters);
                let dims = Value::new(&mut frame, self.dims);

                let out = unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .wrapper_unchecked()
                        .function(&frame, "complexfunc")?
                        .wrapper_unchecked()
                        .as_value()
                        .call_async(&mut frame, &mut [dims, iters])
                        .await
                        .unwrap()
                };

                Ok(out.root(output))
            })
            .await?
            .unbox::<f64>()?;

        Ok(v)
    }
}

pub struct NestingTaskAsyncDynamicCallFrame {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for NestingTaskAsyncDynamicCallFrame {
    type Output = f64;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let output = frame.output();
        let v = frame
            .async_scope(|mut frame| async move {
                let iters = Value::new(&mut frame, self.iters);
                let dims = Value::new(&mut frame, self.dims);

                let out = unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .wrapper_unchecked()
                        .function(&frame, "complexfunc")?
                        .wrapper_unchecked()
                        .as_value()
                        .call_async(&mut frame, &mut [dims, iters])
                        .await
                };

                let out = unsafe {
                    match out {
                        Ok(v) => Ok(v.as_ref().root(output)?),
                        Err(e) => Err(e.as_ref().root(output)?),
                    }
                };

                Ok(out)
            })
            .await?
            .unwrap()
            .unbox::<f64>()?;

        Ok(v)
    }
}

pub struct AccumulatorTask {
    pub init_value: f64,
}

#[async_trait(?Send)]
impl PersistentTask for AccumulatorTask {
    type State<'state> = Value<'state, 'static>;
    type Input = f64;
    type Output = f64;

    const CHANNEL_CAPACITY: usize = 2;

    async fn register<'frame>(mut frame: AsyncGcFrame<'frame>) -> JlrsResult<()> {
        unsafe {
            Value::eval_string(&mut frame, "mutable struct MutFloat64 v::Float64 end")
                .into_jlrs_result()?;
        }
        Ok(())
    }

    async fn init<'frame>(
        &mut self,
        mut frame: AsyncGcFrame<'frame>,
    ) -> JlrsResult<Value<'frame, 'static>> {
        unsafe {
            let output = frame.output();
            let init_value = self.init_value;
            frame
                .async_scope(|mut frame| {
                    async move {
                        // A nested scope is used to only root a single value in the frame provided to
                        // init, rather than two.
                        let func = Module::main(&frame)
                            .global(&frame, "MutFloat64")?
                            .value_unchecked();
                        let init_v = Value::new(&mut frame, init_value);

                        Ok(func.call1(output, init_v))
                    }
                })
                .await?
                .into_jlrs_result()
        }
    }

    async fn run<'frame, 'state: 'frame>(
        &mut self,
        mut frame: AsyncGcFrame<'frame>,
        state: &mut Self::State<'state>,
        input: Self::Input,
    ) -> JlrsResult<Self::Output> {
        let value = state.field_accessor(&frame).field("v")?.access::<f64>()? + input;
        let new_value = Value::new(&mut frame, value);

        unsafe {
            state
                .set_field(&mut frame, "v", new_value)?
                .into_jlrs_result()?;
        }

        Ok(value)
    }
}

pub struct LocalTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for LocalTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "complexfunc")?
                .wrapper_unchecked()
                .call_async_local(&mut frame, &mut [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct LocalSchedulingTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for LocalSchedulingTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "complexfunc")?
                .wrapper_unchecked()
                .schedule_async_local(&mut frame, &mut [dims, iters])
                .unwrap();

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct MainTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for MainTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "complexfunc")?
                .wrapper_unchecked()
                .call_async_main(&mut frame, &mut [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct MainSchedulingTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for MainSchedulingTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "complexfunc")?
                .wrapper_unchecked()
                .schedule_async_main(&mut frame, &mut [dims, iters])
                .unwrap();

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct SchedulingTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for SchedulingTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "complexfunc")?
                .wrapper_unchecked()
                .schedule_async(&mut frame, &mut [dims, iters])
                .unwrap();

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct LocalKwSchedulingTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for LocalKwSchedulingTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let kw = Value::new(&mut frame, 5.0f64);
            let nt = named_tuple!(frame.as_extended_target(), "kw" => kw);

            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "kwfunc")?
                .wrapper_unchecked()
                .provide_keywords(nt)?
                .schedule_async_local(&mut frame, &mut [dims, iters])
                .unwrap();

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct KwSchedulingTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for KwSchedulingTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let kw = Value::new(&mut frame, 5.0f64);
            let nt = named_tuple!(frame.as_extended_target(), "kw" => kw);

            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "kwfunc")?
                .wrapper_unchecked()
                .provide_keywords(nt)?
                .schedule_async(&mut frame, &mut [dims, iters])
                .unwrap();

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct MainKwSchedulingTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for MainKwSchedulingTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let kw = Value::new(&mut frame, 5.0f64);
            let nt = named_tuple!(frame.as_extended_target(), "kw" => kw);

            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "kwfunc")?
                .wrapper_unchecked()
                .provide_keywords(nt)?
                .schedule_async_main(&mut frame, &mut [dims, iters])
                .unwrap();

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct LocalKwTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for LocalKwTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);
        let kw = Value::new(&mut frame, 5.0f64);
        let nt = named_tuple!(frame.as_extended_target(), "kw" => kw);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "kwfunc")?
                .wrapper_unchecked()
                .provide_keywords(nt)?
                .call_async_local(&mut frame, &mut [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct MainKwTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for MainKwTask {
    type Output = f32;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);
        let kw = Value::new(&mut frame, 5.0f64);
        let nt = named_tuple!(frame.as_extended_target(), "kw" => kw);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .wrapper_unchecked()
                .function(&frame, "kwfunc")?
                .wrapper_unchecked()
                .provide_keywords(nt)?
                .call_async_main(&mut frame, &mut [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };

        Ok(v)
    }
}

pub struct BorrowArrayData;

#[async_trait(?Send)]
impl AsyncTask for BorrowArrayData {
    type Output = f64;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let mut data = vec![2.0f64];
        let borrowed = &mut data;
        let output = frame.output();
        let v = unsafe {
            frame
                .relaxed_async_scope(|mut frame| async move {
                    let output_scope = output.into_extended_target(&mut frame);
                    Array::from_slice(output_scope, borrowed, 1)
                })
                .await?
                .into_jlrs_result()?
        };

        let data2 = unsafe { v.inline_data::<f64>()? };
        // Uncommenting next line must be compile error
        // let _ = data[0];
        let v = data2[0];
        Ok(v)
    }
}
