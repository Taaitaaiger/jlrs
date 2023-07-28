use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jlrs::{
    memory::{gc::Gc, target::frame::GcFrame},
    prelude::*,
};
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

#[inline(never)]
fn array_is_borrowed(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = Array::new::<f64, _, _>(&mut frame, 16).unwrap().as_value();

        c.bench_function("Array_is_tracked", |b| {
            b.iter(|| black_box(arr.is_tracked()))
        });
        Ok(())
    })
}

#[inline(never)]
fn array_track_shared(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = Array::new::<f64, _, _>(&mut frame, 16).unwrap();

        c.bench_function("Array_track_shared", |b| {
            b.iter(|| black_box(std::mem::ManuallyDrop::new(arr.track_shared())))
        });
        Ok(())
    })
}

#[inline(never)]
fn array_is_tracked_shared(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = Array::new::<f64, _, _>(&mut frame, 16).unwrap();
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
        let arr = Array::new::<f64, _, _>(&mut frame, 16).unwrap();

        c.bench_function("Array_track_untrack_shared", |b| {
            b.iter(|| black_box(arr.track_shared()))
        });
        Ok(())
    })
}

#[inline(never)]
fn array_track_untrack_exclusive(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let mut arr = Array::new::<f64, _, _>(&mut frame, 16).unwrap();

        c.bench_function("Array_track_untrack_exclusive", |b| {
            b.iter(|| unsafe { std::mem::drop(black_box(arr.track_exclusive())) })
        });
        Ok(())
    })
}

#[inline(never)]
fn array_is_tracked_exclusive(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let mut arr = Array::new::<f32, _, _>(&mut frame, 16).unwrap();
        let a = arr.as_value();
        let b = unsafe { arr.track_exclusive().unwrap() };

        c.bench_function("Array_is_tracked_exclusive", |b| {
            b.iter(|| black_box(a.is_tracked_exclusive()))
        });

        std::mem::drop(b);
        Ok(())
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    unsafe {
        let mut frame = StackFrame::new();
        let mut julia = RuntimeBuilder::new().start().unwrap();
        let mut julia = julia.instance(&mut frame);

        julia
            .scope(|mut frame| {
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

                // frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                // set_array_1d_index_bits(&mut frame, c).unwrap();

                // frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                // access_array_1d_get_bits(&mut frame, c).unwrap();

                // frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                // set_array_1d_set_bits(&mut frame, c).unwrap();

                // frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                // access_array_2d_index_bits(&mut frame, c).unwrap();

                // frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                // set_array_2d_index_bits(&mut frame, c).unwrap();

                // frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                // access_array_2d_get_bits(&mut frame, c).unwrap();

                // frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                // set_array_2d_set_bits(&mut frame, c).unwrap();

                // frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                // access_array_2d_index_arr_bits(&mut frame, c).unwrap();

                // frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                // set_array_2d_index_arr_bits(&mut frame, c).unwrap();

                // frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                // set_array_2d_set_arr_bits(&mut frame, c).unwrap();

                // frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                // access_array_2d_get_arr_bits(&mut frame, c).unwrap();

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
    name = track_array;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

criterion_main!(track_array);
