#[cfg(feature = "local-rt")]
use std::cell::RefCell;

#[cfg(feature = "local-rt")]
use jlrs::{prelude::*, runtime::handle::local_handle::LocalHandle};

#[cfg(feature = "jlrs-derive")]
pub mod derive_impls;

#[cfg(all(feature = "jlrs-derive", feature = "local-rt"))]
#[allow(dead_code)]
pub static JLRS_DERIVE_TESTS_JL: &'static str = include_str!("JlrsDeriveTests.jl");

thread_local! {
    #[cfg(all(feature = "jlrs-derive", feature = "local-rt"))]
    pub static JULIA_DERIVE: RefCell<LocalHandle> = {
        let r = RefCell::new(Builder::new().start_local().unwrap() );
        r.borrow().local_scope::<_, 1>(|mut frame| unsafe {
            Value::eval_string(&mut frame, JLRS_DERIVE_TESTS_JL).expect("failed to evaluate contents of JlrsTests.jl");
        });
        r
    }
}
