#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(feature = "sync-rt")]
use jlrs::{
    runtime::{builder::RuntimeBuilder, sync_rt::Julia},
    wrappers::ptr::value::Value,
};
use std::cell::RefCell;

static JLRS_TESTS_JL: &'static str = include_str!("JlrsTests.jl");

#[cfg(all(
    feature = "sync-rt",
    any(not(feature = "lts"), feature = "all-features-override")
))]
static JLRS_STABLE_TESTS_JL: &'static str = include_str!("JlrsStableTests.jl");

#[cfg(all(
    feature = "sync-rt",
    any(not(feature = "lts"), feature = "all-features-override")
))]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagStable.jl");
#[cfg(all(
    feature = "sync-rt",
    all(feature = "lts", not(feature = "all-features-override"))
))]
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
    pub static JULIA: RefCell<Julia> = {
        let r = RefCell::new(unsafe {RuntimeBuilder::new().start().unwrap() });
        r.borrow_mut().scope_with_capacity(1, |_, mut frame| unsafe {
            Value::eval_string(&mut frame, JLRS_TESTS_JL)?.expect("failed to evaluate contents of JlrsTests.jl");
            #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
            Value::eval_string(&mut frame, JLRS_STABLE_TESTS_JL)?.expect("failed to evaluate contents of JlrsTests.jl");
            Ok(())
        }).unwrap();
        r
    };

    #[cfg(all(feature = "jlrs-derive", feature = "sync-rt"))]
    #[doc(hidden)]
    pub static JULIA_DERIVE: RefCell<Julia> = {
        let r = RefCell::new(unsafe {RuntimeBuilder::new().start().unwrap() });
        r.borrow_mut().scope_with_capacity(1, |_, mut frame| unsafe {
            Value::eval_string(&mut frame, JLRS_DERIVE_TESTS_JL)?.expect("failed to evaluate contents of JlrsTests.jl");
            Ok(())
        }).unwrap();
        r
    };
}
