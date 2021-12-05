use crate::julia::Julia;
use std::cell::RefCell;

thread_local! {
    #[doc(hidden)]
    pub static JULIA: RefCell<Julia> = RefCell::new(unsafe { Julia::init().unwrap() });
}
