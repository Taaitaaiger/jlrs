use std::cell::RefCell;

#[cfg(feature = "sync-rt")]
use jlrs::{
    data::managed::value::Value,
    memory::stack_frame::StackFrame,
    runtime::{builder::RuntimeBuilder, sync_rt::PendingJulia},
};

#[cfg(feature = "sync-rt")]
#[allow(dead_code)]
static JLRS_TESTS_JL: &'static str = include_str!("JlrsTests.jl");

#[cfg(all(feature = "sync-rt", not(feature = "julia-1-6")))]
#[allow(dead_code)]
static JLRS_STABLE_TESTS_JL: &'static str = include_str!("JlrsStableTests.jl");

#[cfg(all(feature = "jlrs-derive", feature = "sync-rt"))]
#[allow(dead_code)]
pub static JLRS_DERIVE_TESTS_JL: &'static str = include_str!("JlrsNewDeriveTests.jl");

#[cfg(all(feature = "sync-rt", not(feature = "julia-1-6")))]
#[allow(dead_code)]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagStable.jl");

#[cfg(all(feature = "sync-rt", feature = "julia-1-6"))]
#[allow(dead_code)]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagLTS.jl");

#[cfg(all(
    feature = "async-rt",
    not(all(target_os = "windows", feature = "julia-1-6"))
))]
#[allow(dead_code)]
pub static ASYNC_TESTS_JL: &'static str = include_str!("AsyncTests.jl");

#[cfg(all(
    feature = "async-rt",
    not(all(target_os = "windows", feature = "julia-1-6"))
))]
pub mod async_tasks;

#[cfg(feature = "jlrs-derive")]
pub mod new_derive_impls;

thread_local! {
    #[cfg(feature = "sync-rt")]
    #[doc(hidden)]
    pub static JULIA: RefCell<PendingJulia> = {
        let mut frame = StackFrame::new();
        let r = RefCell::new(unsafe {RuntimeBuilder::new().start().unwrap() });
        r.borrow_mut().instance(&mut frame).scope(|mut frame| unsafe {
            Value::eval_string(&mut frame, JLRS_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
            #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7")))]
            Value::eval_string(&mut frame, JLRS_STABLE_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
            Ok(())
        }).unwrap();
        r
    };

    #[cfg(all(feature = "jlrs-derive", feature = "sync-rt"))]
    #[doc(hidden)]
    pub static JULIA_DERIVE: RefCell<PendingJulia> = {
        let mut frame = StackFrame::new();
        let r = RefCell::new(unsafe {RuntimeBuilder::new().start().unwrap() });
        r.borrow_mut().instance(&mut frame).scope(|mut frame| unsafe {
            Value::eval_string(&mut frame, JLRS_DERIVE_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
            Ok(())
        }).unwrap();
        r
    };
}
