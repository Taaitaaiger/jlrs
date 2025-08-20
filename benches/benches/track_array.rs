use criterion::{Criterion, black_box, criterion_group, criterion_main};
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
fn array_is_borrowed(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, 16).unwrap().as_value();

        c.bench_function("Array_is_tracked", |b| {
            b.iter(|| black_box(arr.is_tracked()))
        });
        Ok(())
    })
}

#[inline(never)]
fn array_track_shared(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, 16).unwrap();

        c.bench_function("Array_track_shared", |b| {
            b.iter(|| black_box(std::mem::ManuallyDrop::new(arr.track_shared())))
        });
        Ok(())
    })
}

#[inline(never)]
fn array_is_tracked_shared(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, 16).unwrap();
        let b = arr.track_shared().unwrap();

        c.bench_function("Array_is_tracked_shared", |b| {
            b.iter(|| black_box(arr.as_value().is_tracked_shared()))
        });

        std::mem::drop(b);
        Ok(())
    })
}

#[inline(never)]
fn array_track_untrack_shared(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, 16).unwrap();

        c.bench_function("Array_track_untrack_shared", |b| {
            b.iter(|| black_box(arr.track_shared()))
        });
        Ok(())
    })
}

#[inline(never)]
fn array_track_untrack_exclusive(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, 16).unwrap();

        c.bench_function("Array_track_untrack_exclusive", |b| {
            b.iter(|| std::mem::drop(black_box(arr.track_exclusive())))
        });
        Ok(())
    })
}

#[inline(never)]
fn array_is_tracked_exclusive(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f32>::new(&mut frame, 16).unwrap();
        let a = arr.as_value();
        let b = arr.track_exclusive().unwrap();

        c.bench_function("Array_is_tracked_exclusive", |b| {
            b.iter(|| black_box(a.is_tracked_exclusive()))
        });

        std::mem::drop(b);
        Ok(())
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut julia = Builder::new().start_local().unwrap();

    julia.with_stack(|mut stack| {
        stack.scope(|mut frame| {
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            array_is_borrowed(&mut frame, c).unwrap();

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            array_track_shared(&mut frame, c).unwrap();

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            array_is_tracked_shared(&mut frame, c).unwrap();

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            array_track_untrack_shared(&mut frame, c).unwrap();

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            array_track_untrack_exclusive(&mut frame, c).unwrap();

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            array_is_tracked_exclusive(&mut frame, c).unwrap();
        });
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
    name = track_array;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = track_array;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(track_array);
