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
fn array_1d_accessor_slice(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, 16).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("Array<f64,1>_slice", |b| {
            b.iter(|| black_box(accessor.as_slice()))
        });
    })
}

#[inline(never)]
fn ranked_array_1d_accessor_slice(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedVector::<f64>::new(&mut frame, 16).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("TypedVector<f64>_slice", |b| {
            b.iter(|| black_box(accessor.as_slice()))
        });
    })
}

#[inline(never)]
fn array_1d_accessor_mut_slice(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedArray::<f64>::new(&mut frame, 16).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("Array<f64,1>_mut_slice", |b| {
            b.iter(|| {
                black_box(accessor.as_mut_slice());
            })
        });
    })
}

#[inline(never)]
fn ranked_array_1d_accessor_mut_slice(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedVector::<f64>::new(&mut frame, 16).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("TypedVector<f64>_mut_slice", |b| {
            b.iter(|| {
                black_box(accessor.as_mut_slice());
            })
        });
    })
}

#[inline(never)]
fn access_array_1d_index_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, 16).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("Array<f64,1>_access_index", |b| {
            b.iter(|| black_box(accessor[12]))
        });
    })
}

#[inline(never)]
fn access_ranked_array_1d_index_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedVector::<f64>::new(&mut frame, 16).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("TypedVector<f64>_access_index", |b| {
            b.iter(|| black_box(accessor[12]))
        });
    })
}

#[inline(never)]
fn set_array_1d_index_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedArray::<f64>::new(&mut frame, 16).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("Array<f64,1>_set_index", |b| {
            b.iter(|| black_box(accessor[12] = 1.0))
        });
    })
}

#[inline(never)]
fn set_ranked_array_1d_index_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedVector::<f64>::new(&mut frame, 16).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("TypedVector<f64>_set_index", |b| {
            b.iter(|| black_box(accessor[12] = 1.0))
        });
    })
}

#[inline(never)]
fn access_array_1d_get_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, 16).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("Array<f64,1>_access_get", |b| {
            b.iter(|| black_box(accessor.get(12)))
        });
    })
}

#[inline(never)]
fn access_ranked_array_1d_get_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedVector::<f64>::new(&mut frame, 16).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("TypedVector<f64>_access_get", |b| {
            b.iter(|| black_box(accessor.get(12)))
        });
    })
}

#[inline(never)]
fn set_array_1d_set_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedArray::<f64>::new(&mut frame, 16).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("Array<f64,1>_set_set", |b| {
            b.iter(|| black_box(accessor.set(12, 1.0)))
        });
    })
}

#[inline(never)]
fn set_ranked_array_1d_set_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedVector::<f64>::new(&mut frame, 16).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("TypedVector<f64>_set_set", |b| {
            b.iter(|| black_box(accessor.set(12, 1.0)))
        });
    })
}

#[inline(never)]
fn access_array_2d_index_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, [4, 4]).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("Array<f64,2>_access_index", |b| {
            b.iter(|| black_box(accessor[[2, 3]]))
        });
    })
}

#[inline(never)]
fn access_ranked_array_2d_index_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedMatrix::<f64>::new(&mut frame, [4, 4]).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("TypedMatrix<f64>_access_index", |b| {
            b.iter(|| black_box(accessor[[2, 3]]))
        });
    })
}

#[inline(never)]
fn set_array_2d_index_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedArray::<f64>::new(&mut frame, [4, 4]).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("Array<f64,2>_set_index", |b| {
            b.iter(|| black_box(accessor[[2, 2]] = 1.0))
        });
    })
}

#[inline(never)]
fn set_ranked_array_2d_index_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedMatrix::<f64>::new(&mut frame, [4, 4]).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("TypedMatrix<f64>_set_index", |b| {
            b.iter(|| black_box(accessor[[2, 2]] = 1.0))
        });
    })
}

#[inline(never)]
fn access_array_2d_get_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, [4, 4]).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("Array<f64,2>_access_get", |b| {
            b.iter(|| black_box(accessor.get([2, 3])))
        });
    })
}

#[inline(never)]
fn access_ranked_array_2d_get_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedMatrix::<f64>::new(&mut frame, [4, 4]).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("TypedMatrix<f64>_access_get", |b| {
            b.iter(|| black_box(accessor.get([2, 3])))
        });
    })
}

