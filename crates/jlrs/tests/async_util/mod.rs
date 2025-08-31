#[cfg(all(feature = "async-rt",))]
#[allow(dead_code)]
pub static ASYNC_TESTS_JL: &'static str = include_str!("AsyncTests.jl");

#[cfg(all(feature = "async-rt",))]
pub mod async_tasks;
