use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use jlrs::{
    data::managed::background_task::spawn_background_task, prelude::*, weak_handle_unchecked,
};

#[inline(never)]
fn bench_background_task(c: &mut Criterion) {
    c.bench_function("background_task", |b| {
        let handle = unsafe { weak_handle_unchecked!() };
        let func = unsafe {
            Module::base(&handle)
                .global(&handle, "fetch")
                .unwrap()
                .as_value()
        };

        handle.local_scope::<_, 2>(|mut frame| {
            let mut output1 = frame.output();
            let mut output2 = frame.output();

            b.iter(|| {
                let task = spawn_background_task::<usize, _, _>(&mut output1, || Ok(1usize));
                let _v = black_box(unsafe { func.call(&mut output2, [task.as_value()]).unwrap() });
            })
        })
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let handle = Builder::new().start_local().unwrap();
    bench_background_task(c);
    std::mem::drop(handle)
}

criterion_group! {
    name = background_task;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(background_task);
