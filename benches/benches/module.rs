use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jlrs::{data::managed::module::JlrsCore, memory::target::frame::GcFrame, prelude::*};
#[cfg(not(target_os = "windows"))]
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

#[inline(never)]
fn module_submodule(frame: &GcFrame, c: &mut Criterion) {
    c.bench_function("Module::submodule", |b| {
        b.iter(|| Module::jlrs_core(frame).submodule(frame, black_box("Wrap")))
    });
}

#[inline(never)]
fn module_submodule_cached(frame: &GcFrame, c: &mut Criterion) {
    c.bench_function("Module::submodule cached", |b| {
        b.iter(|| JlrsCore::module(&frame))
    });
}

#[inline(never)]
fn module_global(frame: &GcFrame, c: &mut Criterion) {
    c.bench_function("Module::global", |b| {
        b.iter(|| unsafe {
            Module::main(frame)
                .submodule(frame, black_box("Base"))
                .unwrap()
                .as_managed()
                .global(frame, black_box("+"))
                .unwrap()
        })
    });
}

#[inline(never)]
fn module_global_cached(frame: &GcFrame, c: &mut Criterion) {
    c.bench_function("Module::global_cached", |b| unsafe {
        b.iter(|| Module::typed_global_cached::<Value, _, _>(frame, black_box("Main.Base.+")))
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut julia = Builder::new().start_local().unwrap();

    julia.with_stack(|mut stack| {
        stack.scope(|frame| {
            module_submodule(&frame, c);
            module_submodule_cached(&frame, c);
            module_global(&frame, c);
            module_global_cached(&frame, c);
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
    name = module;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = module;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(module);
