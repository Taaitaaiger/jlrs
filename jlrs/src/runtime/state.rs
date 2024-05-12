use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(any(feature = "async-rt", feature = "multi-rt", feature = "local-rt"))]
use jl_sys::jl_is_initialized;

pub(crate) const GC_UNSAFE: i8 = 0;

/// State fof the Julia runtime
#[repr(u8)]
#[derive(PartialEq, Debug)]
pub enum State {
    /// Julia is inactive
    Uninit,
    /// Julia is active and has been embedded in a Rust application
    Init,
    /// Julia is active, has been embedded in a Rust application, and is about to exit
    PendingExit,
    /// Julia has been embedded in a Rust application, and has exited
    Exit,
    /// Julia is active, and the application has been started from Julia
    StartedFromJulia,
}

static JULIA_STATE: AtomicU8 = AtomicU8::new(State::Uninit as u8);

/// Sets the state to [`State::StartedFromJulia`].
///
/// Returns `false` if Julia has been embedded in a Rust application, in this case the state is
/// left unchanged.
///
/// Safety: must only be called from dynamic libraries during the call to `__init__`.
pub unsafe fn set_started_from_julia() -> bool {
    JULIA_STATE
        .compare_exchange(
            State::Uninit as u8,
            State::StartedFromJulia as u8,
            Ordering::Relaxed,
            Ordering::Relaxed,
        )
        .is_ok()
}

/// Returns the current state
pub fn current_state() -> State {
    unsafe { std::mem::transmute(JULIA_STATE.load(Ordering::Relaxed)) }
}

/// Returns `true` if `state` is the current state.
pub fn current_state_is(state: State) -> bool {
    current_state() == state
}

/// Returns `true` if the current state is [`State::Init`]
pub fn is_init() -> bool {
    current_state_is(State::Init)
}

#[cfg(any(feature = "async-rt", feature = "multi-rt", feature = "local-rt"))]
pub(super) fn can_init() -> bool {
    unsafe {
        if jl_is_initialized() != 0 {
            return false;
        }
    }

    try_set_init()
}

#[cfg(any(feature = "async-rt", feature = "multi-rt", feature = "local-rt"))]
pub(super) unsafe fn set_exit() {
    JULIA_STATE.store(State::Exit as _, Ordering::Relaxed);
}

#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
pub(super) unsafe fn set_pending_exit() {
    JULIA_STATE.store(State::PendingExit as _, Ordering::Relaxed);
}

#[cfg(any(feature = "async-rt", feature = "multi-rt", feature = "local-rt"))]
fn try_set_init() -> bool {
    JULIA_STATE
        .compare_exchange(
            State::Uninit as _,
            State::Init as _,
            Ordering::Relaxed,
            Ordering::Relaxed,
        )
        .is_ok()
}
