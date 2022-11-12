use std::{num::NonZeroUsize, path::PathBuf};

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
    async fn register<'frame>(mut frame: AsyncGcFrame<'frame>) -> JlrsResult<()> {
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
    // runtime.
    async fn run<'frame>(&mut self, mut frame: AsyncGcFrame<'frame>) -> JlrsResult<Self::Output> {
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
                .wrapper()
                .function(&frame, "complexfunc")?
                .wrapper()
                .call_async(&mut frame, &mut [dims, iters])
                .await
                .into_jlrs_result()?
                .unbox::<f64>()
        }
    }
}

#[async_std::main]
async fn main() {
    // The first thing we need to do is initialize the async runtime. In this example async-std is
    // used as backing runtime.
    //
    // Afterwards we have an instance of `AsyncJulia` that can be used to interact with the
    // runtime, and a handle to the thread where the runtime is running.
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

    // Send two tasks to the runtime.
    let (sender1, receiver1) = async_std::channel::bounded(1);
    let (sender2, receiver2) = async_std::channel::bounded(1);

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

    // Receive the results of the tasks.
    let res1 = receiver1.recv().await.unwrap().unwrap();
    println!("Result of first task: {:?}", res1);
    let res2 = receiver2.recv().await.unwrap().unwrap();
    println!("Result of second task: {:?}", res2);

    // Dropping `julia` causes the runtime to shut down Julia and itself if it was the final
    // handle to the runtime. Await the runtime handle handle to wait for everything to shut
    // down cleanly.
    std::mem::drop(julia);
    handle.await.expect("The runtime thread panicked");
}
