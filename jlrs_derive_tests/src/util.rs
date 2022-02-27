use jlrs::prelude::*;
use std::cell::RefCell;

thread_local! {
    pub static JULIA: RefCell<Julia> = {
        let r = RefCell::new(unsafe { Julia::init().unwrap() });
        unsafe {
            r.borrow_mut().include("JlrsDeriveTests.jl").unwrap();
        }
        r
    };
}
