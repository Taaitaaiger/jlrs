use jlrs::prelude::*;
use std::time::Duration;

// This struct contains the y-label of the plot.
struct MyTask {
    ylabel: String,
}

// `MyTask` is a task that stays alive while the plot window is open, so we need to implement
// `PersistentTask`. This requires `async_trait` because traits with async methods are not yet
// available in Rust. Because the task itself is executed on a single thread, it is marked
// with `?Send`.
#[async_trait(?Send)]
impl PersistentTask for MyTask {
    // The task takes no input when called, and returns nothing. The state will hold a PyPlot and
    // keyword arguments that will be reused.
    type Input = ();
    type Output = ();
    type State = (PyPlot<'static>, Value<'static, 'static>);

    async fn init<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<Self::State> {
        unsafe {
            // Create the first plot with no data, but with a custom label for the y-axis.
            let plot_fn = Module::plots(global)
                .function_ref("plot")?
                .wrapper_unchecked();

            let ylabel_str = JuliaString::new(&mut *frame, &self.ylabel)?;
            let ylabel = Tuple::new(&mut *frame, &mut [ylabel_str.as_value()])?.into_jlrs_result()?;
            let kws = named_tuple!(&mut *frame, "yaxis" => ylabel)?;

            let plot = PyPlot::new_with_keywords(frame, plot_fn, &mut [], kws)?;

            Ok((plot, kws))
        }
    }

    async fn run<'inner, 'frame>(
        &'inner mut self,
        global: Global<'frame>,
        frame: &'inner mut AsyncGcFrame<'frame>,
        state: &'inner mut Self::State,
        _input: Self::Input,
    ) -> JlrsResult<Self::Output> {
        unsafe {
            println!("Update");
            // Add a line with 100 points to the plot
            let n = Value::new(&mut *frame, 100usize)?;
            let data = Module::base(global)
                .function_ref("randn")?
                .wrapper_unchecked()
                .call1(&mut *frame, n)?
                .into_jlrs_result()?;

            let plot_fn = Module::plots(global)
                .function_ref("plot")?
                .wrapper_unchecked();

            state
                .0
                .update_with_keywords(&mut *frame, plot_fn, &mut [data], state.1)?;
        }

        Ok(())
    }

    async fn exit<'inner>(
        &'inner mut self,
        _: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
        state: &'inner mut Self::State,
    ) {
        // Wait until the plot window is closed.
        println!("Exit");
        state.0.wait_async_main(&mut *frame).await.unwrap();
        println!("Figure was closed");
    }
}

#[tokio::main]
async fn main() {
    let (julia, handle) = unsafe {
        AsyncJulia::init_async(4, 16, Duration::from_millis(1))
            .await
            .expect("Could not init Julia")
    };

    let persistent_handle = julia
        .persistent(MyTask {
            ylabel: String::from("Random data"),
        })
        .await
        .unwrap();

    // Call the task ten times, waiting a second between each call.
    for _ in 0..10 {
        let (s, r) = tokio::sync::oneshot::channel();
        persistent_handle.call((), s).await;
        let res = r.await.unwrap();
        if res.is_err() {
            println!("Error: {}", res.unwrap_err());
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    // Drop the handle, `MyTask::exit` is called which waits until the window has been closed
    std::mem::drop(persistent_handle);

    // Dropping `julia` causes the runtime to shut down Julia and itself because it's the last
    // handle.
    std::mem::drop(julia);
    handle
        .await
        .expect("Julia exited with an error")
        .expect("The Julia thread panicked");
}
