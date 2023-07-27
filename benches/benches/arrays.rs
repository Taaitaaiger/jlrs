use std::ptr::NonNull;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jlrs::{
    data::{managed::array::ArrayType, types::construct_type::ConstructType},
    memory::{gc::Gc, target::frame::GcFrame},
    prelude::*,
};
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

#[inline(never)]
fn construct_array_1d_unrooted(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    c.bench_function("Array<f64,1>_unrooted", |b| {
        b.iter(|| Array::new::<f64, _, _>(&frame, 16))
    });
    Ok(())
}

#[inline(never)]
fn construct_array_2d_err_unrooted(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    c.bench_function("Array<f64,2>_err_unrooted", |b| {
        b.iter(|| Array::new::<f64, _, _>(&frame, (16, usize::MAX / 2)).unwrap_err())
    });
    Ok(())
}

#[inline(never)]
fn construct_array_1d_unchecked_unrooted(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    c.bench_function("Array<f64,1>_unchecked_unrooted", |b| {
        b.iter(|| unsafe { Array::new_unchecked::<f64, _, _>(&frame, 16) })
    });
    Ok(())
}

#[inline(never)]
fn construct_array_2d_unrooted(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    c.bench_function("Array<f64,2>_unrooted", |b| {
        b.iter(|| Array::new::<f64, _, _>(&frame, (4, 4)))
    });
    Ok(())
}

#[inline(never)]
fn construct_array_2d_unchecked_unrooted(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    c.bench_function("Array<f64,2>_unchecked_unrooted", |b| {
        b.iter(|| unsafe { Array::new_unchecked::<f64, _, _>(&frame, (4, 4)) })
    });
    Ok(())
}

#[inline(never)]
fn construct_array_3d_unrooted(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    c.bench_function("Array<f64,3>_unrooted", |b| {
        b.iter(|| Array::new::<f64, _, _>(&frame, (4, 2, 2)))
    });
    Ok(())
}

#[inline(never)]
fn construct_array_3d_unchecked_unrooted(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    c.bench_function("Array<f64,3>_unchecked_unrooted", |b| {
        b.iter(|| unsafe { Array::new_unchecked::<f64, _, _>(&frame, (4, 2, 2)) })
    });
    Ok(())
}

#[inline(never)]
fn construct_array_3d_arrdim_unrooted(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    c.bench_function("Array<f64,3>_arrdim_unrooted", |b| {
        b.iter(|| Array::new::<f64, _, _>(&frame, [4, 2, 2]))
    });
    Ok(())
}

#[inline(never)]
fn construct_array_3d_arrdim_unchecked_unrooted(
    frame: &mut GcFrame,
    c: &mut Criterion,
) -> JlrsResult<()> {
    c.bench_function("Array<f64,3>_arrdim_unchecked_unrooted", |b| {
        b.iter(|| unsafe { Array::new_unchecked::<f64, _, _>(&frame, [4, 2, 2]) })
    });
    Ok(())
}

#[inline(never)]
fn construct_array_4d_unrooted(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    c.bench_function("Array<f64,4>_unrooted", |b| {
        b.iter(|| Array::new::<f64, _, _>(&frame, (2, 2, 2, 2)))
    });
    Ok(())
}

#[inline(never)]
fn construct_array_4d_unchecked_unrooted(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    c.bench_function("Array<f64,4>_unchecked_unrooted", |b| {
        b.iter(|| unsafe { Array::new_unchecked::<f64, _, _>(&frame, (2, 2, 2, 2)) })
    });
    Ok(())
}

#[inline(never)]
fn construct_array_1d_from_slice_unrooted(
    frame: &mut GcFrame,
    c: &mut Criterion,
) -> JlrsResult<()> {
    let mut v = [1.0; 16];
    let mut v = NonNull::from(&mut v);
    c.bench_function("Array<f64,1>_from_slice_unrooted", |b| {
        b.iter(|| {
            let v = unsafe { v.as_mut() };
            Array::from_slice(&frame, v, 16)
        })
    });
    Ok(())
}

#[inline(never)]
fn construct_array_1d_from_slice_unchecked_unrooted(
    frame: &mut GcFrame,
    c: &mut Criterion,
) -> JlrsResult<()> {
    let mut v = [1.0; 16];
    let mut v = NonNull::from(&mut v);
    c.bench_function("Array<f64,1>_from_slice_unchecked_unrooted", |b| {
        b.iter(|| {
            let v = unsafe { v.as_mut() };
            unsafe { Array::from_slice_unchecked(&frame, v, 16) }
        })
    });
    Ok(())
}

#[inline(never)]
fn construct_array_1d_from_vec_unrooted(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    c.bench_function("Array<f64,1>_from_vec_unrooted", |b| {
        b.iter(|| {
            let v: Vec<f64> = Vec::new();
            Array::from_vec(&frame, v, 0)
        })
    });
    Ok(())
}

#[inline(never)]
fn construct_array_1d_from_vec_unchecked_unrooted(
    frame: &mut GcFrame,
    c: &mut Criterion,
) -> JlrsResult<()> {
    c.bench_function("Array<f64,1>_from_vec_unchecked_unrooted", |b| {
        b.iter(|| {
            let v: Vec<f64> = Vec::new();
            unsafe { Array::from_vec_unchecked(&frame, v, 0) }
        })
    });
    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    unsafe {
        let mut frame = StackFrame::new();
        let mut julia = RuntimeBuilder::new().start().unwrap();
        let mut julia = julia.instance(&mut frame);

        julia
            .scope(|mut frame| {
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_2d_err_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_1d_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_1d_unchecked_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_2d_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_2d_unchecked_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_3d_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_3d_unchecked_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_3d_arrdim_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_3d_arrdim_unchecked_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_4d_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_4d_unchecked_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_1d_from_slice_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_1d_from_slice_unchecked_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_1d_from_vec_unrooted(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                construct_array_1d_from_vec_unchecked_unrooted(&mut frame, c).unwrap();

                Ok(())
            })
            .unwrap();
    }
}

fn opts() -> Option<Options<'static>> {
    let mut opts = Options::default();
    opts.image_width = Some(1920);
    opts.min_width = 0.01;
    Some(opts)
}

criterion_group! {
    name = arrays;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

criterion_main!(arrays);
