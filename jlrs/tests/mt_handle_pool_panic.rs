#[cfg(all(feature = "multi-rt", feature = "async-rt"))]
mod mt_handle {

    use async_trait::async_trait;
    use jlrs::{
        async_util::task::AsyncTask,
        memory::target::frame::AsyncGcFrame,
        runtime::{builder::Builder, executor::tokio_exec::Tokio},
    };

    pub struct PanickingTask;

    #[async_trait(?Send)]
    impl AsyncTask for PanickingTask {
        type Output = ();

        async fn run<'base>(&mut self, mut _frame: AsyncGcFrame<'base>) -> Self::Output {
            panic!()
        }
    }

    #[test]
    fn worker_is_revived() {
        Builder::new()
            .start_mt(|julia| {
                let handle = julia
                    .pool_builder(Tokio::<1>::new(false))
                    .n_workers(1.try_into().unwrap())
                    .spawn();

                assert_eq!(handle.n_workers(), 1);
                handle
                    .blocking_task(|_| panic!())
                    .try_dispatch()
                    .unwrap()
                    .blocking_recv()
                    .unwrap_err();
                handle
                    .blocking_task(|_| 1)
                    .try_dispatch()
                    .unwrap()
                    .blocking_recv()
                    .unwrap();
                assert_eq!(handle.n_workers(), 1);
                handle
                    .task(PanickingTask)
                    .try_dispatch()
                    .unwrap()
                    .blocking_recv()
                    .unwrap_err();
                handle
                    .blocking_task(|_| 1)
                    .try_dispatch()
                    .unwrap()
                    .blocking_recv()
                    .unwrap();
                assert_eq!(handle.n_workers(), 1);

                std::mem::drop(julia);
                std::mem::drop(handle);
            })
            .unwrap();
    }
}
