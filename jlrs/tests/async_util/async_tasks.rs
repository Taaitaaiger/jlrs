use jlrs::{memory::gc::Gc, prelude::*};

pub struct MyTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for MyTask {
    type Output = f64;
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        // println!("MyTask");

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "complexfunc")?
                .as_managed()
                .as_value()
                .call_async(&mut frame, [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()?
        };

        // println!("MyTask done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("OtherRetTypeTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "complexfunc")?
                .as_managed()
                .as_value()
                .call_async(&mut frame, [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };

        // println!("OtherRetTypeTask done");
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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("KwTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);
        let kw = Value::new(&mut frame, 5.0f64);
        let nt = named_tuple!(&mut frame, "kw" => kw);

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "kwfunc")?
                .as_managed()
                .provide_keywords(nt)?
                .call_async(&mut frame, [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };
        // println!("KwTask done");

        Ok(v)
    }
}

pub struct ThrowingTask;

#[async_trait(?Send)]
impl AsyncTask for ThrowingTask {
    type Output = f32;
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("ThrowingTask");
        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "throwingfunc")?
                .as_managed()
                .call_async(&mut frame, [])
                .await
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
        // println!("ThrowingTask done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("NestingTaskAsyncFrame");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = frame
            .async_scope(|mut frame| async move {
                unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .as_managed()
                        .function(&frame, "complexfunc")?
                        .as_managed()
                        .as_value()
                        .call_async(&mut frame, [dims, iters])
                        .await
                        .unwrap()
                        .unbox::<f64>()
                }
            })
            .await?;

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
        // println!("NestingTaskAsyncFrame done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("NestingTaskAsyncValueFrame");
        let output = frame.output();

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
        let v = (&mut frame)
            .async_scope(|mut frame| async move {
                // println!("NestingTaskAsyncFrame");
                let iters = Value::new(&mut frame, self.iters);
                let dims = Value::new(&mut frame, self.dims);

                let out = unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .as_managed()
                        .function(&frame, "complexfunc")?
                        .as_managed()
                        .as_value()
                        .call_async(&mut frame, [dims, iters])
                        .await
                        .unwrap()
                };

                Ok(out.root(output))
            })
            .await?
            .unbox::<f64>()?;
        // println!("NestingTaskAsyncValueFrame done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("NestingTaskAsyncCallFrame");
        let output = frame.output();
        let v = frame
            .async_scope(|mut frame| async move {
                let iters = Value::new(&mut frame, self.iters);
                let dims = Value::new(&mut frame, self.dims);

                let out = unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .as_managed()
                        .function(&frame, "complexfunc")?
                        .as_managed()
                        .as_value()
                        .call_async(&mut frame, [dims, iters])
                        .await
                };

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

                let out = {
                    match out {
                        Ok(v) => Ok(v.root(output)),
                        Err(e) => Err(e.root(output)),
                    }
                };

                Ok(out)
            })
            .await?
            .unwrap()
            .unbox::<f64>()?;
        // println!("NestingTaskAsyncCallFrame done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("NestingTaskAsyncGcFrame");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

        let v = frame
            .async_scope(|mut frame| async move {
                unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .as_managed()
                        .function(&frame, "complexfunc")?
                        .as_managed()
                        .as_value()
                        .call_async(&mut frame, [dims, iters])
                        .await
                        .unwrap()
                        .unbox::<f64>()
                }
            })
            .await?;
        // println!("NestingTaskAsyncGcFrame done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("NestingTaskAsyncDynamicValueFrame");
        let output = frame.output();
        let v = frame
            .async_scope(|mut frame| async move {
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                let iters = Value::new(&mut frame, self.iters);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                let dims = Value::new(&mut frame, self.dims);

                let out = unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .as_managed()
                        .function(&frame, "complexfunc")?
                        .as_managed()
                        .as_value()
                        .call_async(&mut frame, [dims, iters])
                        .await
                        .unwrap()
                };

                Ok(out.root(output))
            })
            .await?
            .unbox::<f64>()?;
        // println!("NestingTaskAsyncDynamicValueFrame done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("NestingTaskAsyncDynamicCallFrame");
        let output = frame.output();
        let v = frame
            .async_scope(|mut frame| async move {
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                let iters = Value::new(&mut frame, self.iters);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                let dims = Value::new(&mut frame, self.dims);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

                let out = unsafe {
                    Module::main(&frame)
                        .submodule(&frame, "AsyncTests")?
                        .as_managed()
                        .function(&frame, "complexfunc")?
                        .as_managed()
                        .as_value()
                        .call_async(&mut frame, [dims, iters])
                        .await
                };
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

                let out = unsafe {
                    match out {
                        Ok(v) => Ok(v.as_ref().root(output)),
                        Err(e) => Err(e.as_ref().root(output)),
                    }
                };
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

                Ok(out)
            })
            .await?
            .unwrap()
            .unbox::<f64>()?;
        // println!("NestingTaskAsyncDynamicCallFrame done");

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
    type Affinity = DispatchAny;

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
        // println!("AccumulatorTask intit");
        unsafe {
            let output = frame.output();
            let init_value = self.init_value;
            let res = frame
                .async_scope(|mut frame| {
                    async move {
                        // A nested scope is used to only root a single value in the frame provided to
                        // init, rather than two.
                        let func = Module::main(&frame)
                            .global(&frame, "MutFloat64")?
                            .as_value();
                        let init_v = Value::new(&mut frame, init_value);

                        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

                        Ok(func.call1(output, init_v))
                    }
                })
                .await?
                .into_jlrs_result();

            // println!("AccumulatorTask intit");
            res
        }
    }

    async fn run<'frame, 'state: 'frame>(
        &mut self,
        mut frame: AsyncGcFrame<'frame>,
        state: &mut Self::State<'state>,
        input: Self::Input,
    ) -> JlrsResult<Self::Output> {
        // println!("AccumulatorTask run");
        let value = state.field_accessor().field("v")?.access::<f64>()? + input;
        let new_value = Value::new(&mut frame, value);

        unsafe {
            state
                .set_field(&mut frame, "v", new_value)?
                .into_jlrs_result()?;
        }

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
        // println!("AccumulatorTask run done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("LocalTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "complexfunc")?
                .as_managed()
                .call_async_local(&mut frame, [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
        // println!("LocalTask done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("LocalSchedulingTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "complexfunc")?
                .as_managed()
                .schedule_async_local(&mut frame, [dims, iters])
                .unwrap();

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };

        // println!("LocalSchedulingTask done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("MainTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "complexfunc")?
                .as_managed()
                .call_async_main(&mut frame, [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
        // println!("MainTask done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("MainSchedulingTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "complexfunc")?
                .as_managed()
                .schedule_async_main(&mut frame, [dims, iters])
                .unwrap();

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };
        // println!("MainSchedulingTask done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("SchedulingTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "complexfunc")?
                .as_managed()
                .schedule_async(&mut frame, [dims, iters])
                .unwrap();

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };

        // println!("SchedulingTask done");
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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("LocalKwSchedulingTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let kw = Value::new(&mut frame, 5.0f64);
            let nt = named_tuple!(&mut frame, "kw" => kw);

            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "kwfunc")?
                .as_managed()
                .provide_keywords(nt)?
                .schedule_async_local(&mut frame, [dims, iters])
                .unwrap();

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };
        // println!("LocalKwSchedulingTask done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("KwSchedulingTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let kw = Value::new(&mut frame, 5.0f64);
            let nt = named_tuple!(&mut frame, "kw" => kw);

            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "kwfunc")?
                .as_managed()
                .provide_keywords(nt)?
                .schedule_async(&mut frame, [dims, iters])
                .unwrap();

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };
        // println!("KwSchedulingTask done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("MainKwSchedulingTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        let v = unsafe {
            let kw = Value::new(&mut frame, 5.0f64);
            let nt = named_tuple!(&mut frame, "kw" => kw);

            let task = Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "kwfunc")?
                .as_managed()
                .provide_keywords(nt)?
                .schedule_async_main(&mut frame, [dims, iters])
                .unwrap();

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

            Module::base(&frame)
                .function(&mut frame, "fetch")?
                .call1(&mut frame, task.as_value())
                .into_jlrs_result()?
                .unbox::<f64>()? as f32
        };
        // println!("MainKwSchedulingTask done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("LocalKwTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);
        let kw = Value::new(&mut frame, 5.0f64);
        let nt = named_tuple!(&mut frame, "kw" => kw);

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "kwfunc")?
                .as_managed()
                .provide_keywords(nt)?
                .call_async_local(&mut frame, [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };
        // println!("LocalKwTask done");

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
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("MainKwTask");
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);
        let kw = Value::new(&mut frame, 5.0f64);
        let nt = named_tuple!(&mut frame, "kw" => kw);

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

        let v = unsafe {
            Module::main(&frame)
                .submodule(&frame, "AsyncTests")?
                .as_managed()
                .function(&frame, "kwfunc")?
                .as_managed()
                .provide_keywords(nt)?
                .call_async_main(&mut frame, [dims, iters])
                .await
                .unwrap()
                .unbox::<f64>()? as f32
        };
        // println!("MainKwTask done");

        Ok(v)
    }
}

pub struct BorrowArrayData;

#[async_trait(?Send)]
impl AsyncTask for BorrowArrayData {
    type Output = f64;
    type Affinity = DispatchAny;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // println!("BorrowArrayData");
        let mut data = vec![2.0f64];
        let borrowed = &mut data;
        let output = frame.output();
        let v = unsafe {
            frame
                .relaxed_async_scope(|_frame| async move { Array::from_slice(output, borrowed, 1) })
                .await?
                .into_jlrs_result()?
        };

        let data2 = unsafe { v.inline_data::<f64>()? };

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
        // Uncommenting next line must be compile error
        // let _ = data[0];
        let v = data2[0];
        // println!("BorrowArrayData done");
        Ok(v)
    }
}
