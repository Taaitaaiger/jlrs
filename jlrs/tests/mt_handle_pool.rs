#[cfg(all(feature = "multi-rt", feature = "async-rt"))]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
mod mt_handle {

    use jlrs::runtime::{builder::Builder, executor::tokio_exec::Tokio};

    #[test]
    fn create_pool() {
        let (julia, th) = Builder::new().spawn_mt().unwrap();

        let handle = julia
            .pool_builder(Tokio::<1>::new(false))
            .n_workers(2.try_into().unwrap())
            .spawn();

        assert_eq!(handle.n_workers(), 2);

        std::mem::drop(julia);
        std::mem::drop(handle);
        th.join().unwrap();
    }
}
