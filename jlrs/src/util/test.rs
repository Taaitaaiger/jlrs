use std::cell::RefCell;

use crate::runtime::{builder::Builder, sync_rt::PendingJulia};

thread_local! {
    #[doc(hidden)]
    #[allow(deprecated)]
    pub static JULIA: RefCell<PendingJulia<>> = {
        RefCell::new(unsafe {Builder::new().start().unwrap() })
    }
}
