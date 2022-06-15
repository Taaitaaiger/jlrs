//! Julia memory management.
//!
//! This module contains all structs and traits that are used to deal with memory management and
//! enforcing a degree of compile-time memory safety by applying reasonable lifetime bounds to
//! Julia data.
//!
//! The Julia GC is unaware of any references to Julia data existing outside of Julia itself. To
//! make these references known to the GC, data that is used must be rooted. This ensures the GC
//! doesn't accidentally identify the data as unused and free it.
//!
//! A scope must be created before data can be rooted, the sync runtime lets you create a scope
//! directly with [`Julia::scope`]. This method takes a closure which takes two arguments, the
//! second is a mutable reference to a [`GcFrame`]. A `GcFrame` is used to store roots, each scope
//! has its own `GcFrame`, any Julia data rooted in this frame is guaranteed to be protected from
//! being freed by the GC until you leave the scope.
//!
//! There are two other frame types, [`AsyncGcFrame`] and [`NullFrame`], the first is used by the
//! async runtime and is used with several async functions, while the latter is only available
//! when calling Rust from Julia with its `ccall` interface. All these frame types implement the
//! [`Frame`] trait.
//!
//! It's important to avoid rooting data longer than necessary. As more data is rooted, more
//! memory remains in use, which causes the GC to run more often and require more time to run.
//! Scopes can be nested by calling [`Frame::scope`], they form a single nested hierarchy. Any
//! data rooted in the frame provided to this new scope is rooted until that new scope ends.
//! Lifetimes ensure no data that is rooted in the child scope can be returned to the parent
//! scope.
//!
//! To return rooted data from a child scope, it must be rooted in an ancestral scope. The method
//! [`Frame::output`] can be used to reserve an [`Output`] in a frame. Methods that return rooted
//! data generally take an implementation of [`PartialScope`] or [`Scope`]. Methods that take a
//! `PartialScope` only need to root a single value, both `Output` and mutable references to an
//! implementation of `Frame` implement this trait. In the first case the data is rooted in the
//! frame targeted by the output, in the second it's rooted in the current frame. Methods that
//! take a `Scope` need to root temporary data. While mutable references to an implementation of
//! `Frame` implement this trait, `Output` doesn't. An `Output` must first be upgraded to an
//! [`OutputScope`] by calling [`Output::into_scope`].
//!
//! Not all data needs to be rooted, roots form the starting point for the GC to trace the entire
//! graph of reachable data. As long as data is reachable from a root, it won't be freed. Julia
//! modules provide global scopes, their contents are rooted unless the module is reloaded or the
//! data is overwritten some other way (e.g. by mutating a global value). If you never use the
//! result of a Julia function call the result doesn't need to be rooted either. Most methods that
//! return rooted Julia data have a corresponding method that leaves the result unrooted. These
//! methods often only require a [`Global`], which ensures that this data can't be accessed before
//! Julia has been initialized and that reasonable lifetime bounds are applied.
//!
//! The final tool that is available to manage the rootedness of Julia data is the
//! [`ReusableSlot`], it can be created with [`Frame::reusable_slot`] and provides a slot in that
//! frame that can be overwritten.
//!
//! [`Global`]: global::Global
//! [`ReusableSlot`]: reusable_slot::ReusableSlot
//! [`GcFrame`]: frame::GcFrame
//! [`AsyncGcFrame`]: frame::AsyncGcFrame
//! [`NullFrame`]: frame::NullFrame
//! [`PartialScope`]: scope::PartialScope
//! [`Scope`]: scope::Scope
//! [`Frame`]: frame::Frame
//! [`Frame::scope`]: frame::Frame::scope
//! [`Frame::output`]: frame::Frame::output
//! [`Frame::reusable_slot`]: frame::Frame::reusable_slot
//! [`Output`]: output::Output
//! [`OutputScope`]: scope::OutputScope
//! [`Output::into_scope`]: output::Output::into_scope
//! [`Julia::scope`]: crate::runtime::sync_rt::Julia::scope
pub mod frame;
pub mod gc;
pub mod global;
pub mod mode;
pub mod output;
pub mod reusable_slot;
pub mod scope;
pub(crate) mod stack_page;

use cfg_if::cfg_if;
use jl_sys::jl_tls_states_t;

pub(crate) unsafe fn get_tls() -> *mut jl_tls_states_t {
    cfg_if! {
        if #[cfg(all(feature = "lts", not(feature = "all-features-override")))] {
            use jl_sys::jl_get_ptls_states;
            jl_get_ptls_states()
        } else {
            use jl_sys::jl_get_current_task;
            use std::ptr::NonNull;
            NonNull::new_unchecked(jl_get_current_task()).as_ref().ptls
        }
    }
}
