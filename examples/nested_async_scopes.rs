use std::path::PathBuf;

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
        mut frame: AsyncGcFrame<'base>,
    ) -> JlrsResult<()> {
        unsafe {
            let path = PathBuf::from("MyModule.jl");
            if path.exists() {
                Value::include(&mut frame, "MyModule.jl")?.into_jlrs_result()?;
            } else {
                Value::include(&mut frame, "examples/MyModule.jl")?.into_jlrs_result()?;
            }
        }
        Ok(())
    }

    // This is the async variation of the closure you provide `Julia::scope` when using the sync
    // runtime. The `Global` can be used to access `Module`s and other static data, while the
    // `AsyncGcFrame` lets you create new Julia values, call functions, and create nested scopes.
    async fn run<'base>(
        &mut self,
        global: Global<'base>,
        mut frame: AsyncGcFrame<'base>,
    ) -> JlrsResult<Self::Output> {
        // Nesting async frames works like nesting on ordinary scope. The main differences are the `async`
        // block in the closure.
        let output = frame.output();
        frame
            .async_scope(|mut frame| async move {
                // Convert the two arguments to values Julia can work with.
                let dims = Value::new(&mut frame, self.dims);
                let iters = Value::new(&mut frame, self.iters);

                // Get `complexfunc` in `MyModule`, call it on another thread with `call_async`, and await
                // the result before casting it to an `f64` (which that function returns). A function that
                // is called with `call_async` is executed on another thread by calling
                // `Base.threads.@spawn`.
                // The module and function don't have to be rooted because the module is never redefined,
                // so they're globally rooted.
                let out = unsafe {
                    Module::main(global)
                        .submodule_ref("MyModule")?
                        .wrapper_unchecked()
                        .function_ref("complexfunc")?
                        .wrapper_unchecked()
                        .call_async(&mut frame, &mut [dims, iters])
                        .await?
                        .into_jlrs_result()?
                };

                Ok(out.root(output))
            })
            .await?
            .unbox::<f64>()
    }
}

#[tokio::main]
async fn main() {
    // Initialize the async runtime.
    //
    // The runtime runs in a separate thread. It receives messages through a channel, a backlog
    // can build up if a task which does a significant amount of work on the main thread is
    // blocking the runtime. The queue size of this channel is set with the first argument of
    // `AsyncJulia::init`. Here we allow for a backlog of 16 messages before the channel is full.
    //
    // When one or more functions are running in other threads but the runtime has no synchronous
    // work to do, the garbage collector can't run. Similarly, async events in Julia (such as
    // rescheduling a task that has yielded after calling `sleep` or `println`) will not be
    // handled either. In order to fix this, event must be processed. We do so every millisecond.
    //
    // After calling this function we have an instance of `AsyncJulia` that can be used to send
    // tasks and requests to include a file to the runtime, and a handle to the thread where the
    // runtime is running.
    let (julia, handle) = unsafe {
        RuntimeBuilder::new()
            .async_runtime::<Tokio, UnboundedChannel<_>>()
            .start_async::<1>()
            .expect("Could not init Julia")
    };

    {
        // Include the custom code MyTask needs by registering it.
        let (sender, receiver) = tokio::sync::oneshot::channel();
        julia.register_task::<MyTask, _>(sender).await.unwrap();
        receiver.await.unwrap().unwrap();
    }

    // Create channels for each of the tasks (this is not required but helps distinguish which
    // result belongs to which task).
    let (sender1, receiver1) = tokio::sync::oneshot::channel();
    let (sender2, receiver2) = tokio::sync::oneshot::channel();
    let (sender3, receiver3) = tokio::sync::oneshot::channel();
    let (sender4, receiver4) = tokio::sync::oneshot::channel();
    // Send four tasks to the runtime.
    julia
        .task(
            MyTask {
                dims: 4,
                iters: 1_000_000,
            },
            sender1,
        )
        .await
        .expect("Cannot send task");

    julia
        .task(
            MyTask {
                dims: 4,
                iters: 1_000_000,
            },
            sender2,
        )
        .await
        .expect("Cannot send task");

    julia
        .task(
            MyTask {
                dims: 4,
                iters: 1_000_000,
            },
            sender3,
        )
        .await
        .expect("Cannot send task");

    julia
        .task(
            MyTask {
                dims: 4,
                iters: 1_000_000,
            },
            sender4,
        )
        .await
        .expect("Cannot send task");

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
    handle
        .await
        .expect("Could not await the task")
        .expect("Julia exited with an error");
}
