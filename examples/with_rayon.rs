use jlrs::prelude::*;
use rayon::prelude::*;

fn main() {
    Builder::new()
        .start_mt(|handle| {
            // A separate thread pool must be used to ensure these threads don't outlive this
            // thread. Using the global pool to call into Julia can cause a deadlock in
            // `Builder::start_mt` after returning from this closure.
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(2)
                .build()
                .unwrap();

            let n = 100;
            let arr = (0i32..n).collect::<Vec<_>>();

            // `ThreadPool::install` executes the operation within the given threadpool
            let res = pool.install(|| {
                // We can use the `ParallelIterator::*_with` methods to propagate the handle
                arr.par_iter()
                    .map_with(handle, |mt_handle, v| {
                        // Don't yield while an `ActiveHandle` exists
                        mt_handle.with(|active_handle| {
                            active_handle.local_scope::<_, 1>(|mut frame| {
                                let val = Value::new(&mut frame, *v);
                                val.unbox::<i32>().unwrap() + 1
                            })
                        })
                    })
                    .collect::<Vec<_>>()
            });

            assert_eq!(res, (1..n + 1).collect::<Vec<_>>());
        })
        .unwrap();
}
