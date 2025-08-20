use std::ptr::NonNull;

use criterion::{Criterion, criterion_group, criterion_main};
use jlrs::{
    memory::{gc::Gc, target::frame::GcFrame},
    prelude::*,
};
#[cfg(not(target_os = "windows"))]
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

#[inline(never)]
fn construct_array_1d_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,1>_unrooted", |b| {
        b.iter(|| TypedArray::<f64>::new(&frame, 16))
    });
}

#[inline(never)]
fn construct_array_1d_unrooted2(frame: &mut GcFrame, c: &mut Criterion) {
    jlrs::define_fast_array_key!(pub Foo, f32, 1);

    c.bench_function("Array<f64,1>_unrooted2", |b| {
        b.iter(|| {
            let x: Result<WeakTypedRankedArray<f32, 1>, _> = Foo::new(&frame, 16);
            x
        })
    });
}

#[inline(never)]
fn construct_array_2d_err_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,2>_err_unrooted", |b| {
        b.iter(|| TypedArray::<f64>::new(&frame, [16, usize::MAX / 2]).unwrap_err())
    });
}

#[inline(never)]
fn construct_array_1d_unchecked_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,1>_unchecked_unrooted", |b| {
        b.iter(|| unsafe { TypedArray::<f64>::new_unchecked(&frame, 16) })
    });
}

#[inline(never)]
fn construct_array_2d_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,2>_unrooted", |b| {
        b.iter(|| TypedArray::<f64>::new(&frame, [4, 4]))
    });
}

#[inline(never)]
fn construct_array_2d_unchecked_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,2>_unchecked_unrooted", |b| {
        b.iter(|| unsafe { TypedArray::<f64>::new_unchecked(&frame, [4, 4]) })
    });
}

#[inline(never)]
fn construct_array_2d_unchecked_unrooted2(frame: &mut GcFrame, c: &mut Criterion) {
    jlrs::define_fast_array_key!(pub Foo, f32, 2);

    c.bench_function("Array<f64,2>_unchecked_unrooted2", |b| {
        b.iter(|| unsafe { Foo::new_unchecked(&frame, [4, 4]) })
    });
}

#[inline(never)]
fn construct_array_3d_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,3>_unrooted", |b| {
        b.iter(|| TypedArray::<f64>::new(&frame, [4, 2, 2]))
    });
}

#[inline(never)]
fn construct_array_3d_unchecked_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,3>_unchecked_unrooted", |b| {
        b.iter(|| unsafe { TypedArray::<f64>::new_unchecked(&frame, [4, 2, 2]) })
    });
}

#[inline(never)]
fn construct_array_3d_arrdim_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,3>_arrdim_unrooted", |b| {
        b.iter(|| TypedArray::<f64>::new(&frame, [4, 2, 2]))
    });
}

#[inline(never)]
fn construct_array_3d_arrdim_unchecked_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,3>_arrdim_unchecked_unrooted", |b| {
        b.iter(|| unsafe { TypedArray::<f64>::new_unchecked(&frame, [4, 2, 2]) })
    });
}

#[inline(never)]
fn construct_array_4d_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,4>_unrooted", |b| {
        b.iter(|| TypedRankedArray::<f64, 4>::new(&frame, [2, 2, 2, 2]))
    });
}

#[inline(never)]
fn construct_array_4d_unchecked_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,4>_unchecked_unrooted", |b| {
        b.iter(|| unsafe { TypedArray::<f64>::new_unchecked(&frame, [2, 2, 2, 2]) })
    });
}

#[inline(never)]
fn construct_array_1d_from_slice_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    let mut v = [1.0; 16];
    let mut v = NonNull::from(&mut v);
    c.bench_function("Array<f64,1>_from_slice_unrooted", |b| {
        b.iter(|| {
            let v = unsafe { v.as_mut() };
            TypedArray::<f64>::from_slice(&frame, v, 16)
        })
    });
}

#[inline(never)]
fn construct_array_1d_from_slice_unchecked_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    let mut v = [1.0; 16];
    let mut v = NonNull::from(&mut v);
    c.bench_function("Array<f64,1>_from_slice_unchecked_unrooted", |b| {
        b.iter(|| {
            let v = unsafe { v.as_mut() };
            unsafe { TypedArray::<f64>::from_slice_unchecked(&frame, v, 16) }
        })
    });
}

#[inline(never)]
fn construct_array_1d_from_vec_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,1>_from_vec_unrooted", |b| {
        b.iter(|| {
            let v: Vec<f64> = Vec::new();
            TypedArray::<f64>::from_vec(&frame, v, 0)
        })
    });
}

#[inline(never)]
fn construct_array_1d_from_vec_unchecked_unrooted(frame: &mut GcFrame, c: &mut Criterion) {
    c.bench_function("Array<f64,1>_from_vec_unchecked_unrooted", |b| {
        b.iter(|| {
            let v: Vec<f64> = Vec::new();
            unsafe { TypedArray::<f64>::from_vec_unchecked(&frame, v, 0) }
        })
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut julia = Builder::new().start_local().unwrap();

    julia.with_stack(|mut stack| {
        stack.scope(|mut frame| {
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_2d_err_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_1d_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_1d_unrooted2(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_1d_unchecked_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_2d_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_2d_unchecked_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_2d_unchecked_unrooted2(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_3d_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_3d_unchecked_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_3d_arrdim_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_3d_arrdim_unchecked_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_4d_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_4d_unchecked_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_1d_from_slice_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_1d_from_slice_unchecked_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_1d_from_vec_unrooted(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            construct_array_1d_from_vec_unchecked_unrooted(&mut frame, c);
        })
    })
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
    name = arrays;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = arrays;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(arrays);
