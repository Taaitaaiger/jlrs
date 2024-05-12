#[macro_use]
extern crate criterion;
use criterion::{black_box, Criterion};
use jlrs::{
    memory::{gc::Gc, scope::Scope, target::frame::GcFrame},
    prelude::{Call, JlrsResult, Value},
    runtime::{builder::Builder, handle::with_stack::WithStack},
};
#[cfg(not(target_os = "windows"))]
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

#[inline(never)]
fn call_0_unchecked(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|frame| {
        c.bench_function("call_0_unchecked", |b| {
            b.iter(|| {
                let vs = [];
                black_box(unsafe { func.call_unchecked(&frame, vs) })
            })
        });
        Ok(())
    })
}

#[inline(never)]
fn call_0(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|frame| {
        c.bench_function("call_0", |b| {
            b.iter(|| black_box(unsafe { func.call0(black_box(&frame)) }))
        });
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
                black_box(unsafe { func.call_unchecked(black_box(&frame), vs) })
            })
        });
        Ok(())
    })
}

#[inline(never)]
fn call_1(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let v = Value::new(&mut frame, 0usize);
        c.bench_function("call_1", |b| {
            b.iter(|| black_box(unsafe { func.call1(black_box(&frame), v) }))
        });
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
                black_box(unsafe { func.call_unchecked(black_box(&frame), vs) })
            })
        });
        Ok(())
    })
}

#[inline(never)]
fn call_2(frame: &mut GcFrame, c: &mut Criterion, func: Value) -> JlrsResult<()> {
    frame.scope(|mut frame| {
        let v = Value::new(&mut frame, 0usize);
        c.bench_function("call_2", |b| {
            b.iter(|| black_box(unsafe { func.call2(black_box(&frame), v, v) }))
        });
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
                black_box(unsafe { func.call_unchecked(black_box(&frame), vs) })
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
            b.iter(|| unsafe { func.call3(black_box(&frame), v, v, v) })
        });
        Ok(())
    })
}

fn bench_group(c: &mut Criterion) {
    let mut julia = Builder::new().start_local().unwrap();

    julia.with_stack(|mut stack| {
        stack.scope(|mut frame| {
            let func =unsafe{ 
                Value::eval_string(&frame, "function dummy(a1::Any=nothing, a2::Any=nothing, a3::Any=nothing, a4::Any=nothing, a5::Any=nothing, a6::Any=nothing)
                    @nospecialize a1 a2 a3 a4 a5 a6
                end").unwrap().as_value()
            };
            
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            call_0_unchecked(&mut frame, c, func).unwrap();
            
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            call_0(&mut frame, c, func).unwrap();
            
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            call_1_unchecked(&mut frame, c, func).unwrap();
            
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            call_1(&mut frame, c, func).unwrap();
            
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            call_2_unchecked(&mut frame, c, func).unwrap();
            
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            call_2(&mut frame, c, func).unwrap();
            
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            call_3_unchecked(&mut frame, c, func).unwrap();
            
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
            call_3(&mut frame, c, func).unwrap();
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
    name = call_function;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = bench_group
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = call_function;
    config = Criterion::default();
    targets = bench_group
}

criterion_main!(call_function);
