use jlrs::prelude::*;
use std::time::Duration;

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

#[tokio::main]
async fn main() {
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
    let (julia, handle) = unsafe {
        AsyncJulia::init_async(4, 16, Duration::from_millis(1))
            .await
            .expect("Could not init Julia")
    };

    // Include the custom code our task needs.
    julia.include("MyModule.jl").await.unwrap();

    // Create channels for each of the tasks. 
    let (sender1, receiver1) = tokio::sync::oneshot::channel();
    let (sender2, receiver2) = tokio::sync::oneshot::channel();
    let (sender3, receiver3) = tokio::sync::oneshot::channel();
    let (sender4, receiver4) = tokio::sync::oneshot::channel();

    // Send four tasks to the runtime.
    julia
        .task(MyTask {
            dims: 4,
            iters: 100_000_000,
        }, sender1)
        .await;

    julia
        .task(MyTask {
            dims: 4,
            iters: 200_000_000,
        }, sender2)
        .await;

    julia
        .task(MyTask {
            dims: 4,
            iters: 300_000_000,
        }, sender3)
        .await;

    julia
        .task(MyTask {
            dims: 4,
            iters: 400_000_000,
        }, sender4)
        .await;

    // Receive the results of the tasks.
    let res1 = receiver1.await.unwrap().unwrap();
    println!("Result of first task: {:?}", res1);
    let res2 = receiver2.await.unwrap().unwrap();
    println!("Result of second task: {:?}", res2);
    let res3 = receiver3.await.unwrap().unwrap();
    println!("Result of third task: {:?}", res3);
    let res4 = receiver4.await.unwrap().unwrap();
    println!("Result of fourth task: {:?}", res4);

    // Dropping `julia` causes the runtime to shut down Julia and itself if it was the final
    // handle to the runtime.
    std::mem::drop(julia);
    handle.await.expect("Julia exited with an error").expect("The Julia thread panicked");
}
