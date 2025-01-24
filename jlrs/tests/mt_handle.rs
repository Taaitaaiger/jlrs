#[cfg(feature = "multi-rt")]
mod mt_handle {
    use jlrs::{data::managed::value::Value, memory::scope::LocalScope, runtime::builder::Builder};

    #[test]
    fn call_from_spawned_threads() {
        Builder::new()
            .start_mt(|julia| {
                let t1 = julia.spawn(move |mut julia| {
                    julia.with(|handle| {
                        handle.local_scope::<_, 1>(|mut frame| unsafe {
                            Value::eval_string(&mut frame, "1 + 2")
                                .unwrap()
                                .unbox::<isize>()
                        })
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

                assert_eq!(t1.join().unwrap().unwrap(), 3);
                assert_eq!(t2.join().unwrap().unwrap(), 5);
            })
            .unwrap();
    }
}
