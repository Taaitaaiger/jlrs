use crate::runtime::{builder::RuntimeBuilder, sync_rt::PendingJulia};
use std::cell::RefCell;

thread_local! {
    #[doc(hidden)]
    pub static JULIA: RefCell<PendingJulia<>> = {
        RefCell::new(unsafe {RuntimeBuilder::new().start().unwrap() })
    }
}
