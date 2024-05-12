use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jlrs::{
    convert::to_symbol::ToSymbol,
    data::managed::symbol::static_symbol::{sym, StaticSymbol, Sym},
    define_static_binary_symbol, define_static_symbol,
    prelude::*,
};
#[cfg(not(target_os = "windows"))]
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};

define_static_symbol!(WhereSym, "where");
define_static_binary_symbol!(WhereSym2, b"where");

fn criterion_benchmark(c: &mut Criterion) {
    let mut julia = Builder::new().start_local().unwrap();

    julia.with_stack(|mut stack| {
        stack.scope(|frame| {
            c.bench_function("StaticSymbolRef", |b| {
                b.iter(|| black_box(WhereSym::get_symbol(&frame)))
            });

            c.bench_function("Sym", |b| {
                b.iter(|| black_box(Sym::new(&frame, WhereSym).to_symbol(&frame)))
            });

            c.bench_function("SymPhantom", |b| {
                b.iter(|| black_box(sym::<WhereSym, _>(&frame).to_symbol(&frame)))
            });

            c.bench_function("StaticBinarySymbolRef", |b| {
                b.iter(|| black_box(WhereSym2::get_symbol(&frame)))
            });

            c.bench_function("BinarySym", |b| {
                b.iter(|| black_box(Sym::new(&frame, WhereSym2).to_symbol(&frame)))
            });

            c.bench_function("BinarySymPhantom", |b| {
                b.iter(|| black_box(sym::<WhereSym2, _>(&frame).to_symbol(&frame)))
            });
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
    name = static_symbol;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = static_symbol;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(static_symbol);
