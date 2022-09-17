#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(feature = "sync-rt")]
use jlrs::{
    runtime::{builder::RuntimeBuilder, sync_rt::Julia,},
    memory::context::ContextFrame,
    wrappers::ptr::value::Value,
};
use std::{cell::RefCell, ffi::c_void, ptr::null_mut};

static JLRS_TESTS_JL: &'static str = include_str!("JlrsTests.jl");

#[cfg(all(feature = "sync-rt", not(feature = "lts")))]
static JLRS_STABLE_TESTS_JL: &'static str = include_str!("JlrsStableTests.jl");

#[cfg(all(feature = "sync-rt", not(feature = "lts")))]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagStable.jl");
#[cfg(all(feature = "sync-rt", feature = "lts"))]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagLTS.jl");

#[cfg(all(feature = "async-rt", not(all(target_os = "windows", feature = "lts"))))]
pub static ASYNC_TESTS_JL: &'static str = include_str!("AsyncTests.jl");

#[cfg(feature = "jlrs-derive")]
pub static JLRS_DERIVE_TESTS_JL: &'static str = include_str!("JlrsDeriveTests.jl");

#[cfg(all(feature = "async-rt", not(all(target_os = "windows", feature = "lts"))))]
pub mod async_tasks;

#[cfg(feature = "jlrs-derive")]
pub mod derive_impls;

thread_local! {
    #[cfg(feature = "sync-rt")]
    #[doc(hidden)]
    pub static JULIA: RefCell<Julia<'static>> = {
        let context_frame = jlrs::util::test::static_context_frame();
        let r = RefCell::new(unsafe {RuntimeBuilder::new().start(context_frame).unwrap() });
        r.borrow_mut().scope(|_, mut frame| unsafe {
            Value::eval_string(&mut frame, JLRS_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
            #[cfg(not(feature = "lts"))]
            Value::eval_string(&mut frame, JLRS_STABLE_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
            Ok(())
        }).unwrap();
        r
    };

    #[cfg(all(feature = "jlrs-derive", feature = "sync-rt"))]
    #[doc(hidden)]
    pub static JULIA_DERIVE: RefCell<Julia<'static>> = {
        let context_frame = jlrs::util::test::static_context_frame();
        let r = RefCell::new(unsafe {RuntimeBuilder::new().start(context_frame).unwrap() });
        r.borrow_mut().scope(|_, mut frame| unsafe {
            Value::eval_string(&mut frame, JLRS_DERIVE_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
            Ok(())
        }).unwrap();
        r
    };
}
