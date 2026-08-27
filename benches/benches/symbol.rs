use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use jlrs::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut julia = Builder::new().start_local().unwrap();

    julia.with_stack(|mut stack| {
        stack.scope(|frame| {
            c.bench_function("Symbol::new_4", |b| {
                b.iter(|| Symbol::new(&frame, black_box("1234")))
            });

            c.bench_function("Symbol::new_8", |b| {
                b.iter(|| Symbol::new(&frame, black_box("12345678")))
            });

            c.bench_function("Symbol::new_12", |b| {
                b.iter(|| Symbol::new(&frame, black_box("123456789012")))
            });

            c.bench_function("Symbol::new_16", |b| {
                b.iter(|| Symbol::new(&frame, black_box("1234567890123456")))
            });

            c.bench_function("Symbol::new_32", |b| {
                b.iter(|| Symbol::new(&frame, black_box("12345678901234561234567890123456")))
            });
        });
    })
}

criterion_group! {
    name = symbol;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(symbol);
