use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use jlrs::{prelude::*, runtime::handle::async_handle::AsyncHandle};

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

criterion_group! {
    name = async_rt;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(async_rt);
