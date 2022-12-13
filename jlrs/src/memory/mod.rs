//! Julia memory management.
//!
//! This module contains all structs and traits that are used to deal with memory management and
//! enforcing a degree of compile-time memory safety by applying reasonable lifetime bounds to
//! Julia data.
//!
//! The functionality is split across three submodules, [`target`], [`stack_frame`], and [`gc`].
//! The first provides targets, which are used by methods that return Julia data, they ensure the
//! returned data is of the correct type and has appropriate lifetimes assigned to it. The second
//! provides a raw GC frame. The last provides access to methods that control the GC itself.
//!
//! Julia data can be considered as being owned by the Julia GC because the GC is responsible for
//! freeing this data after it has become inaccessible. In order to determine what data is still
//! accessible a set of roots is maintained. These roots are pointers to Julia data, during the
//! GC's marking phase these roots are used as a starting point to recursively mark all pointers
//! to other Julia data that can be reached as accessible. Afterwards, inaccessible data is freed
//! during the GC's sweep phase.
//!
//! When you're writing Julia code you don't have to worry about this because the Julia compiler
//! ensures that data remains reachable while it's in use. Low-level code that interfaces directly
//! with the C API, either through a `ccall`ed function or by embedding Julia, has to be more
//! careful because the Julia compiler is completely unaware of references to Julia data existing
//! outside of Julia code.

pub(crate) mod context;
pub mod gc;
pub mod stack_frame;
pub mod target;

#[julia_version(since = "1.9")]
use jl_sys::jl_ptls_t;
#[julia_version(until = "1.8")]
use jl_sys::jl_tls_states_t;
use jlrs_macros::julia_version;

#[julia_version(since = "1.9")]
pub type PTls = jl_ptls_t;

#[julia_version(until = "1.8")]
pub type PTls = *mut jl_tls_states_t;

#[julia_version(until = "1.6")]
pub(crate) unsafe fn get_tls() -> PTls {
    use jl_sys::jl_get_ptls_states;
    jl_get_ptls_states()
}

#[julia_version(since = "1.7")]
pub(crate) unsafe fn get_tls() -> PTls {
    use std::ptr::NonNull;

    use jl_sys::jl_get_current_task;
    NonNull::new_unchecked(jl_get_current_task()).as_ref().ptls
}
