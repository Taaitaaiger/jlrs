#[cfg(feature = "sync-rt")]
use std::cell::RefCell;

#[cfg(feature = "sync-rt")]
use jlrs::{
    data::managed::value::Value,
    memory::stack_frame::StackFrame,
    runtime::{builder::RuntimeBuilder, sync_rt::PendingJulia},
};

#[cfg(feature = "jlrs-derive")]
pub mod derive_impls;

#[cfg(all(feature = "jlrs-derive", feature = "sync-rt"))]
#[allow(dead_code)]
pub static JLRS_DERIVE_TESTS_JL: &'static str = include_str!("JlrsDeriveTests.jl");

thread_local! {
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
