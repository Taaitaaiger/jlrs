//! Runtime handles

use parking_lot::{Condvar, Mutex};

#[cfg(feature = "async-rt")]
pub mod async_handle;
#[cfg(feature = "ccall")]
pub mod ccall;
pub mod delegated_handle;
#[cfg(feature = "local-rt")]
pub mod local_handle;
#[cfg(feature = "multi-rt")]
pub mod mt_handle;
pub mod weak_handle;
pub mod with_stack;

/// Implemented by active handles. Only active handles allow calling into Julia.
pub trait IsActive: Sized {}

pub(crate) fn notify(pair: &(Mutex<bool>, Condvar)) {
    let (ref lock, ref cvar) = &pair;
    let mut complete = lock.lock();
    *complete = true;
    cvar.notify_one();
}

pub(crate) fn wait(pair: &(Mutex<bool>, Condvar)) {
    let (ref lock, ref cvar) = &pair;
    let mut complete = lock.lock();
    if !*complete {
        cvar.wait(&mut complete);
    }
}