#[inline(never)]
fn set_array_2d_set_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedArray::<f64>::new(&mut frame, [4, 4]).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("Array<f64,2>_set_set", |b| {
            b.iter(|| black_box(accessor.set([2, 2], 1.0)))
        });
    })
}

#[inline(never)]
fn set_ranked_array_2d_set_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedMatrix::<f64>::new(&mut frame, [4, 4]).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("TypedMatrix<f64>_set_set", |b| {
            b.iter(|| black_box(accessor.set([2, 2], 1.0)))
        });
    })
}

#[inline(never)]
fn access_array_2d_index_arr_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, [4, 4]).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("Array<f64,2>_access_arr_index", |b| {
            b.iter(|| black_box(accessor[[2, 3]]))
        });
    })
}

#[inline(never)]
fn access_ranked_array_2d_index_arr_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedMatrix::<f64>::new(&mut frame, [4, 4]).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("TypedMatrix<f64>_access_arr_index", |b| {
            b.iter(|| black_box(accessor[[2, 3]]))
        });
    })
}

#[inline(never)]
fn set_array_2d_index_arr_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedArray::<f64>::new(&mut frame, [4, 4]).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("Array<f64,2>_set_arr_index", |b| {
            b.iter(|| black_box(accessor[[2, 2]] = 1.0))
        });
    })
}

#[inline(never)]
fn set_ranked_array_2d_index_arr_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedMatrix::<f64>::new(&mut frame, [4, 4]).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("TypedMatrix<f64>_set_arr_index", |b| {
            b.iter(|| black_box(accessor[[2, 2]] = 1.0))
        });
    })
}

#[inline(never)]
fn access_array_2d_get_arr_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedArray::<f64>::new(&mut frame, [4, 4]).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("Array<f64,2>_access_arr_get", |b| {
            b.iter(|| black_box(accessor.get([2, 3])))
        });
    })
}

#[inline(never)]
fn access_ranked_array_2d_get_arr_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let arr = TypedMatrix::<f64>::new(&mut frame, [4, 4]).unwrap();
        let accessor = unsafe { arr.bits_data() };

        c.bench_function("TypedMatrix<f64>_access_arr_get", |b| {
            b.iter(|| black_box(accessor.get([2, 3])))
        });
    })
}

#[inline(never)]
fn set_array_2d_set_arr_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedArray::<f64>::new(&mut frame, [4, 4]).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("Array<f64,2>_set_arr_set", |b| {
            b.iter(|| black_box(accessor.set([2, 2], 1.0)))
        });
    })
}

#[inline(never)]
fn set_ranked_array_2d_set_arr_bits(frame: &mut GcFrame, c: &mut Criterion) {
    frame.scope(|mut frame| {
        let mut arr = TypedMatrix::<f64>::new(&mut frame, [4, 4]).unwrap();
        let mut accessor = unsafe { arr.bits_data_mut() };

        c.bench_function("TypedMatrix<f64>_set_arr_set", |b| {
            b.iter(|| black_box(accessor.set([2, 2], 1.0)))
        });
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut julia = Builder::new().start_local().unwrap();

    julia.with_stack(|mut stack| {
        stack.scope(|mut frame| {
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            array_1d_accessor_slice(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            ranked_array_1d_accessor_slice(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            array_1d_accessor_mut_slice(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            ranked_array_1d_accessor_mut_slice(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_array_1d_index_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_ranked_array_1d_index_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_array_1d_index_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_ranked_array_1d_index_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_array_1d_get_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_ranked_array_1d_get_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_array_1d_set_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_ranked_array_1d_set_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_array_2d_index_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_ranked_array_2d_index_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_array_2d_index_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_ranked_array_2d_index_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_array_2d_get_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_ranked_array_2d_get_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_array_2d_set_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_ranked_array_2d_set_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_array_2d_index_arr_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_ranked_array_2d_index_arr_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_array_2d_index_arr_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_ranked_array_2d_index_arr_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_array_2d_set_arr_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            set_ranked_array_2d_set_arr_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_array_2d_get_arr_bits(&mut frame, c);

            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            access_ranked_array_2d_get_arr_bits(&mut frame, c);
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
    name = array_access;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = array_access;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(array_access);
