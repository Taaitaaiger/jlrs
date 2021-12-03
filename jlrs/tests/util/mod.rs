// This module exists for testing purposes, the thread-local instance ensures Julia is only
// initialized once. Note that tests that involve calling Julia functions must always be
// executed with `cargo test -- --test-threads=1`.

#[cfg(feature = "sync-rt")]
use jlrs::{julia::Julia, wrappers::ptr::value::Value};
#[cfg(feature = "sync-rt")]
use std::cell::RefCell;

#[cfg(feature = "sync-rt")]
static JLRS_TESTS_JL: &'static str = include_str!("JlrsTests.jl");
#[cfg(feature = "sync-rt")]
thread_local! {
#[doc(hidden)]
    pub static JULIA: RefCell<Julia> = {
        let r = RefCell::new(unsafe { Julia::init().unwrap() });
        r.borrow_mut().scope_with_slots(1, |_, frame| unsafe {
            Value::eval_string(frame, JLRS_TESTS_JL)?.expect("failed to evaluate contents of JlrsTests.jl");
            Ok(())
        }).unwrap();
        r
    };
}
