//! Runtime handles

#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
use parking_lot::{Condvar, Mutex};

#[cfg(feature = "async-rt")]
pub mod async_handle;
#[cfg(feature = "ccall")]
pub mod ccall;
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
pub mod delegated_handle;
#[cfg(feature = "local-rt")]
pub mod local_handle;
#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
pub mod mt_handle;
pub mod weak_handle;
pub mod with_stack;

/// Implemented by active handles. Only active handles allow calling into Julia.
pub trait IsActive: Sized {}

#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
pub(crate) fn notify(pair: &(Mutex<bool>, Condvar)) {
    let (ref lock, ref cvar) = &pair;
    let mut complete = lock.lock();
    *complete = true;
    cvar.notify_one();
}

#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
pub(crate) fn wait(pair: &(Mutex<bool>, Condvar)) {
    let (ref lock, ref cvar) = &pair;
    let mut complete = lock.lock();
    if !*complete {
        cvar.wait(&mut complete);
    }
}
