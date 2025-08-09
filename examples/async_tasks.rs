use std::path::PathBuf;

use jlrs::{async_util::task::Register, prelude::*};

// This struct contains the data our task will need. This struct must be `Send`, `Sync`, and
// contain no borrowed data.
struct MyTask {
    dims: isize,
    iters: isize,
}

impl AsyncTask for MyTask {
    // Different tasks can return different results. If successful, this task returns an `f64`.
    type Output = JlrsResult<f64>;

    // This is the async variation of the closure you provide `Julia::scope` when using the sync
    // runtime.
    async fn run<'frame>(self, mut frame: AsyncGcFrame<'frame>) -> Self::Output {
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
                .as_managed()
                .global(&frame, "complexfunc")?
                .as_managed()
                .call_async(&mut frame, [dims, iters])
                .await
                .into_jlrs_result()?
                .unbox::<f64>()
        }
    }
}

impl Register for MyTask {
    // Include the custom code MyTask needs.
    async fn register<'frame>(frame: AsyncGcFrame<'frame>) -> JlrsResult<()> {
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

fn main() {
    // The first thing we need to do is initialize the async runtime.
    let (julia, handle) = Builder::new()
        .async_runtime(Tokio::<1>::new(false))
        .channel_capacity(2)
        .spawn()
        .expect("Could not init Julia");

    // Include the custom code MyTask needs by registering it.
    julia
        .register_task::<MyTask>()
        .try_dispatch()
        .unwrap()
        .blocking_recv()
        .unwrap()
        .unwrap();

    // Send two tasks to the runtime.
    let task1 = julia
        .task(MyTask {
            dims: 4,
            iters: 1_000_000,
        })
        .try_dispatch()
        .unwrap();

    let task2 = julia
        .task(MyTask {
            dims: 6,
            iters: 1_000_000,
        })
        .try_dispatch()
        .unwrap();

    // Receive the results of the tasks.
    let x = task1.blocking_recv().unwrap().unwrap();
    println!("Result of first task: {}", x);

    let y = task2.blocking_recv().unwrap().unwrap();
    println!("Result of second task: {}", y);

    // Dropping `julia` causes the runtime to shut down Julia and itself. Join the handle to wait
    // for everything to shut down cleanly.
    std::mem::drop(julia);
    handle.join().expect("runtime thread panicked");
}
