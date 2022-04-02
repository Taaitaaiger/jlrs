// This module exists for testing purposes, the thread-local instance ensures Julia is only
// initialized once. Note that tests that involve calling Julia functions must always be
// executed with `cargo test -- --test-threads=1`.

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
