use jlrs::prelude::*;
use std::any::Any;
use crossbeam_channel::Sender;

struct MyTask {
    dims: isize,
    iters: isize,
    sender: Sender<JlrsResult<Box<dyn Any + Send + Sync>>>,
}

#[async_trait(?Send)]
impl JuliaTask for MyTask {
    type T = Box<dyn Any + Send + Sync>;
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

        Ok(Box::new(v))
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
        let (julia, handle) =
            unsafe { AsyncJulia::init(16, 2, 16, 1, "../jlrs.jl").expect("Could not init Julia") };

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

        assert!(receiver2.recv().unwrap().unwrap().downcast_ref::<f64>().is_some());
        assert!(receiver1.recv().unwrap().unwrap().downcast_ref::<f64>().is_some());

        std::mem::drop(julia);
        handle
            .join()
            .expect("Cannot join")
            .expect("Unable to stop Julia");
    }
}
