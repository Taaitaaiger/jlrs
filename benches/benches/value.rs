use std::ptr::NonNull;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jlrs::{memory::target::frame::GcFrame, prelude::*};
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

#[inline(never)]
fn value_new_usize_unrooted(frame: &GcFrame, c: &mut Criterion) {
    c.bench_function("Value::new::<usize> unrooted", |b| {
        b.iter(|| black_box(Value::new(frame, 1usize)))
    });
}

#[inline(never)]
fn value_new_usize_reusable_slot(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|frame| {
        let mut output = frame.reusable_slot();
        let mut output = NonNull::from(&mut output);

        c.bench_function("Value::new::<usize>_reusable_slot", |b| {
            b.iter(|| {
                let o = unsafe { output.as_mut() };
                black_box(Value::new(o, 1usize))
            })
        });
        Ok(())
    })
}

#[inline(never)]
fn value_new_usize_local_reusable_slot(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.local_scope::<_, _, 1>(|mut frame| {
        let mut output = frame.local_reusable_slot();
        let mut output = NonNull::from(&mut output);

        c.bench_function("Value::new::<usize>_local_reusable_slot", |b| {
            b.iter(|| {
                let o = unsafe { output.as_mut() };
                black_box(Value::new(o, 1usize))
            })
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
                value_new_usize_unrooted(&frame, c);
                value_new_usize_reusable_slot(&mut frame, c).unwrap();
                value_new_usize_local_reusable_slot(&mut frame, c).unwrap();

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
    name = value;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

criterion_main!(value);
