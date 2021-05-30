// This module exists for testing purposes, the thread-local instance ensures Julia is only
// initialized once. Note that tests that involve calling Julia functions must always be
// executed with `cargo test -- --test-threads=1`.

use crate::{wrappers::ptr::value::Value, Julia};
use std::cell::RefCell;

pub static JLRS_TESTS_JL: &'static str = include_str!("../tests/julia/JlrsTests.jl");

thread_local! {
    #[doc(hidden)]
    pub static JULIA: RefCell<Julia> = {
        let r = RefCell::new(unsafe { Julia::init().unwrap() });
        r.borrow_mut().scope_with_slots(1, |_, frame| {
            Value::eval_string(frame, JLRS_TESTS_JL)?.expect("failed to evaluate contents of JlrsTests.jl");
            Ok(())
        }).unwrap();
        r
    };
}
