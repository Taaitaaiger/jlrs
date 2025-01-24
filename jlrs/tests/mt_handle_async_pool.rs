#[cfg(all(feature = "multi-rt", feature = "tokio-rt"))]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
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
