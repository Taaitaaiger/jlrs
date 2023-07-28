use std::ptr::NonNull;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jlrs::{memory::target::frame::GcFrame, prelude::*};
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

#[inline(never)]
fn push_pop_frame_dynamic(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| unsafe {
        let frame = &mut frame;
        let mut frame = NonNull::new_unchecked(frame as *mut GcFrame);

        c.bench_function("push_pop_frame_dynamic", |b| {
            b.iter(|| {
                let frame = frame.as_mut();
                frame.scope(|_| black_box(Ok(())))
            })
        });
        Ok(())
    })
}

#[inline(never)]
fn push_pop_frame_local(frame: &mut GcFrame, c: &mut Criterion) -> JlrsResult<()> {
    frame.scope(|mut frame| unsafe {
        let frame = &mut frame;
        let mut frame = NonNull::new_unchecked(frame as *mut GcFrame);

        c.bench_function("push_pop_frame_local", |b| {
            b.iter(|| {
                let frame = frame.as_mut();
                frame.local_scope::<_, _, 0>(|_| black_box(Ok(())))
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
                push_pop_frame_dynamic(&mut frame, c).unwrap();
                push_pop_frame_local(&mut frame, c).unwrap();

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
    name = frames;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

criterion_main!(frames);
