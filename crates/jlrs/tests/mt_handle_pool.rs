#[cfg(all(feature = "multi-rt", feature = "async-rt"))]
mod mt_handle {

    use jlrs::runtime::{builder::Builder, executor::tokio_exec::Tokio};

    #[test]
    fn create_pool() {
        Builder::new()
            .start_mt(|julia| {
                let handle = julia
                    .pool_builder(Tokio::<1>::new(false))
                    .n_workers(2.try_into().unwrap())
                    .spawn();

                assert_eq!(handle.n_workers(), 2);
            })
            .unwrap();
    }
}
