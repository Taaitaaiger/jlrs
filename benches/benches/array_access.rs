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
fn access_array_1d_index_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = Array::new::<f64, _, _>(&mut frame, 16).unwrap();
        let accessor = unsafe { arr.bits_data::<f64>().unwrap() };

        c.bench_function("Array<f64,1>_access_index", |b| {
            b.iter(|| accessor[black_box(12)])
        });
        Ok(())
    })
}

#[inline(never)]
fn set_array_1d_index_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let mut arr = Array::new::<f64, _, _>(&mut frame, 16).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut::<f64>().unwrap() };

        c.bench_function("Array<f64,1>_set_index", |b| {
            b.iter(|| accessor[black_box(12)] = 1.0)
        });
        Ok(())
    })
}

#[inline(never)]
fn access_array_1d_get_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = Array::new::<f64, _, _>(&mut frame, 16).unwrap();
        let accessor = unsafe { arr.bits_data::<f64>().unwrap() };

        c.bench_function("Array<f64,1>_access_get", |b| {
            b.iter(|| accessor.get(black_box(12)))
        });
        Ok(())
    })
}

#[inline(never)]
fn set_array_1d_set_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let mut arr = Array::new::<f64, _, _>(&mut frame, 16).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut::<f64>().unwrap() };

        c.bench_function("Array<f64,1>_set_set", |b| {
            b.iter(|| accessor.set(black_box(12), black_box(1.0)))
        });

        Ok(())
    })
}

#[inline(never)]
fn access_array_2d_index_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = Array::new::<f64, _, _>(&mut frame, (4, 4)).unwrap();
        let accessor = unsafe { arr.bits_data::<f64>().unwrap() };

        c.bench_function("Array<f64,2>_access_index", |b| {
            b.iter(|| accessor[black_box((2, 3))])
        });
        Ok(())
    })
}

#[inline(never)]
fn set_array_2d_index_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let mut arr = Array::new::<f64, _, _>(&mut frame, (4, 4)).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut::<f64>().unwrap() };

        c.bench_function("Array<f64,2>_set_index", |b| {
            b.iter(|| accessor[black_box((2, 2))] = 1.0)
        });
        Ok(())
    })
}

#[inline(never)]
fn access_array_2d_get_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = Array::new::<f64, _, _>(&mut frame, (4, 4)).unwrap();
        let accessor = unsafe { arr.bits_data::<f64>().unwrap() };

        c.bench_function("Array<f64,2>_access_get", |b| {
            b.iter(|| accessor.get(black_box((2, 3))))
        });
        Ok(())
    })
}

#[inline(never)]
fn set_array_2d_set_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let mut arr = Array::new::<f64, _, _>(&mut frame, (4, 4)).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut::<f64>().unwrap() };

        c.bench_function("Array<f64,2>_set_set", |b| {
            b.iter(|| accessor.set(black_box((2, 2)), 1.0))
        });
        Ok(())
    })
}

#[inline(never)]
fn access_array_2d_index_arr_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = Array::new::<f64, _, _>(&mut frame, (4, 4)).unwrap();
        let accessor = unsafe { arr.bits_data::<f64>().unwrap() };

        c.bench_function("Array<f64,2>_access_arr_index", |b| {
            b.iter(|| accessor[black_box([2, 3])])
        });
        Ok(())
    })
}

#[inline(never)]
fn set_array_2d_index_arr_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let mut arr = Array::new::<f64, _, _>(&mut frame, (4, 4)).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut::<f64>().unwrap() };

        c.bench_function("Array<f64,2>_set_arr_index", |b| {
            b.iter(|| accessor[black_box([2, 2])] = 1.0)
        });
        Ok(())
    })
}

#[inline(never)]
fn access_array_2d_get_arr_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let arr = Array::new::<f64, _, _>(&mut frame, (4, 4)).unwrap();
        let accessor = unsafe { arr.bits_data::<f64>().unwrap() };

        c.bench_function("Array<f64,2>_access_arr_get", |b| {
            b.iter(|| accessor.get(black_box([2, 3])))
        });
        Ok(())
    })
}

#[inline(never)]
fn set_array_2d_set_arr_bits(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let mut arr = Array::new::<f64, _, _>(&mut frame, (4, 4)).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut::<f64>().unwrap() };

        c.bench_function("Array<f64,2>_set_arr_set", |b| {
            b.iter(|| accessor.set(black_box([2, 2]), 1.0))
        });
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
                access_array_1d_index_bits(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                set_array_1d_index_bits(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                access_array_1d_get_bits(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                set_array_1d_set_bits(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                access_array_2d_index_bits(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                set_array_2d_index_bits(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                access_array_2d_get_bits(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                set_array_2d_set_bits(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                access_array_2d_index_arr_bits(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                set_array_2d_index_arr_bits(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                set_array_2d_set_arr_bits(&mut frame, c).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                access_array_2d_get_arr_bits(&mut frame, c).unwrap();

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
    name = array_access;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

criterion_main!(array_access);
