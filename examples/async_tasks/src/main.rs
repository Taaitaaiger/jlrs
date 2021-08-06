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
    // The first thing we need to do is initialize Julia in a separate thread, to do so the method
    // AsyncJulia::init is used. This method takes three arguments: the maximum number of active
    // tasks, the capacity of the channel used to communicate with the async runtime, and the
    // timeout in ms that is used when trying to receive a new message. If the timeout happens
    // while there are active tasks, control of the thread is yielded to Julia, this allows the
    // garbage collector and scheduler to run.
    //
    // Here we allow four tasks to be running concurrently, a backlog of sixteen messages before
    // the channel is full, and yield control of the thread to Julia after one ms.
    //
    // After calling this method we have an instance of `AsyncJulia` that can be used to send
    // tasks and requests to include a file to the runtime, and a handle to the thread where the
    // runtime is running.
    let (julia, handle) = unsafe { AsyncJulia::init(4, 16, 1).expect("Could not init Julia") };

    // Include the custom code MyTask needs.
    julia.try_include("MyModule.jl").unwrap();

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
