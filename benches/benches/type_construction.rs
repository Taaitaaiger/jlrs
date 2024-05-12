use criterion::{criterion_group, criterion_main, Criterion};
use jlrs::{
    data::types::{
        abstract_type::{AbstractSet, Integer, Number},
        construct_type::{ArrayTypeConstructor, ConstantIsize, ConstructType},
    },
    memory::target::frame::GcFrame,
    prelude::*,
};
#[cfg(not(target_os = "windows"))]
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

#[inline(never)]
fn construct_primitive_type_uncached(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let output = frame.reusable_slot();

        c.bench_function("ConstructType_f64_uncached", |b| {
            b.iter(|| f64::construct_type_uncached(&output))
        });
        Ok(())
    })
}

#[inline(never)]
fn construct_primitive_type_cached(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let output = frame.reusable_slot();

        c.bench_function("ConstructType_f64_cached", |b| {
            b.iter(|| f64::construct_type(&output))
        });
        Ok(())
    })
}

#[inline(never)]
fn construct_abstract_type_uncached(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let output = frame.reusable_slot();

        c.bench_function("ConstructType_number_uncached", |b| {
            b.iter(|| Number::construct_type_uncached(&output))
        });
        Ok(())
    })
}

#[inline(never)]
fn construct_abstract_type_cached(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let output = frame.reusable_slot();

        c.bench_function("ConstructType_number_cached", |b| {
            b.iter(|| Number::construct_type(&output))
        });
        Ok(())
    })
}
#[inline(never)]
fn construct_integer_uncached(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let output = frame.reusable_slot();

        c.bench_function("ConstructType_integer_uncached", |b| {
            b.iter(|| Integer::construct_type_uncached(&output))
        });
        Ok(())
    })
}

#[inline(never)]
fn construct_integer_cached(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let output = frame.reusable_slot();

        c.bench_function("ConstructType_integer_cached", |b| {
            b.iter(|| Integer::construct_type(&output))
        });
        Ok(())
    })
}

#[inline(never)]
fn construct_abstract_parametric_type_uncached(
    frame: &mut GcFrame,
    c: &mut Criterion,
) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let output = frame.reusable_slot();

        c.bench_function("ConstructType_abstract_set_uncached", |b| {
            b.iter(|| AbstractSet::<f64>::construct_type_uncached(&output))
        });
        Ok(())
    })
}

#[inline(never)]
fn construct_abstract_parametric_type_cached(
    frame: &mut GcFrame,
    c: &mut Criterion,
) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let output = frame.reusable_slot();

        c.bench_function("ConstructType_abstract_set_cached", |b| {
            b.iter(|| AbstractSet::<f64>::construct_type(&output))
        });
        Ok(())
    })
}

#[inline(never)]
fn construct_array_type_uncached(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let output = frame.reusable_slot();

        c.bench_function("ConstructType_array_f64_3_uncached", |b| {
            b.iter(|| {
                ArrayTypeConstructor::<f64, ConstantIsize<3>>::construct_type_uncached(&output)
            })
        });
        Ok(())
    })
}

#[inline(never)]
fn construct_array_type_cached(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let output = frame.reusable_slot();

        c.bench_function("ConstructType_array_f64_3_cached", |b| {
            b.iter(|| ArrayTypeConstructor::<f64, ConstantIsize<3>>::construct_type(&output))
        });
        Ok(())
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    Builder::new()
        .start_local()
        .unwrap()
        .with_stack(|mut stack| {
            stack.scope(|mut frame| {
                construct_primitive_type_uncached(&mut frame, c).unwrap();
                construct_primitive_type_cached(&mut frame, c).unwrap();
                construct_abstract_type_uncached(&mut frame, c).unwrap();
                construct_abstract_type_cached(&mut frame, c).unwrap();
                construct_integer_uncached(&mut frame, c).unwrap();
                construct_integer_cached(&mut frame, c).unwrap();
                construct_abstract_parametric_type_uncached(&mut frame, c).unwrap();
                construct_abstract_parametric_type_cached(&mut frame, c).unwrap();
                construct_array_type_uncached(&mut frame, c).unwrap();
                construct_array_type_cached(&mut frame, c).unwrap();
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
    name = type_construction;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = type_construction;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(type_construction);
