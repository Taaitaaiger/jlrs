#[cfg(feature = "local-rt")]
use std::cell::RefCell;

#[cfg(feature = "local-rt")]
use jlrs::{prelude::*, runtime::handle::local_handle::LocalHandle};

#[cfg(feature = "local-rt")]
#[allow(dead_code)]
static JLRS_TESTS_JL: &'static str = include_str!("JlrsTests.jl");

#[cfg(all(feature = "local-rt"))]
#[allow(dead_code)]
static JLRS_STABLE_TESTS_JL: &'static str = include_str!("JlrsStableTests.jl");

#[cfg(all(feature = "local-rt"))]
#[allow(dead_code)]
pub static MIXED_BAG_JL: &'static str = include_str!("MixedBagStable.jl");

thread_local! {
    pub static JULIA: RefCell<LocalHandle> = {
        let r = RefCell::new(Builder::new().start_local().unwrap() );
        r.borrow().local_scope::<_, 2>(|mut frame| unsafe {
            Value::eval_string(&mut frame, JLRS_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
            Value::eval_string(&mut frame, JLRS_STABLE_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
        });
        r
    }
}
