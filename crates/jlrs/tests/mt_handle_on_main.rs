#[cfg(feature = "multi-rt")]
mod mt_handle {
    use jlrs::{data::managed::value::Value, memory::scope::LocalScope, runtime::builder::Builder};

    #[test]
    fn call_from_current_and_spawned_thread() {
        Builder::new()
            .start_mt(|mut julia| {
                let t1 = julia.with(|handle| {
                    handle.local_scope::<_, 1>(|mut frame| unsafe {
                        Value::eval_string(&mut frame, "1 + 2")
                            .unwrap()
                            .unbox::<isize>()
                    })
                });

                let t2 = julia.spawn(move |mut julia| {
                    julia.with(|handle| {
                        handle.local_scope::<_, 1>(|mut frame| unsafe {
                            Value::eval_string(&mut frame, "2 + 3")
                                .unwrap()
                                .unbox::<isize>()
                        })
                    })
                });

                assert_eq!(t1.unwrap(), 3);
                assert_eq!(t2.join().unwrap().unwrap(), 5);
            })
            .unwrap();
    }
}
