use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jlrs::{
    data::managed::background_task::spawn_background_task, prelude::*, weak_handle_unchecked,
};
#[cfg(not(target_os = "windows"))]
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
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

#[cfg(not(target_os = "windows"))]
fn opts() -> Option<Options<'static>> {
    let mut opts = Options::default();
    opts.image_width = Some(1920);
    opts.min_width = 0.01;
    Some(opts)
}

#[cfg(not(target_os = "windows"))]
criterion_group! {
    name = background_task;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = background_task;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(background_task);
