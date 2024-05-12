#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
mod delegated_task {
    use jlrs::{
        data::{
            layout::valid_layout::ValidLayout,
            managed::delegated_task::{spawn_delegated_task, DelegatedTaskLayout},
        },
        prelude::*,
        runtime::handle::local_handle::LocalHandle,
    };

    fn delegated_task(handle: &LocalHandle) {
        handle.local_scope::<_, 2>(|mut frame| {
            let data = ();

            let delegated = spawn_delegated_task(
                &mut frame,
                |handle, _data| {
                    handle.local_scope::<_, 0>(|frame| Ok(Value::new(&frame, 1isize).leak()))
                },
                data,
            );

            assert!(DelegatedTaskLayout::valid_layout(
                delegated.as_value().datatype().as_value()
            ));

            let v = unsafe {
                Module::base(&frame)
                    .global(&frame, "fetch")
                    .unwrap()
                    .as_value()
                    .call1(&mut frame, delegated.as_value())
                    .into_jlrs_result()
                    .unwrap()
                    .unbox::<isize>()
                    .unwrap()
            };

            assert_eq!(v, 1);
        });
    }

    #[test]
    fn run_delegated_task() {
        let rt = Builder::new().start_local().unwrap();
        delegated_task(&rt);
        std::mem::drop(rt);
    }
}
