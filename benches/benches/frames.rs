use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jlrs::{
    prelude::*, runtime::handle::local_handle::LocalHandle,
    weak_handle, weak_handle_unchecked,
};
#[cfg(not(target_os = "windows"))]
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

#[inline(never)]
fn create_weak_handle(c: &mut Criterion) {
    c.bench_function("create_weak_handle", |b| {
        b.iter(|| {
            let _ = black_box(weak_handle!());
        })
    });
}

#[inline(never)]
fn create_weak_handle_unchecked(c: &mut Criterion) {
    c.bench_function("create_weak_handle_unchecked", |b| {
        b.iter(|| {
            let _ = black_box(unsafe { weak_handle_unchecked!() });
        })
    });
}

#[inline(never)]
fn push_pop_frame_dynamic(c: &mut Criterion) {
    let mut weak_handle = unsafe { weak_handle_unchecked!() };
    weak_handle.with_stack(|mut stack| {
        c.bench_function("push_pop_frame_dynamic", |b| {
            b.iter(|| {
                black_box(stack.scope(|f| {
                    black_box(&f);
                }))
            })
        });
    });
}

#[inline(never)]
fn push_pop_frame_local(handle: &LocalHandle, c: &mut Criterion) {
    c.bench_function("push_pop_frame_local", |b| {
        b.iter(|| {
            black_box(handle.local_scope::<_, 0>(|f| {
                black_box(&f);
            }))
        })
    });
}

#[inline(never)]
fn push_pop_frame_local_1(handle: &LocalHandle, c: &mut Criterion) {
    c.bench_function("push_pop_frame_local_1", |b| {
        b.iter(|| {
            black_box(handle.local_scope::<_, 1>(|f| {
                black_box(&f);
            }))
        })
    });
}

#[inline(never)]
fn push_pop_frame_local_2(handle: &LocalHandle, c: &mut Criterion) {
    c.bench_function("push_pop_frame_local_2", |b| {
        b.iter(|| {
            black_box(handle.local_scope::<_, 2>(|f| {
                black_box(&f);
            }))
        })
    });
}

#[inline(never)]
fn push_pop_frame_local_n(handle: &LocalHandle, c: &mut Criterion) {
    c.bench_function("push_pop_frame_local_n", |b| {
        b.iter(|| {
            black_box(handle.unsized_local_scope(black_box(2), |f| {
                black_box(&f);
            }))
        })
    });
}

#[inline(never)]
fn push_pop_frame_local_const_n(handle: &LocalHandle, c: &mut Criterion) {
    c.bench_function("push_pop_frame_local_const_n", |b| {
        b.iter(|| {
            black_box(handle.unsized_local_scope(2, |f| {
                black_box(&f);
            }))
        })
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let handle = Builder::new().start_local().unwrap();

    create_weak_handle(c);
    create_weak_handle_unchecked(c);
    push_pop_frame_dynamic(c);
    push_pop_frame_local(&handle, c);
    push_pop_frame_local_1(&handle, c);
    push_pop_frame_local_2(&handle, c);
    push_pop_frame_local_n(&handle, c);
    push_pop_frame_local_const_n(&handle, c);
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
    name = frames;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = frames;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(frames);
