use jlrs::{
    data::{
        layout::valid_layout::ValidLayout,
        managed::background_task::{spawn_background_task, BackgroundTaskLayout},
    },
    prelude::*,
};

#[test]
fn run_background_task() {
    let rt = Builder::new().start_local().unwrap();

    rt.local_scope::<_, 2>(|mut frame| {
        let bg_task = spawn_background_task::<usize, _, _>(&mut frame, || Ok(7usize));

        assert!(BackgroundTaskLayout::<usize>::valid_layout(
            bg_task.as_value().datatype().as_value()
        ));

        let v = unsafe {
            Module::base(&frame)
                .global(&frame, "fetch")
                .unwrap()
                .as_value()
                .call1(&mut frame, bg_task.as_value())
                .into_jlrs_result()
                .unwrap()
                .unbox::<usize>()
                .unwrap()
        };

        assert_eq!(v, 7);
    });

    std::mem::drop(rt);
}
