use jlrs::{
    data::{
        layout::valid_layout::ValidLayout,
        managed::background_task::{BackgroundTaskLayout, spawn_background_task},
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
                .call(&mut frame, [bg_task.as_value()])
                .unwrap()
                .unbox::<usize>()
                .unwrap()
        };

        assert_eq!(v, 7);
    });

    std::mem::drop(rt);
}
