use std::path::PathBuf;

use jlrs::{async_util::task::Register, prelude::*};

// This struct contains the data our task will need. This struct must be `Send`, `Sync`, and
// contain no borrowed data.
struct MyTask {
    dims: isize,
    iters: isize,
}

// `MyTask` is a task we want to be executed, so we need to implement `AsyncTask`.
impl AsyncTask for MyTask {
    // Different tasks can return different results. If successful, this task returns an `f64`.
    type Output = JlrsResult<f64>;

    // This is the async variation of the closure you provide `Julia::scope` when using the sync
    // runtime.
    async fn run<'base>(self, mut frame: AsyncGcFrame<'base>) -> Self::Output {
        // Nesting async frames works like nesting on ordinary scope. The main difference is that
        // the closure must return an async block.
        let output = frame.output();
        frame
            .async_scope(async move |mut frame| -> JlrsResult<_> {
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
                    Module::main(&frame)
                        .submodule(&frame, "MyModule")?
                        .as_managed()
                        .global(&frame, "complexfunc")?
                        .as_managed()
                        .call_async(&mut frame, [dims, iters])
                        .await
                        .into_jlrs_result()?
                };

                Ok(out.root(output))
            })
            .await?
            .unbox::<f64>()
    }
}

impl Register for MyTask {
    // Include the custom code MyTask needs.
    async fn register<'base>(frame: AsyncGcFrame<'base>) -> JlrsResult<()> {
        unsafe {
            let path = PathBuf::from("MyModule.jl");
            if path.exists() {
                frame.runtime_settings().include("MyModule.jl")?;
            } else {
                frame.runtime_settings().include("examples/MyModule.jl")?;
            }
        }
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // The first thing we need to do is initialize the async runtime. In this example tokio is
    // used as backing runtime.
    //
    // Afterwards we have an instance of `AsyncJulia` that can be used to interact with the
    // runtime, and a handle to the thread where the runtime is running.
    let (julia, handle) = Builder::new()
        .async_runtime(Tokio::<1>::new(false))
        .channel_capacity(4)
        .spawn()
        .expect("Could not init Julia");

    {
        // Include the custom code MyTask needs by registering it.
        julia
            .register_task::<MyTask>()
            .dispatch()
            .await
            .expect("channel closed")
            .await
            .expect("channel closed")
            .expect("registration failed");
    }

    // Send two tasks to the runtime.
    let t1 = julia
        .task(MyTask {
            dims: 4,
            iters: 1_000_000,
        })
        .dispatch()
        .await
        .expect("channel closed");

    let t2 = julia
        .task(MyTask {
            dims: 4,
            iters: 1_000_000,
        })
        .dispatch()
        .await
        .expect("channel closed");

    // Receive the results of the tasks.
    let res1 = t1.await.unwrap().unwrap();
    println!("Result of first task: {:?}", res1);
    let res2 = t2.await.unwrap().unwrap();
    println!("Result of second task: {:?}", res2);

    // Dropping `julia` causes the runtime to shut down Julia and itself if it was the final
    // handle to the runtime. Await the runtime handle handle to wait for everything to shut
    // down cleanly.
    std::mem::drop(julia);
    handle.join().expect("Julia exited with an error");
}
