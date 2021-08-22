use jlrs::prelude::*;
use std::time::Duration;

// This struct contains the data our task will need. This struct must implement `Send`, `Sync`,
// and contain no borrowed data.
struct AccumulatorTask {
    init_value: f64,
}

// `Implement `GeneratorTask` for `AccumulatorTask`. This requires `async_trait` because traits
// with async methods are not yet available in Rust. Because the task itself is not thread-safe it
// is marked with `?Send`.
#[async_trait(?Send)]
impl GeneratorTask for AccumulatorTask {
    // State is the type of data that GeneratorTask::init returns. The frame provided to
    // GeneratorTask::init isn't dropped until the generator is dropped so the state can contain
    // Julia data rooted in that frame. When GeneratorTask::run is called it can use a mutable
    // reference to this data. The state of this task is simply a Julia value.
    type State = Value<'static, 'static>;

    // Input is the type of data that must be provided when the generator's handle is used to
    // call it. Like State, it's provided to GeneratorTask::run. This tasks expects an f64.
    type Input = f64;

    // Output is the type of data that GeneratorTask::run returns if it completes successfully.
    // This result is returned to the caller through a channel.
    type Output = f64;

    // The first three of these constants can be set to adjust the number of slots that are
    // preallocated for the frames provided to GeneratorTask::register, GeneratorTask::init
    // and GeneratorTask::run respectively. By default they're 0 and no slots are preallocated.
    // The last sets the capacity of the channel that's used by the generator and its handle to
    // communicate, by default it's 0, in which case an unbounded channel is used.
    const REGISTER_SLOTS: usize = 1;
    const INIT_SLOTS: usize = 1;
    const RUN_SLOTS: usize = 1;
    const CHANNEL_CAPACITY: usize = 2;

    // Register this task. This method can take care of custom initialization work, in this case
    // creating the mutable MutFloat64 type in the Main module.
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

    // Initialize the generator. Because the frame is not dropped until all handles to the task 
    // have been dropped and every pending call has completed, Julia data rooted in this frame 
    // can be returned as State. Here, the value we'll use as an accumulator is created and 
    // returned.
    async fn init<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<Self::State> {
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

    // Call the generator once. Note that while the state can be mutated, you can't replace any
    // Julia data that it contains with newly allocated data because it's called in a nested 
    // scope.
    async fn run<'inner, 'frame>(
        &'inner mut self,
        _global: Global<'frame>,
        frame: &'inner mut AsyncGcFrame<'frame>,
        state: &'inner mut Self::State,
        input: Self::Input,
    ) -> JlrsResult<Self::Output> {
        // Add call_cata to the accumulator and return its new value. The accumulator is mutable
        // Julia data so its contents can be changed.
        let value = state.get_raw_field::<f64, _>("v")? + input;
        let new_value = Value::new(&mut *frame, value)?;

        unsafe {
            state.set_field(frame, "v", new_value)?.into_jlrs_result()?;
        }

        Ok(value)
    }
}

fn main() {
    // The first thing we need to do is initialize Julia in a separate thread, to do so the method
    // AsyncJulia::init is used. This method takes three arguments: the maximum number of active
    // tasks, the capacity of the channel used to communicate with the async runtime, and the
    // timeout in ms that is used when trying to receive a new message. If the timeout happens
    // while there are active tasks, control of the thread is yielded to Julia, this allows the
    // garbage collector and scheduler to run.
    //
    // Here we allow four tasks to be running concurrently, a backlog of sixteen messages before
    // the channel is full, and yield control of the thread to Julia after one ms.
    let (julia, thread_handle) = unsafe { AsyncJulia::init(4, 16, Duration::from_millis(1)).expect("Could not init Julia") };

    {
        // Register AccumulatorTask, otherwise AccumulatorTask::init returns an error.
        let (init_sender, init_receiver) = crossbeam_channel::bounded(1);
        julia.try_register_generator::<AccumulatorTask, _>(init_sender).unwrap();
        init_receiver.recv().unwrap().unwrap();
    }

    // Create a new AccumulatorTask, if AccumulatorTask::init completes successfully a handle to 
    // the generator is returned.
    let generator = julia
        .try_generator(AccumulatorTask { init_value: 5.0 })
        .expect("AccumulatorTask::init failed");

    // Call the generator twice. Because AccumulatorTask::Input is f64, that data must be 
    // provided here.
    let (sender1, receiver1) = crossbeam_channel::bounded(1);
    generator.try_call(5.0, sender1).unwrap();
    let (sender2, receiver2) = crossbeam_channel::bounded(1);
    generator.try_call(10.0, sender2).unwrap();

    // Receive the results of the tasks.
    let x = receiver1.recv().unwrap().unwrap();
    println!("Result of first task: {}", x);

    let y = receiver2.recv().unwrap().unwrap();
    println!("Result of second task: {}", y);

    // Dropping the generator and `julia` causes the runtime to shut down Julia and itself. Join 
    // the handle to wait for everything to shut down cleanly.
    std::mem::drop(generator);
    std::mem::drop(julia);
    thread_handle
        .join()
        .expect("Cannot join")
        .expect("Unable to init Julia");
}
