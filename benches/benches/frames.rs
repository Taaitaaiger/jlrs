use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use jlrs::{
    prelude::*, runtime::handle::local_handle::LocalHandle, weak_handle, weak_handle_unchecked,
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
            unsafe {
                let _ = black_box(weak_handle_unchecked!());
            };
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
            black_box(unsafe {
                handle.unsized_local_scope(black_box(2), |f| {
                    black_box(&f);
                })
            })
        })
    });
}

#[inline(never)]
fn push_pop_frame_local_const_n(handle: &LocalHandle, c: &mut Criterion) {
    c.bench_function("push_pop_frame_local_const_n", |b| {
        b.iter(|| {
            black_box(unsafe {
                handle.unsized_local_scope(2, |f| {
                    black_box(&f);
                })
            })
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

criterion_group! {
    name = frames;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(frames);
