#[cfg(all(feature = "tokio-rt",))]
#[cfg(test)]
mod async_util;

#[cfg(all(feature = "tokio-rt",))]
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use jlrs::prelude::*;
    use once_cell::sync::OnceCell;

    use super::async_util::{async_tasks::*, ASYNC_TESTS_JL};

    fn init() -> Arc<AsyncHandle> {
        unsafe {
            let r = Arc::new(
                Builder::new()
                    .async_runtime(Tokio::<4>::new(false))
                    .n_threads(4)
                    .channel_capacity(32)
                    .spawn()
                    .expect("Could not init Julia")
                    .0,
            );

            let blocking_recv = r
                .as_ref()
                .blocking_task(|mut frame| -> JlrsResult<()> {
                    Value::eval_string(&mut frame, ASYNC_TESTS_JL).into_jlrs_result()?;
                    Ok(())
                })
                .try_dispatch()
                .ok()
                .expect("Could not send blocking task");

            blocking_recv
                .blocking_recv()
                .expect("Could not receive reply")
                .expect("Could not load AsyncTests module");

            r
        }
    }

    pub static JULIA: OnceCell<Arc<AsyncHandle>> = OnceCell::new();

    #[test]
    fn test_task() {
        let julia = JULIA.get_or_init(init);

        let blocking_recv = julia
            .task(MyTask {
                dims: 4,
                iters: 5_000_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(
            blocking_recv.blocking_recv().unwrap().unwrap(),
            20_000_004.0
        );
    }

    #[test]
    fn test_other_ret_type_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(OtherRetTypeTask {
                dims: 4,
                iters: 5_000_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap(), 20_000_004.0);
    }

    #[test]
    fn test_kw_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(KwTask {
                dims: 4,
                iters: 5_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_throwing_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia.task(ThrowingTask).try_dispatch().ok().unwrap();

        assert!(receiver.blocking_recv().unwrap().is_err());
    }

    #[test]
    fn test_nesting_static_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(NestingTaskAsyncFrame {
                dims: 6,
                iters: 5_000_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 30_000_006.0);
    }

    #[test]
    fn test_nesting_value_static_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(NestingTaskAsyncValueFrame {
                dims: 6,
                iters: 5_000_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 30_000_006.0);
    }

    #[test]
    fn test_nesting_call_static_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(NestingTaskAsyncCallFrame {
                dims: 6,
                iters: 5_000_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 30_000_006.0);
    }

    #[test]
    fn test_nesting_dynamic_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(NestingTaskAsyncGcFrame {
                dims: 6,
                iters: 5_000_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 30_000_006.0);
    }

    #[test]
    fn test_nesting_value_dynamic_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(NestingTaskAsyncDynamicValueFrame {
                dims: 6,
                iters: 5_000_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 30_000_006.0);
    }

    // /*
    // #[test]
    // fn test_nesting_call_dynamic_task() {
    //     let julia = JULIA.get_or_init(init);

    //    let receiver =

    //     julia
    //         .task(
    //             NestingTaskAsyncDynamicCallFrame {
    //                 dims: 6,
    //                 iters: 5_000_000,
    //             },
    //         )
    //         .try_dispatch().ok()
    //         .unwrap();

    //     assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 30_000_006.0);
    // }
    // */
    #[test]
    fn test_persistent() {
        let julia = JULIA.get_or_init(init);

        julia
            .register_task::<AccumulatorTask>()
            .try_dispatch()
            .ok()
            .unwrap()
            .blocking_recv()
            .unwrap()
            .unwrap();

        //     let handle = julia
        //         .persistent(AccumulatorTask { init_value: 5.0 })
        //         .try_dispatch()
        //         .ok()
        //         .expect("Cannot send task")
        //         .blocking_recv()
        //         .unwrap()
        //         .unwrap();

        //     let res = handle
        //         .call(7.0)
        //         .try_dispatch()
        //         .ok()
        //         .unwrap()
        //         .blocking_recv()
        //         .unwrap()
        //         .unwrap();

        //     assert_eq!(res, 12.0);

        //     let res = handle
        //         .call(12.0)
        //         .try_dispatch()
        //         .ok()
        //         .unwrap()
        //         .blocking_recv()
        //         .unwrap()
        //         .unwrap();

        //     assert_eq!(res, 24.0);
        //     // let handle = {
        //     //     let (handle_sender, handle_receiver) = crossbeam_channel::bounded(1);

        //     //     handle_receiver
        //     //         .blocking_recv()
        //     //         .expect("Channel was closed")
        //     //         .expect("Cannot init task")
        //     // };

        //     // handle.try_call(7.0.clone()).unwrap();
        //     // assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 12.0);

        //     // handle.try_call(12.0).unwrap();
        //     // assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 24.0);
    }

    #[test]
    fn test_local_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(LocalTask {
                dims: 4,
                iters: 5_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 20_004.0);
    }

    #[test]
    fn test_scheduling_local_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(LocalSchedulingTask {
                dims: 4,
                iters: 5_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 20_004.0);
    }

    #[test]
    fn test_main_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(MainTask {
                dims: 4,
                iters: 5_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 20_004.0);
    }

    #[test]
    fn test_scheduling_main_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(MainSchedulingTask {
                dims: 4,
                iters: 5_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 20_004.0);
    }

    #[test]
    fn test_scheduling_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(SchedulingTask {
                dims: 4,
                iters: 5_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 20_004.0);
    }

    #[test]
    fn test_scheduling_kw_local_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(LocalKwSchedulingTask {
                dims: 4,
                iters: 5_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_scheduling_kw_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(KwSchedulingTask {
                dims: 4,
                iters: 5_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_scheduling_kw_main_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(MainKwSchedulingTask {
                dims: 4,
                iters: 5_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_local_kw_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(LocalKwTask {
                dims: 4,
                iters: 5_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_main_kw_task() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia
            .task(MainKwTask {
                dims: 4,
                iters: 5_000,
            })
            .try_dispatch()
            .ok()
            .unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_borrow_array_data() {
        let julia = JULIA.get_or_init(init);

        let receiver = julia.task(BorrowArrayData).try_dispatch().ok().unwrap();

        assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 2.0);
    }

    // #[test]
    // fn test_post_task() {
    //     let julia = JULIA.get_or_init(init);

    //     let receiver = julia
    //         .post_blocking_task(|mut frame| {
    //             let one = Value::new(&mut frame, 1.0);
    //             unsafe {
    //                 Module::base(&frame)
    //                     .function(&frame, "+")
    //                     .unwrap()
    //                     .as_managed()
    //                     .call2(&mut frame, one, one)
    //                     .into_jlrs_result()?
    //                     .unbox::<f64>()
    //             }
    //         })
    //         .try_dispatch()
    //         .ok()
    //         .unwrap();

    //     assert_eq!(receiver.blocking_recv().unwrap().unwrap(), 2.0);
    // }
}
