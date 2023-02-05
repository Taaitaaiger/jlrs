//! Julia memory management.
//!
//! This module contains all structs and traits that are used to deal with memory management and
//! enforcing a degree of compile-time memory safety by applying reasonable lifetime bounds to
//! Julia data. The functionality is split across three submodules, [`target`], [`stack_frame`],
//! and [`gc`]. The first provides targets, which are used by methods that return Julia data, they
//! ensure the returned data is of the correct type and has appropriate lifetimes assigned to it.
//! The second provides a raw GC frame. The last provides access to methods that control the GC
//! itself.

pub(crate) mod context;
pub mod gc;
pub mod stack_frame;
pub mod target;

#[julia_version(since = "1.8")]
use jl_sys::jl_ptls_t;
#[julia_version(until = "1.7")]
use jl_sys::jl_tls_states_t;
use jlrs_macros::julia_version;

#[julia_version(since = "1.8")]
pub type PTls = jl_ptls_t;

#[julia_version(until = "1.7")]
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
    let task = jl_get_current_task();
    NonNull::new(task).unwrap().as_ref().ptls
}
