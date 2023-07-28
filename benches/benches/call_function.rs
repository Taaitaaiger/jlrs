#[macro_use]
extern crate criterion;
use criterion::Criterion;
use jlrs::{
    memory::{gc::Gc, target::frame::GcFrame},
    prelude::{Call, JlrsResult, RuntimeBuilder, StackFrame, Value},
};
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

// Thanks to the example provided by @jebbow in his article
// https://www.jibbow.com/posts/criterion-flamegraphs/

#[inline(never)]
fn call_0_unchecked(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|frame| {
        c.bench_function("call_0_unchecked", |b| {
            b.iter(|| {
                let vs = [];
                unsafe { func.call_unchecked(&frame, vs) }
            })
        });
        Ok(())
    })
}

#[inline(never)]
fn call_0(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|frame| {
        c.bench_function("call_0", |b| b.iter(|| unsafe { func.call0(&frame) }));
        Ok(())
    })
}

#[inline(never)]
fn call_1_unchecked(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let v = Value::new(&mut frame, 0usize);
        c.bench_function("call_1_unchecked", |b| {
            b.iter(|| {
                let vs = [v];
                unsafe { func.call_unchecked(&frame, vs) }
            })
        });
        Ok(())
    })
}

#[inline(never)]
fn call_1(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let v = Value::new(&mut frame, 0usize);
        c.bench_function("call_1", |b| b.iter(|| unsafe { func.call1(&frame, v) }));
        Ok(())
    })
}

#[inline(never)]
fn call_2_unchecked(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let v = Value::new(&mut frame, 0usize);
        c.bench_function("call_2_unchecked", |b| {
            b.iter(|| {
                let vs = [v, v];
                unsafe { func.call_unchecked(&frame, vs) }
            })
        });
        Ok(())
    })
}

#[inline(never)]
fn call_2(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let v = Value::new(&mut frame, 0usize);
        c.bench_function("call_2", |b| b.iter(|| unsafe { func.call2(&frame, v, v) }));
        Ok(())
    })
}

#[inline(never)]
fn call_3_unchecked(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let v = Value::new(&mut frame, 0usize);
        c.bench_function("call_3_unchecked", |b| {
            b.iter(|| {
                let vs = [v, v, v];
                unsafe { func.call_unchecked(&frame, vs) }
            })
        });
        Ok(())
    })
}

#[inline(never)]
fn call_3(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let v = Value::new(&mut frame, 0usize);
        c.bench_function("call_3", |b| {
            b.iter(|| unsafe { func.call3(&frame, v, v, v) })
        });
        Ok(())
    })
}

fn bench_group(c: &mut Criterion) {
    unsafe {
        let mut frame = StackFrame::new();
        let mut julia = RuntimeBuilder::new().start().unwrap();
        let mut julia = julia.instance(&mut frame);

        julia
            .scope(|mut frame| {

                let func = Value::eval_string(&frame, "function dummy(a1::Any=nothing, a2::Any=nothing, a3::Any=nothing, a4::Any=nothing, a5::Any=nothing, a6::Any=nothing)
                    @nospecialize a1 a2 a3 a4 a5 a6
                end").unwrap().as_value();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                call_0_unchecked(&mut frame, c, func).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                call_0(&mut frame, c, func).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                call_1_unchecked(&mut frame, c, func).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                call_1(&mut frame, c, func).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                call_2_unchecked(&mut frame, c, func).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                call_2(&mut frame, c, func).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                call_3_unchecked(&mut frame, c, func).unwrap();

                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                call_3(&mut frame, c, func).unwrap();
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
    name = call_function;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = bench_group
}

criterion_main!(call_function);
