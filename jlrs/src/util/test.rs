use std::cell::RefCell;

use crate::runtime::{builder::RuntimeBuilder, sync_rt::PendingJulia};

thread_local! {
    #[doc(hidden)]
    pub static JULIA: RefCell<PendingJulia<>> = {
        RefCell::new(unsafe {RuntimeBuilder::new().start().unwrap() })
    }
}
