use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use jlrs::{memory::target::RootingTarget, prelude::*};

#[inline(never)]
fn typecheck_type<'target, Tgt: RootingTarget<'target>>(
    target: Tgt,
    c: &mut Criterion,
) -> JlrsResult<()> {
    let value = Tgt::into_concrete_type(Value::new(target, 32i8));
    c.bench_function("typecheck_type", |b| {
        b.iter(|| black_box(black_box(value.datatype()).is::<i8>()))
    });
    Ok(())
}

#[inline(never)]
fn typecheck_value<'target, Tgt: RootingTarget<'target>>(
    target: Tgt,
    c: &mut Criterion,
) -> JlrsResult<()> {
    let value = Tgt::into_concrete_type(Value::new(target, 32i8));

    c.bench_function("typecheck_value", |b| {
        b.iter(|| black_box(value.is::<i8>()))
    });
    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    let h = Builder::new().start_local();
    println!("Started");
    h.unwrap().local_scope::<_, 2>(|mut frame| {
        typecheck_value(&mut frame, c).unwrap();
        typecheck_type(&mut frame, c).unwrap();
    });
}

criterion_group! {
    name = type_construction;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(type_construction);
