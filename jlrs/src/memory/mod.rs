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

use cfg_if::cfg_if;
#[cfg(feature = "nightly")]
use jl_sys::jl_ptls_t;
#[cfg(not(feature = "nightly"))]
use jl_sys::jl_tls_states_t;

#[cfg(feature = "nightly")]
pub type PTls = jl_ptls_t;
#[cfg(not(feature = "nightly"))]
pub type PTls = *mut jl_tls_states_t;

pub(crate) unsafe fn get_tls() -> PTls {
    cfg_if! {
        if #[cfg(feature = "lts")] {
            use jl_sys::jl_get_ptls_states;
            jl_get_ptls_states()
        } else {
            use jl_sys::jl_get_current_task;
            use std::ptr::NonNull;
            NonNull::new_unchecked(jl_get_current_task()).as_ref().ptls
        }
    }
}
