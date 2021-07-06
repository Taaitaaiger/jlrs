use jlrs::prelude::*;
use std::any::Any;

// This struct contains the data needed to call some Julia functions, in this case
// `dims` and `iters`. There's also a `sender` that is used to send back the eventual result.
// For more complex types that don't implement `Copy`, you can wrap them in `Option` and use
// `Option::take` to extract them from the struct.
struct MyTask {
    dims: isize,
    iters: isize,
    sender: async_std::channel::Sender<JlrsResult<Box<dyn Any + Send + Sync>>>,
}

// `MyTask` is a task we want to be executed, so we need to implement `AsyncTask`. This requires
// `async_trait` because traits with async methods are not yet available in Rust. Because the
// task itself is executed on a single thread, it is marked with `?Send`.
#[async_trait(?Send)]
impl AsyncTask for MyTask {
    // If successful, the data is returned as a boxed `Any`. This way we can have different tasks
    // that return data of different types.
    type T = Box<dyn Any + Send + Sync>;
    // We use the `Sender` from the crossbeam_channel crate to send back results. Even if this
    // task doesn't return a result to the caller, `R` must be set.
    type R = async_std::channel::Sender<JlrsResult<Self::T>>;

    // This is the async variation of the closure you give to `Julia::scope` or
    // `Julia::scope_with_slots` when you use the synchronous runtime. The `Global` can be used to
    // access `Module`s and other static data, while the `AsyncGcFrame` let you create values, call
    // functions, and create nested frames.
    async fn run<'base>(
        &mut self,
        global: Global<'base>,
        frame: &mut AsyncGcFrame<'base>,
    ) -> JlrsResult<Self::T> {
        // Nesting async frames works like nesting on ordinary frame. The main differences are the `async`
        // block in the closure, and frame is provided by value rather than by mutable reference.
        let v = frame
            .async_value_scope(|output, frame| async move {
                // Convert the two arguments to values Julia can work with.
                let iters = Value::new(&mut *frame, self.iters)?;
                let dims = Value::new(&mut *frame, self.dims)?;

                // Get `complexfunc` in `MyModule`, call it asynchronously with `call_async`, and await
                // the result before casting it to an `f64` (which that function returns). A function that
                // is called with `call_async` is executed on a thread created with `Base.threads.@spawn`.
                // The module and function don't have to be rooted because the module is never redefined.
                let out = unsafe {
                    Module::main(global)
                        .submodule_ref("MyModule")?
                        .wrapper_unchecked()
                        .function_ref("complexfunc")?
                        .wrapper_unchecked()
                        .call_async(&mut *frame, &mut [dims, iters])
                        .await?
                        .unwrap()
                };

                let output = output.into_scope(frame);
                Ok(out.as_unrooted(output))
            })
            .await?
            .unbox::<f64>()?;

        // Box the result
        Ok(Box::new(v))
    }

    // Override the default implementation of `return_channel` so the result of this task is sent
    // back.
    fn return_channel(&self) -> Option<&async_std::channel::Sender<JlrsResult<Self::T>>> {
        Some(&self.sender)
    }
}

#[async_std::main]
async fn main() {
    // Initialize the async runtime. The `JULIA_NUM_THREADS` environment variable must be set to a
    // value larger than 1, or an error is returned.
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
        AsyncJulia::init_async(16, 1)
            .await
            .expect("Could not init Julia")
    };

    // Let's include the custom code our task needs.
    julia.include("MyModule.jl").await.unwrap();

    // Create channels for each of the tasks (this is not required but helps distinguish which
    // result belongs to which task).
    let (sender1, receiver1) = async_std::channel::bounded(1);
    let (sender2, receiver2) = async_std::channel::bounded(1);
    let (sender3, receiver3) = async_std::channel::bounded(1);
    let (sender4, receiver4) = async_std::channel::bounded(1);

    // Send four tasks to the runtime.
    julia
        .task(MyTask {
            dims: 4,
            iters: 100_000_000,
            sender: sender1,
        })
        .await;

    julia
        .task(MyTask {
            dims: 4,
            iters: 200_000_000,
            sender: sender2,
        })
        .await;

    julia
        .task(MyTask {
            dims: 4,
            iters: 300_000_000,
            sender: sender3,
        })
        .await;

    julia
        .task(MyTask {
            dims: 4,
            iters: 400_000_000,
            sender: sender4,
        })
        .await;

    // Receive the results of the tasks. `Any::downcast_ref` can be used to convert the
    // result to the appropriate type.
    let res1 = receiver1.recv().await.unwrap().unwrap();
    println!("Result of first task: {:?}", res1.downcast_ref::<f64>());
    let res2 = receiver2.recv().await.unwrap().unwrap();
    println!("Result of second task: {:?}", res2.downcast_ref::<f64>());
    let res3 = receiver3.recv().await.unwrap().unwrap();
    println!("Result of third task: {:?}", res3.downcast_ref::<f64>());
    let res4 = receiver4.recv().await.unwrap().unwrap();
    println!("Result of fourth task: {:?}", res4.downcast_ref::<f64>());

    // Dropping `julia` causes the runtime to shut down Julia and itself if it was the final
    // handle to the runtime.
    std::mem::drop(julia);
    handle.await.expect("Julia exited with an error");
}
