use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jlrs::prelude::*;
#[cfg(not(target_os = "windows"))]
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

#[inline(never)]
fn blocking_task(handle: &AsyncHandle, c: &mut Criterion) {
    c.bench_function("blocking_task", |b| {
        b.iter(|| {
            black_box(
                handle
                    .blocking_task(|_| 1usize)
                    .try_dispatch()
                    .unwrap()
                    .blocking_recv()
                    .unwrap(),
            );
        })
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let (handle, th_handle) = Builder::new()
        .async_runtime(Tokio::<1>::new(false))
        .spawn()
        .unwrap();

    blocking_task(&handle, c);
    std::mem::drop(handle);
    th_handle.join().unwrap();
}

#[cfg(not(target_os = "windows"))]
fn opts() -> Option<Options<'static>> {
    let mut opts = Options::default();
    opts.image_width = Some(1920);
    opts.min_width = 0.01;
    Some(opts)
}

#[cfg(not(target_os = "windows"))]
criterion_group! {
    name = async_rt;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = async_rt;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(async_rt);
