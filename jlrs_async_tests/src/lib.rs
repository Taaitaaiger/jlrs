#[cfg(target_os = "linux")]
mod example {
    use crossbeam_channel::Sender;
    use jlrs::prelude::*;

    struct MyTask {
        dims: isize,
        iters: isize,
        sender: Sender<JlrsResult<f64>>,
    }

    #[async_trait(?Send)]
    impl JuliaTask for MyTask {
        type T = f64;
        type R = Sender<JlrsResult<Self::T>>;

        async fn run<'base>(
            &mut self,
            global: Global<'base>,
            frame: &mut AsyncFrame<'base>,
        ) -> JlrsResult<Self::T> {
            let dims = Value::new(frame, self.dims)?;
            let iters = Value::new(frame, self.iters)?;

            let v = Module::main(global)
                .submodule("MyModule")?
                .function("complexfunc")?
                .call_async(frame, [dims, iters])
                .await?
                .unwrap()
                .cast::<f64>()?;

            Ok(v)
        }

        fn return_channel(&self) -> Option<&Sender<JlrsResult<Self::T>>> {
            Some(&self.sender)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            let (julia, handle) = unsafe {
                AsyncJulia::init(16, 2, 16, 1).expect("Could not init Julia")
            };

            julia.try_include("MyModule.jl").unwrap();

            let (sender1, receiver1) = crossbeam_channel::bounded(1);
            let (sender2, receiver2) = crossbeam_channel::bounded(1);

            julia
                .try_new_task(MyTask {
                    dims: 4,
                    iters: 5_000_000,
                    sender: sender1,
                })
                .unwrap();

            julia
                .try_new_task(MyTask {
                    dims: 6,
                    iters: 5_000_000,
                    sender: sender2,
                })
                .unwrap();

            assert_eq!(receiver2.recv().unwrap().unwrap(), 30_000_006.0);
            assert_eq!(receiver1.recv().unwrap().unwrap(), 20_000_004.0);

            std::mem::drop(julia);
            handle
                .join()
                .expect("Cannot join")
                .expect("Unable to start Julia");
        }
    }
}
