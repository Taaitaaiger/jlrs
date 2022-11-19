#[cfg(all(feature = "tokio-rt", not(all(target_os = "windows", feature = "lts"))))]
#[cfg(test)]
mod util;

#[cfg(all(feature = "tokio-rt", not(all(target_os = "windows", feature = "lts"))))]
#[cfg(test)]
mod tests {
    use super::util::{async_tasks::*, ASYNC_TESTS_JL};
    use jlrs::prelude::*;
    use once_cell::sync::OnceCell;
    use std::{num::NonZeroUsize, sync::Arc};

    fn init() -> Arc<AsyncJulia<Tokio>> {
        unsafe {
            let r = Arc::new(
                RuntimeBuilder::new()
                    .async_runtime::<Tokio>()
                    .n_threads(4)
                    .channel_capacity(NonZeroUsize::new_unchecked(32))
                    .start::<4>()
                    .expect("Could not init Julia")
                    .0,
            );

            let (sender, recv) = tokio::sync::oneshot::channel();
            r.as_ref()
                .try_blocking_task(
                    |mut frame| {
                        Value::eval_string(&mut frame, ASYNC_TESTS_JL).into_jlrs_result()?;
                        Ok(())
                    },
                    sender,
                    Affinity::Any,
                )
                .expect("Could not send blocking task");

            recv.blocking_recv()
                .expect("Could not receive reply")
                .expect("Could not load AsyncTests module");

            r
        }
    }

    pub static JULIA: OnceCell<Arc<AsyncJulia<Tokio>>> = OnceCell::new();

    #[test]
    fn test_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                MyTask {
                    dims: 4,
                    iters: 5_000_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_000_004.0);
    }

    #[test]
    fn test_other_ret_type_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                OtherRetTypeTask {
                    dims: 4,
                    iters: 5_000_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_000_004.0);
    }

    #[test]
    fn test_kw_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                KwTask {
                    dims: 4,
                    iters: 5_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_throwing_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia.try_task(ThrowingTask, sender).unwrap();

        assert!(receiver.recv().unwrap().is_err());
    }

    #[test]
    fn test_nesting_static_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                NestingTaskAsyncFrame {
                    dims: 6,
                    iters: 5_000_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
    }

    #[test]
    fn test_nesting_value_static_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                NestingTaskAsyncValueFrame {
                    dims: 6,
                    iters: 5_000_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
    }

    #[test]
    fn test_nesting_call_static_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                NestingTaskAsyncCallFrame {
                    dims: 6,
                    iters: 5_000_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
    }

    #[test]
    fn test_nesting_dynamic_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                NestingTaskAsyncGcFrame {
                    dims: 6,
                    iters: 5_000_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
    }

    #[test]
    fn test_nesting_value_dynamic_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                NestingTaskAsyncDynamicValueFrame {
                    dims: 6,
                    iters: 5_000_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
    }

    #[test]
    fn test_nesting_call_dynamic_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                NestingTaskAsyncDynamicCallFrame {
                    dims: 6,
                    iters: 5_000_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 30_000_006.0);
    }

    #[test]
    fn test_persistent() {
        let julia = JULIA.get_or_init(init);

        let (is, ir) = crossbeam_channel::bounded(1);
        julia
            .try_register_persistent::<AccumulatorTask, _>(is)
            .unwrap();
        ir.recv().unwrap().unwrap();

        let (sender, receiver) = crossbeam_channel::bounded(1);

        let handle = {
            let (handle_sender, handle_receiver) = crossbeam_channel::bounded(1);
            julia
                .try_persistent::<UnboundedChannel<_>, _, _>(
                    AccumulatorTask { init_value: 5.0 },
                    handle_sender,
                )
                .expect("Cannot send task");

            handle_receiver
                .recv()
                .expect("Channel was closed")
                .expect("Cannot init task")
        };

        handle.try_call(7.0, sender.clone()).unwrap();
        assert_eq!(receiver.recv().unwrap().unwrap(), 12.0);

        handle.try_call(12.0, sender).unwrap();
        assert_eq!(receiver.recv().unwrap().unwrap(), 24.0);
    }

    #[test]
    fn test_local_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                LocalTask {
                    dims: 4,
                    iters: 5_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_004.0);
    }

    #[test]
    fn test_scheduling_local_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                LocalSchedulingTask {
                    dims: 4,
                    iters: 5_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_004.0);
    }

    #[test]
    fn test_main_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                MainTask {
                    dims: 4,
                    iters: 5_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_004.0);
    }

    #[test]
    fn test_scheduling_main_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                MainSchedulingTask {
                    dims: 4,
                    iters: 5_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_004.0);
    }

    #[test]
    fn test_scheduling_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                SchedulingTask {
                    dims: 4,
                    iters: 5_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_004.0);
    }

    #[test]
    fn test_scheduling_kw_local_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                LocalKwSchedulingTask {
                    dims: 4,
                    iters: 5_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_scheduling_kw_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                KwSchedulingTask {
                    dims: 4,
                    iters: 5_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_scheduling_kw_main_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                MainKwSchedulingTask {
                    dims: 4,
                    iters: 5_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_local_kw_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                LocalKwTask {
                    dims: 4,
                    iters: 5_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_main_kw_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_task(
                MainKwTask {
                    dims: 4,
                    iters: 5_000,
                },
                sender,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 20_009.0);
    }

    #[test]
    fn test_borrow_array_data() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia.try_task(BorrowArrayData, sender).unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 2.0);
    }

    #[test]
    fn test_post_task() {
        let julia = JULIA.get_or_init(init);

        let (sender, receiver) = crossbeam_channel::bounded(1);

        julia
            .try_post_blocking_task(
                |mut frame| {
                    let one = Value::new(&mut frame, 1.0);
                    unsafe {
                        Module::base(&frame)
                            .function(&frame, "+")
                            .unwrap()
                            .wrapper()
                            .call2(&mut frame, one, one)
                            .into_jlrs_result()?
                            .unbox::<f64>()
                    }
                },
                sender,
                Affinity::Any,
            )
            .unwrap();

        assert_eq!(receiver.recv().unwrap().unwrap(), 2.0);
    }
}
