#[cfg(all(feature = "multi-rt", feature = "tokio-rt"))]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
mod mt_handle {
    use jlrs::prelude::*;

    #[test]
    fn create_pool() {
        let (julia, a_h, th) = Builder::new()
            .async_runtime(Tokio::<3>::new(false))
            .spawn_mt()
            .unwrap();

        let handle = julia
            .pool_builder(Tokio::<1>::new(false))
            .n_workers(2.try_into().unwrap())
            .spawn();

        assert_eq!(handle.n_workers(), 2);

        std::mem::drop(julia);
        std::mem::drop(handle);
        std::mem::drop(a_h);
        th.join().unwrap();
    }
}
