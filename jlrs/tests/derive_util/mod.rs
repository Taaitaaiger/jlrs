#[cfg(feature = "local-rt")]
use std::cell::RefCell;

#[cfg(feature = "local-rt")]
use jlrs::{
    data::managed::value::Value,
    memory::{scope::Scope, stack_frame::StackFrame},
    runtime::{builder::Builder, sync_rt::PendingJulia},
};

#[cfg(feature = "jlrs-derive")]
pub mod derive_impls;

#[cfg(all(feature = "jlrs-derive", feature = "local-rt"))]
#[allow(dead_code)]
pub static JLRS_DERIVE_TESTS_JL: &'static str = include_str!("JlrsDeriveTests.jl");

thread_local! {
    #[cfg(all(feature = "jlrs-derive", feature = "local-rt"))]
    #[doc(hidden)]
    #[allow(deprecated)]
    pub static JULIA_DERIVE: RefCell<PendingJulia> = {
        let mut frame = StackFrame::new();
        let r = RefCell::new(unsafe {Builder::new().start().unwrap() });
        r.borrow_mut().instance(&mut frame).scope(|mut frame| unsafe {
            Value::eval_string(&mut frame, JLRS_DERIVE_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
        });
        r
    };
}
