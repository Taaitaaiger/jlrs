use jlrs::{async_util::task::Register, memory::gc::Gc, prelude::*};

pub struct MyTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for MyTask {
    type Output = JlrsResult<f64>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

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

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        let dims = Value::new(&mut frame, self.dims);
        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

        let iters = Value::new(&mut frame, self.iters);

        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

        unsafe {
            let res = Module::main(&frame)
                .submodule(&frame, "AsyncTests")
                .unwrap()
                .as_managed()
                .function(&frame, "complexfunc")
                .unwrap()
                .as_managed()
                .as_value()
                .call_async(&mut frame, [dims, iters])
                .await
                .unwrap();

            res.unbox::<f64>().unwrap() as f32
        }
    }
}

pub struct KwTask {
    pub dims: isize,
    pub iters: isize,
}

#[async_trait(?Send)]
impl AsyncTask for KwTask {
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("KwTask {:p}", frame.stack_addr());
        let dims = Value::new(&mut frame, self.dims);
        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);
        let iters = Value::new(&mut frame, self.iters);
        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);
        let kw = Value::new(&mut frame, 5.0f64);
        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);
        let nt = named_tuple!(&mut frame, "kw" => kw);
        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

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
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("ThrowingTask {:p}", frame.stack_addr());
        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

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
    type Output = JlrsResult<f64>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("NestingTaskAsyncFrame {:p}", frame.stack_addr());
        let dims = Value::new(&mut frame, self.dims);
        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);
        let iters = Value::new(&mut frame, self.iters);
        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

        let v = frame
            .async_scope(|mut frame| async move {
                unsafe {
                    frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

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

        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);
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
    type Output = JlrsResult<f64>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("NestingTaskAsyncValueFrame {:p}", frame.stack_addr());
        let output = frame.output();

        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

        let v = (&mut frame)
            .async_scope(|mut frame| async move {
                // println!("NestingTaskAsyncFrame");
                let iters = Value::new(&mut frame, self.iters);

                frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);
                let dims = Value::new(&mut frame, self.dims);

                frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

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

                frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

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
    type Output = JlrsResult<f64>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("NestingTaskAsyncCallFrame {:p}", frame.stack_addr());
        let output = frame.output();
        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

        let v = frame
            .async_scope(|mut frame| async move {
                let iters = Value::new(&mut frame, self.iters);
                frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);
                let dims = Value::new(&mut frame, self.dims);
                frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

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

                frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

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
    type Output = JlrsResult<f64>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("NestingTaskAsyncGcFrame {:p}", frame.stack_addr());
        let dims = Value::new(&mut frame, self.dims);
        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);
        let iters = Value::new(&mut frame, self.iters);

        frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

        let v = frame
            .async_scope(|mut frame| async move {
                unsafe {
                    frame.gc_collect_n(jlrs::memory::gc::GcCollection::Full, 3);

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
    type Output = JlrsResult<f64>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("NestingTaskAsyncDynamicValueFrame {:p}", frame.stack_addr());
        let output = frame.output();
        let v = frame
            .async_scope(|mut frame| async move {
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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

                // println!("Root {out:?}");
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
    type Output = JlrsResult<f64>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("NestingTaskAsyncDynamicCallFrame {:p}", frame.stack_addr());
        let output = frame.output();
        let v = frame
            .async_scope(|mut frame| async move {
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                let iters = Value::new(&mut frame, self.iters);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                let dims = Value::new(&mut frame, self.dims);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);

                let out = unsafe {
                    match out {
                        Ok(v) => Ok(v.as_ref().root(output)),
                        Err(e) => Err(e.as_ref().root(output)),
                    }
                };
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
impl Register for AccumulatorTask {
    async fn register<'frame>(mut frame: AsyncGcFrame<'frame>) -> JlrsResult<()> {
        unsafe {
            Value::eval_string(&mut frame, "mutable struct MutFloat64 v::Float64 end")
                .into_jlrs_result()?;
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl PersistentTask for AccumulatorTask {
    type State<'state> = Value<'state, 'static>;
    type Input = f64;
    type Output = JlrsResult<f64>;

    const CHANNEL_CAPACITY: usize = 2;

    async fn init<'frame>(
        &mut self,
        mut frame: AsyncGcFrame<'frame>,
    ) -> JlrsResult<Value<'frame, 'static>> {
        // println!("AccumulatorTask init {:p}", frame.stack_addr());
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

            // println!("AccumulatorTask init");
            res
        }
    }

    async fn run<'frame, 'state: 'frame>(
        &mut self,
        mut frame: AsyncGcFrame<'frame>,
        state: &mut Self::State<'state>,
        input: Self::Input,
    ) -> Self::Output {
        // println!("AccumulatorTask run {:p}", frame.stack_addr());
        let value = state.field_accessor().field("v")?.access::<f64>()? + input;
        let new_value = Value::new(&mut frame, value);

        unsafe {
            state
                .set_field(&mut frame, "v", new_value)?
                .into_jlrs_result()?;
        }

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("LocalTask {:p}", frame.stack_addr());
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
        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("LocalSchedulingTask {:p}", frame.stack_addr());
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
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("MainTask {:p}", frame.stack_addr());
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
        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("MainSchedulingTask {:p}", frame.stack_addr());
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
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("SchedulingTask {:p}", frame.stack_addr());
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
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("LocalKwSchedulingTask {:p}", frame.stack_addr());
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
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("KwSchedulingTask {:p}", frame.stack_addr());
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
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("MainKwSchedulingTask {:p}", frame.stack_addr());
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
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("LocalKwTask {:p}", frame.stack_addr());
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);
        let kw = Value::new(&mut frame, 5.0f64);
        let nt = named_tuple!(&mut frame, "kw" => kw);

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
    type Output = JlrsResult<f32>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("MainKwTask {:p}", frame.stack_addr());
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);
        let kw = Value::new(&mut frame, 5.0f64);
        let nt = named_tuple!(&mut frame, "kw" => kw);

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
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
    type Output = JlrsResult<f64>;

    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // println!("BorrowArrayData {:p}", frame.stack_addr());
        let mut data = vec![2.0f64];
        let borrowed = &mut data;
        let output = frame.output();
        let v = unsafe {
            frame
                .relaxed_async_scope(|_frame| async move {
                    TypedArray::<f64>::from_slice(output, borrowed, 1)
                })
                .await?
                .into_jlrs_result()?
        };

        let data2 = unsafe { v.inline_data() };

        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
        // Uncommenting next line must be compile error
        // let _ = data[0];
        let v = data2[0];
        // println!("BorrowArrayData done");
        Ok(v)
    }
}
