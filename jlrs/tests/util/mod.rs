use std::cell::RefCell;

#[cfg(feature = "sync-rt")]
use jlrs::{
    memory::stack_frame::StackFrame,
    runtime::{builder::RuntimeBuilder, sync_rt::PendingJulia},
    wrappers::ptr::value::Value,
};

#[cfg(feature = "sync-rt")]
#[allow(dead_code)]
static JLRS_TESTS_JL: &'static str = include_str!("JlrsTests.jl");

#[cfg(all(feature = "sync-rt", not(feature = "lts")))]
#[allow(dead_code)]
static JLRS_STABLE_TESTS_JL: &'static str = include_str!("JlrsStableTests.jl");

#[cfg(all(feature = "jlrs-derive", feature = "sync-rt"))]
#[allow(dead_code)]
pub static JLRS_DERIVE_TESTS_JL: &'static str = include_str!("JlrsDeriveTests.jl");

#[cfg(all(feature = "sync-rt", not(feature = "lts")))]
#[allow(dead_code)]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagStable.jl");

#[cfg(all(feature = "sync-rt", feature = "lts"))]
#[allow(dead_code)]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagLTS.jl");

#[cfg(all(feature = "async-rt", not(all(target_os = "windows", feature = "lts"))))]
#[allow(dead_code)]
pub static ASYNC_TESTS_JL: &'static str = include_str!("AsyncTests.jl");

#[cfg(all(feature = "async-rt", not(all(target_os = "windows", feature = "lts"))))]
pub mod async_tasks;

#[cfg(feature = "jlrs-derive")]
pub mod derive_impls;

thread_local! {
    #[cfg(feature = "sync-rt")]
    #[doc(hidden)]
    pub static JULIA: RefCell<PendingJulia> = {
        let mut frame = StackFrame::new();
        let r = RefCell::new(unsafe {RuntimeBuilder::new().start().unwrap() });
        r.borrow_mut().instance(&mut frame).scope(|mut frame| unsafe {
            Value::eval_string(&mut frame, JLRS_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
            #[cfg(not(feature = "lts"))]
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
