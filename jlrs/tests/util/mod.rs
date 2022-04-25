#[cfg(feature = "sync-rt")]
use jlrs::{julia::Julia, wrappers::ptr::value::Value};
#[cfg(feature = "sync-rt")]
use std::cell::RefCell;

#[cfg(feature = "sync-rt")]
#[allow(dead_code)]
static JLRS_TESTS_JL: &'static str = include_str!("JlrsTests.jl");

#[cfg(all(feature = "sync-rt", not(feature = "lts")))]
#[allow(dead_code)]
static JLRS_STABLE_TESTS_JL: &'static str = include_str!("JlrsStableTests.jl");

// This module is expensive, don't include it by default.
#[cfg(all(feature = "sync-rt", not(feature = "lts")))]
#[allow(dead_code)]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagStable.jl");
#[cfg(all(feature = "sync-rt", feature = "lts"))]
#[allow(dead_code)]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagLTS.jl");

thread_local! {
    #[cfg(feature = "sync-rt")]
    #[doc(hidden)]
    pub static JULIA: RefCell<Julia> = {
        let r = RefCell::new(unsafe { Julia::init().unwrap() });
        r.borrow_mut().scope_with_capacity(1, |_, frame| unsafe {
            Value::eval_string(&mut *frame, JLRS_TESTS_JL)?.expect("failed to evaluate contents of JlrsTests.jl");
            #[cfg(all(feature = "sync-rt", not(feature = "lts")))]
            Value::eval_string(&mut *frame, JLRS_STABLE_TESTS_JL)?.expect("failed to evaluate contents of JlrsTests.jl");
            Ok(())
        }).unwrap();
        r
    };
}
