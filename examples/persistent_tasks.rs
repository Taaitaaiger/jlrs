use jlrs::prelude::*;

// This struct contains the data our task will need. This struct must implement `Send`, `Sync`,
// and contain no borrowed data.
struct AccumulatorTask {
    init_value: f64,
}

// `Implement `PersistentTask` for `AccumulatorTask`. This requires `async_trait` because traits
// with async methods are not yet available in Rust. Because the task itself is not thread-safe it
// is marked with `?Send`.
#[async_trait(?Send)]
impl PersistentTask for AccumulatorTask {
    // State is the type of data that PersistentTask::init returns. The frame provided to
    // PersistentTask::init isn't dropped until the task is dropped so the state can contain
    // Julia data rooted in that frame. When PersistentTask::run is called it can use a mutable
    // reference to this data.
    type State<'state> = Value<'state, 'static>;

    // Input is the type of data that must be provided when the task's handle is used to
    // call it. Like State, it's provided to PersistentTask::run. This tasks expects an f64.
    type Input = f64;

    // Output is the type of data that PersistentTask::run returns if it completes successfully.
    // This result is returned to the caller through a channel.
    type Output = f64;

    // The capacity of the channel used to communicate with this task
    const CHANNEL_CAPACITY: usize = 2;

    // Register this task. This method can take care of custom initialization work, in this case
    // creating the mutable MutFloat64 type in the Main module.
    async fn register<'frame>(mut frame: AsyncGcFrame<'frame>) -> JlrsResult<()> {
        unsafe {
            Value::eval_string(&mut frame, "mutable struct MutFloat64 v::Float64 end")
                .into_jlrs_result()?;
        }
        Ok(())
    }

    // Initialize the task. Because the frame is not dropped until all handles to the task
    // have been dropped and every pending call has completed, Julia data rooted in this frame
    // can be returned as State. Here, the value we'll use as an accumulator is created and
    // returned.
    async fn init<'frame>(
        &mut self,
        mut frame: AsyncGcFrame<'frame>,
    ) -> JlrsResult<Self::State<'frame>> {
        unsafe {
            let output = frame.output();
            frame
                .scope(|mut frame| {
                    // A nested scope is used to only root a single value in the frame provided to
                    // init, rather than two.
                    let func = Module::main(&frame).global(&frame, "MutFloat64")?.value();
                    let init_v = Value::new(&mut frame, self.init_value);

                    Ok(func.call1(output, init_v))
                })?
                .into_jlrs_result()
        }
    }

    // Call the task once. Note that while the state can be mutated, you can't replace any
    // Julia data that it contains with newly allocated data because it's called in a nested
    // scope.
    async fn run<'frame, 'state: 'frame>(
        &mut self,
        mut frame: AsyncGcFrame<'frame>,
        state: &mut Self::State<'state>,
        input: Self::Input,
    ) -> JlrsResult<Self::Output> {
        // Add call_cata to the accumulator and return its new value. The accumulator is mutable
        // Julia data so its contents can be changed.
        let value = state.field_accessor().field("v")?.access::<f64>()? + input;
        let new_value = Value::new(&mut frame, value);

        unsafe {
            state.set_field_unchecked("v", new_value)?;
        }

        Ok(value)
    }
}

fn main() {
    // The first thing we need to do is initialize the async runtime. In this example tokio is
    // used as backing runtime.
    //
    // Afterwards we have an instance of `AsyncJulia` that can be used to interact with the
    // runtime, and a handle to the thread where the runtime is running.
    let (julia, handle) = unsafe {
        RuntimeBuilder::new()
            .async_runtime::<Tokio>()
            .start::<1>()
            .expect("Could not init Julia")
    };

    {
        // Register AccumulatorTask, otherwise AccumulatorTask::init returns an error.
        let (init_sender, init_receiver) = crossbeam_channel::bounded(1);
        julia
            .try_register_persistent::<AccumulatorTask, _>(init_sender)
            .unwrap();
        init_receiver.recv().unwrap().unwrap();
    }

    // Create a new AccumulatorTask, if AccumulatorTask::init completes successfully a handle to
    // the task is returned.
    let persistent = {
        let (handle_sender, handle_receiver) = crossbeam_channel::bounded(1);
        julia
            .try_persistent::<UnboundedChannel<_>, _, _>(
                AccumulatorTask { init_value: 5.0 },
                handle_sender,
            )
            .expect("Cannot send task");

        handle_receiver
            .recv()
            .expect("Channel was closed")
            .expect("Cannot init task")
    };

    // Call the task twice. Because AccumulatorTask::Input is f64, that data must be
    // provided here.
    let (sender1, receiver1) = crossbeam_channel::bounded(1);
    persistent.try_call(5.0, sender1).unwrap();
    let (sender2, receiver2) = crossbeam_channel::bounded(1);
    persistent.try_call(10.0, sender2).unwrap();

    // Receive the results of the tasks.
    let x = receiver1.recv().unwrap().unwrap();
    println!("Result of first task: {}", x);

    let y = receiver2.recv().unwrap().unwrap();
    println!("Result of second task: {}", y);

    // Dropping the task and `julia` causes the runtime to shut down Julia and itself. Join
    // the handle to wait for everything to shut down cleanly.
    std::mem::drop(persistent);
    std::mem::drop(julia);
    handle
        .join()
        .expect("Could not await the task")
        .expect("Julia exited with an error");
}
