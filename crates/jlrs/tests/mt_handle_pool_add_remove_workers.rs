#[cfg(all(feature = "multi-rt", feature = "async-rt"))]
mod mt_handle {

    use std::{thread::sleep, time::Duration};

    use jlrs::runtime::{builder::Builder, executor::tokio_exec::Tokio};

    #[test]
    fn add_remove_workers() {
        Builder::new()
            .start_mt(|julia| {
                let handle = julia
                    .pool_builder(Tokio::<1>::new(false))
                    .n_workers(1.try_into().unwrap())
                    .spawn();
                assert_eq!(handle.n_workers(), 1);

                assert!(handle.try_add_worker());
                while handle.n_workers() == 1 {
                    sleep(Duration::from_millis(1));
                }
                assert_eq!(handle.n_workers(), 2);

                assert!(handle.try_remove_worker());
                while handle.n_workers() == 2 {
                    sleep(Duration::from_millis(1));
                }
                assert_eq!(handle.n_workers(), 1);

                assert!(handle.try_remove_worker());
                while handle.n_workers() == 1 {
                    sleep(Duration::from_millis(1));
                }
                assert_eq!(handle.n_workers(), 0);
                assert!(handle.is_closed());

                assert!(!handle.try_add_worker());
                assert!(!handle.try_remove_worker());
            })
            .unwrap();
    }
}
