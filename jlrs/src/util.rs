use crate::runtime::{builder::RuntimeBuilder, sync_rt::Julia};
use std::cell::RefCell;

thread_local! {
    #[doc(hidden)]
    pub static JULIA: RefCell<Julia> = RefCell::new(unsafe {RuntimeBuilder::new().start().unwrap() });
}
