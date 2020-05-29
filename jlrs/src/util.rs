//! This module exists for testing purposes, the thread-local instance ensures Julia is only 
//! initialized once.

use crate::prelude::*;
use std::cell::RefCell;

thread_local! {
    #[doc(hidden)]
    pub static JULIA: RefCell<Julia> = {
        let r = RefCell::new(unsafe { Julia::init(32).unwrap() });
        r.borrow_mut().include("../jlrs.jl").unwrap();
        r
    };
}
