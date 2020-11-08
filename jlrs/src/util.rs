// This module exists for testing purposes, the thread-local instance ensures Julia is only
// initialized once.

use crate::prelude::*;
use std::cell::RefCell;

thread_local! {
    #[doc(hidden)]
    pub static JULIA: RefCell<Julia> = {
        use std::env;
        let dir = env::var("JLRS_ROOT").expect("You must set the JLRS_ROOT environement variable to the root of the repository (the directory that contains jlrs, jlrs_derive, etc) in order to run the tests.");
        let r = RefCell::new(unsafe { Julia::init(32).unwrap() });
        r.borrow_mut().include(format!("{}/jlrs/tests/julia/JlrsTests.jl", dir)).unwrap();
        r
    };
}
