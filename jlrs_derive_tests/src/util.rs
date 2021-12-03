#[cfg(feature = "sync-rt")]
use jlrs::prelude::*;
#[cfg(feature = "sync-rt")]
use std::cell::RefCell;

#[cfg(feature = "sync-rt")]
thread_local! {
    pub static JULIA: RefCell<Julia> = {
        let r = RefCell::new(unsafe { Julia::init().unwrap() });
        r.borrow_mut().include("JlrsDeriveTests.jl").unwrap();
        r
    };
}
