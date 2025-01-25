use std::{cell::RefCell, future::Future};

use criterion::{
    async_executor::AsyncExecutor, black_box, /* criterion_group, criterion_main, */ Criterion,
};
use jlrs::{prelude::*, runtime::handle::mt_handle::MtHandle};
#[cfg(not(target_os = "windows"))]
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::Options,
};
use tokio::{runtime::Runtime, task::LocalSet};

thread_local! {
    static LOCAL_SET: RefCell<LocalSet> = RefCell::new(LocalSet::new());
    static RUNTIME: RefCell<Option<Runtime>> = RefCell::new(None);
}

pub struct TokioExecutor;
impl AsyncExecutor for TokioExecutor {
    fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        RUNTIME.with(|rt_refcell| {
            let mut rt_ref = rt_refcell.borrow_mut();
            if rt_ref.is_none() {
                *rt_ref = tokio::runtime::Builder::new_current_thread().build().ok();
            }

            let rt = rt_ref.as_ref().unwrap();
            LOCAL_SET.with(|ls| ls.borrow().block_on(rt, future))
        })
    }
}

struct MyTask;

impl AsyncTask for MyTask {
    type Output = ();

    async fn run<'base>(&mut self, _frame: AsyncGcFrame<'base>) -> Self::Output {}
}

#[inline(never)]
fn blocking_task(handle: &MtHandle, c: &mut Criterion) {
    let pool = black_box(
        handle
            .pool_builder(Tokio::<2>::new(false))
            .channel_capacity(1)
            .spawn(),
    );

    c.bench_function("blocking_task_pool", |b| {
        b.to_async(TokioExecutor).iter(|| async {
            pool.blocking_task(|_| 1usize)
                .dispatch()
                .await
                .unwrap()
                .await
                .unwrap()
        })
    });
}

#[inline(never)]
fn async_task(handle: &MtHandle, c: &mut Criterion) {
    let pool = black_box(handle.pool_builder(Tokio::<2>::new(false)).spawn());

    c.bench_function("async_task_pool", |b| {
        b.to_async(TokioExecutor)
            .iter(|| async { pool.task(MyTask).dispatch().await.unwrap().await.unwrap() })
    });
}

#[inline(never)]
fn use_local(handle: &mut MtHandle, c: &mut Criterion) {
    c.bench_function("use_local", |b| {
        b.iter(|| {
            black_box(handle.with(|active| {
                active.local_scope::<_, 1>(|frame| {
                    black_box(frame);
                })
            }));
        })
    });
}

#[cfg(not(target_os = "windows"))]
fn opts() -> Option<Options<'static>> {
    let mut opts = Options::default();
    opts.image_width = Some(1920);
    opts.min_width = 0.01;
    Some(opts)
}

// #[cfg(not(target_os = "windows"))]
// criterion_group! {
//     name = mt_rt_pool;
//     config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
//     targets = criterion_benchmark
// }

// #[cfg(target_os = "windows")]
// criterion_group! {
//     name = mt_rt_pool;
//     config = Criterion::default();
//     targets = criterion_benchmark
// }

// criterion_main!(mt_rt_pool);
fn main() {
    Builder::new()
        .start_mt(|mut handle| {
            #[cfg(not(target_os = "windows"))]
            let mut c = Criterion::default()
                .with_profiler(PProfProfiler::new(1000, Output::Flamegraph(opts())));
            #[cfg(target_os = "windows")]
            let mut c = Criterion::default();

            blocking_task(&handle, &mut c);
            async_task(&handle, &mut c);
            use_local(&mut handle, &mut c);

            Criterion::default().configure_from_args().final_summary();
        })
        .unwrap();
}
