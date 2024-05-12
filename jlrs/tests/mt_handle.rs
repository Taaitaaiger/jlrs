#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
mod mt_handle {
    use std::thread;

    use jlrs::{data::managed::value::Value, memory::scope::LocalScope, runtime::builder::Builder};

    #[test]
    fn call_from_spawned_threads() {
        let (mut julia, th) = Builder::new().spawn_mt().unwrap();
        let mut julia2 = julia.clone();

        let t1 = thread::spawn(move || {
            julia.with(|handle| {
                handle.local_scope::<_, 1>(|mut frame| unsafe {
                    Value::eval_string(&mut frame, "1 + 2")
                        .unwrap()
                        .unbox::<isize>()
                })
            })
        });

        let t2 = thread::spawn(move || {
            julia2.with(|handle| {
                handle.local_scope::<_, 1>(|mut frame| unsafe {
                    Value::eval_string(&mut frame, "2 + 3")
                        .unwrap()
                        .unbox::<isize>()
                })
            })
        });

        assert_eq!(t1.join().unwrap().unwrap(), 3);
        assert_eq!(t2.join().unwrap().unwrap(), 5);

        th.join().unwrap();
    }
}
