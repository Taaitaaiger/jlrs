#[cfg(feature = "local-rt")]
use std::cell::RefCell;

#[cfg(feature = "local-rt")]
use jlrs::{
    data::managed::value::Value,
    error::JlrsResult,
    memory::{scope::Scope, stack_frame::StackFrame},
    runtime::{builder::Builder, sync_rt::PendingJulia},
};

#[cfg(feature = "local-rt")]
#[allow(dead_code)]
static JLRS_TESTS_JL: &'static str = include_str!("JlrsTests.jl");

#[cfg(all(feature = "local-rt", not(feature = "julia-1-6")))]
#[allow(dead_code)]
static JLRS_STABLE_TESTS_JL: &'static str = include_str!("JlrsStableTests.jl");

#[cfg(all(feature = "local-rt", not(feature = "julia-1-6")))]
#[allow(dead_code)]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagStable.jl");

#[cfg(all(feature = "local-rt", feature = "julia-1-6"))]
#[allow(dead_code)]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagLTS.jl");

thread_local! {
    #[cfg(feature = "local-rt")]
    #[doc(hidden)]
    #[allow(deprecated)]
    pub static JULIA: RefCell<PendingJulia> = {
        let mut frame = StackFrame::new();
        let r = RefCell::new(unsafe {Builder::new().start().unwrap() });
        r.borrow_mut().instance(&mut frame).returning::<JlrsResult<_>>().scope(|mut frame| unsafe {
            Value::eval_string(&mut frame, JLRS_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
            #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7")))]
            Value::eval_string(&mut frame, JLRS_STABLE_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
            Ok(())
        }).unwrap();
        r
    };
}
