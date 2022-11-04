use jlrs::prelude::*;
use std::{num::NonZeroUsize, path::PathBuf};

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
    async fn register<'base>(mut frame: AsyncGcFrame<'base>) -> JlrsResult<()> {
        unsafe {
            let path = PathBuf::from("MyModule.jl");
            if path.exists() {
                Value::include(frame.as_extended_target(), "MyModule.jl")?.into_jlrs_result()?;
            } else {
                Value::include(frame.as_extended_target(), "examples/MyModule.jl")?
                    .into_jlrs_result()?;
            }
        }
        Ok(())
    }

    // This is the async variation of the closure you provide `Julia::scope` when using the sync
    // runtime. The `Global` can be used to access `Module`s and other static data, while the
    // `AsyncGcFrame` lets you create new Julia values, call functions, and create nested scopes.
    async fn run<'base>(&mut self, mut frame: AsyncGcFrame<'base>) -> JlrsResult<Self::Output> {
        // Convert the two arguments to values Julia can work with.
        let dims = Value::new(&mut frame, self.dims);
        let iters = Value::new(&mut frame, self.iters);

        // Get `complexfunc` in `MyModule`, call it on another thread with `call_async`, and await
        // the result before casting it to an `f64` (which that function returns). A function that
        // is called with `call_async` is executed on another thread by calling
        // `Base.threads.@spawn`.
        // The module and function don't have to be rooted because the module is never redefined,
        // so they're globally rooted.
        unsafe {
            Module::main(&frame)
                .submodule(&frame, "MyModule")?
                .wrapper_unchecked()
                .function(&frame, "complexfunc")?
                .wrapper_unchecked()
                .call_async(&mut frame, &mut [dims, iters])
                .await
                .into_jlrs_result()?
                .unbox::<f64>()
        }
    }
}

#[async_std::main]
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
        RuntimeBuilder::new()
            .async_runtime::<AsyncStd>()
            .channel_capacity(NonZeroUsize::new(4).unwrap())
            .start_async::<1>()
            .expect("Could not init Julia")
    };

    {
        // Include the custom code MyTask needs by registering it.
        let (sender, receiver) = async_std::channel::bounded(1);
        julia.register_task::<MyTask, _>(sender).await;
        receiver.recv().await.unwrap().unwrap();
    }

    // Create channels for each of the tasks (this is not required but helps distinguish which
    // result belongs to which task).
    let (sender1, receiver1) = async_std::channel::bounded(1);
    let (sender2, receiver2) = async_std::channel::bounded(1);
    let (sender3, receiver3) = async_std::channel::bounded(1);
    let (sender4, receiver4) = async_std::channel::bounded(1);

    // Send four tasks to the runtime.
    julia
        .task(
            MyTask {
                dims: 4,
                iters: 1_000_000,
            },
            sender1,
        )
        .await;

    julia
        .task(
            MyTask {
                dims: 4,
                iters: 1_000_000,
            },
            sender2,
        )
        .await;

    julia
        .task(
            MyTask {
                dims: 4,
                iters: 1_000_000,
            },
            sender3,
        )
        .await;

    julia
        .task(
            MyTask {
                dims: 4,
                iters: 1_000_000,
            },
            sender4,
        )
        .await;

    // Receive the results of the tasks.
    let res1 = receiver1.recv().await.unwrap().unwrap();
    println!("Result of first task: {:?}", res1);
    let res2 = receiver2.recv().await.unwrap().unwrap();
    println!("Result of second task: {:?}", res2);
    let res3 = receiver3.recv().await.unwrap().unwrap();
    println!("Result of third task: {:?}", res3);
    let res4 = receiver4.recv().await.unwrap().unwrap();
    println!("Result of fourth task: {:?}", res4);

    // Dropping `julia` causes the runtime to shut down Julia and itself if it was the final
    // handle to the runtime.
    std::mem::drop(julia);
    handle.await.expect("Julia exited with an error");
}
