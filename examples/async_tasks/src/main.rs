use jlrs::prelude::*;

// This struct contains the data our task will need. This struct must be `Send`, `Sync`, and
// contain no borrowed data.
struct MyTask {
    dims: isize,
    iters: isize,
}

// `MyTask` is a task we want to be executed, so we need to implement `AsyncTask`. This requires
// `async_trait` because traits with async methods are not yet available in Rust. Because the
// task itself is executed on a single thread, it is marked with `?Send`.
#[async_trait(?Send)]
impl AsyncTask for MyTask {
    // Different tasks can return different results. If successful, this task returns an `f64`.
    type Output = f64;

    // Include the custom code MyTask needs.
    async fn register<'base>(
        _global: Global<'base>,
        frame: &mut AsyncGcFrame<'base>,
    ) -> JlrsResult<()> {
        unsafe {
            Value::include(frame, "MyModule.jl")?.into_jlrs_result()?;
        }
        Ok(())
    }

    // This is the async variation of the closure you provide `Julia::scope` when using the sync
    // runtime. The `Global` can be used to access `Module`s and other static data, while the
    // `AsyncGcFrame` lets you create new Julia values, call functions, and create nested scopes.
    async fn run<'base>(
        &mut self,
        global: Global<'base>,
        frame: &mut AsyncGcFrame<'base>,
    ) -> JlrsResult<Self::Output> {
        // Convert the two arguments to values Julia can work with.
        let dims = Value::new(&mut *frame, self.dims)?;
        let iters = Value::new(&mut *frame, self.iters)?;

        // Get `complexfunc` in `MyModule`, call it on another thread with `call_async`, and await
        // the result before casting it to an `f64` (which that function returns). A function that
        // is called with `call_async` is executed on another thread by calling
        // `Base.threads.@spawn`.
        // The module and function don't have to be rooted because the module is never redefined,
        // so they're globally rooted.
        unsafe {
            Module::main(global)
                .submodule_ref("MyModule")?
                .wrapper_unchecked()
                .function_ref("complexfunc")?
                .wrapper_unchecked()
                .call_async(&mut *frame, &mut [dims, iters])
                .await?
                .unwrap()
                .unbox::<f64>()
        }
    }
}

fn main() {
    // The first thing we need to do is initialize Julia on a separate thread. In this example
    // tokio is used.
    //
    // Afterwards we have an instance of `AsyncJulia` that can be used to send
    // tasks and requests to include a file to the runtime, and a handle to the thread where the
    // runtime is running.
    let (julia, handle) = unsafe {
        RuntimeBuilder::new()
            .async_runtime::<Tokio, UnboundedChannel<_>>()
            .start()
            .expect("Could not init Julia")
    };

    {
        // Include the custom code MyTask needs by registering it.
        let (sender, receiver) = crossbeam_channel::bounded(1);
        julia.try_register_task::<MyTask, _>(sender).unwrap();
        receiver.recv().unwrap().unwrap();
    }

    // Send two tasks to the runtime.
    let receiver1 = {
        let task = MyTask {
            dims: 4,
            iters: 5_000_000,
        };
        let (sender, receiver) = crossbeam_channel::bounded(1);
        julia.try_task(task, sender).unwrap();
        receiver
    };

    let receiver2 = {
        let task = MyTask {
            dims: 6,
            iters: 5_000_000,
        };
        let (sender, receiver) = crossbeam_channel::bounded(1);
        julia.try_task(task, sender).unwrap();
        receiver
    };

    // Receive the results of the tasks.
    let x = receiver1.recv().unwrap().unwrap();
    println!("Result of first task: {}", x);

    let y = receiver2.recv().unwrap().unwrap();
    println!("Result of second task: {}", y);

    // Dropping `julia` causes the runtime to shut down Julia and itself. Join the handle to wait
    // for everything to shut down cleanly.
    std::mem::drop(julia);
    handle
        .join()
        .expect("Cannot join")
        .expect("Unable to init Julia");
}
