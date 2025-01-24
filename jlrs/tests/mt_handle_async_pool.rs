#[cfg(all(feature = "multi-rt", feature = "tokio-rt"))]
mod mt_handle {
    use jlrs::prelude::*;

    #[test]
    fn create_pool() {
        Builder::new()
            .async_runtime(Tokio::<3>::new(false))
            .start_mt(|julia, a_h: AsyncHandle| {
                let handle = julia
                    .pool_builder(Tokio::<1>::new(false))
                    .n_workers(2.try_into().unwrap())
                    .spawn();

                assert_eq!(handle.n_workers(), 2);
                std::mem::drop(a_h);
            })
            .unwrap();
    }
}
