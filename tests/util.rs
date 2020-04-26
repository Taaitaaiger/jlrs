use jlrs::prelude::*;
use std::cell::RefCell;

thread_local! {
    pub static JULIA: RefCell<Julia> = {
        let r = RefCell::new(unsafe { Julia::init(32).unwrap() });
        r.borrow_mut().include("jlrs.jl").unwrap();
        r
    };
}
