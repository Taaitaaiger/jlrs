//! Structs and traits to protect data from being garbage collected.
//!
//! Julia is a garbage-collected programming language, whenever Julia is called through its C API
//! the user is responsible for ensuring the garbage collector can reach all values that are in
//! use. The garbage collector uses a set of values called roots as a starting point when
//! determining what values can still be reached, any value that is reachable is not freed.
//! Whenever a newly allocated value is returned by the C API, it's not reachable from one of the
//! existing roots so it must be added to this set. Structurally, this set is a stack of GC
//! frames. A frame is essentially a dynamically-sized array of roots. The C API provides several
//! macros to create such a frame and push it to the stack, which can only be used once in a scope
//! and must be matched by a call to the macro that pops the frame from the stack before leaving
//! the scope.
//!
//! These macros can be neither directly translated to Rust, nor wrapped in another C function,
//! because these macros allocate the frame on the stack with `alloca`, which is not possible in
//! Rust. Instead, the structs and traits in this module provide a reimplementation of this
//! mechanism.
//!
//! In particular, when you use jlrs all interactions with Julia happen inside a scope. A base
//! scope can be created with the methods [`Julia::scope`] and [`Julia::scope_with_slots`]. These
//! methods take a closure which is called inside this scope. This closure is provided with its
//! two arguments, a [`Global`] and a mutable reference to a [`GcFrame`]. The first of these is an
//! access token that can be used to access Julia modules and their contents, the second is a new
//! frame that is used to store roots. The frame is popped from the stack when leaving the scope,
//! so any value rooted in that frame can be used until you leave the scope.
//!
//! Whenever a new value is created, it's usually rooted automatically by jlrs. Methods that
//! create new values either require an argument that implements the [`Scope`] trait, or a mutable
//! reference to something that implements the [`Frame`] trait. All mutable references to an
//! implementation of [`Frame`] implement [`Scope`].
//!
//! More informaton can be found in the [`frame`] and [`scope`] modules.
//!
//! [`Julia::scope`]: crate::Julia::scope
//! [`Julia::scope_with_slots`]: crate::Julia::scope_with_slots
//! [`Global`]: global::Global
//! [`GcFrame`]: frame::GcFrame
//! [`Scope`]: scope::Scope
//! [`Frame`]: frame::Frame
//! [`ScopeExt::scope`]: scope::Scope::scope
//! [`ScopeExt`]: scope::ScopeExt
//! [`Scope::value_scope`]: scope::Scope::value_scope
//! [`Scope::result_scope`]: scope::Scope::result_scope
//! [`Output`]: output::Output
//! [`OutputScope`]: output::OutputScope
//! [`Output::into_scope`]: output::Output::into_scope
pub mod frame;
pub mod gc;
pub mod global;
pub mod mode;
pub mod output;
pub mod reusable_slot;
pub(crate) mod root_pending;
pub mod scope;
pub(crate) mod stack_page;

#[cfg(feature = "lts")]
use jl_sys::jl_get_ptls_states;
use jl_sys::jl_tls_states_t;
#[cfg(not(feature = "lts"))]
use jl_sys::jlrs_current_task;
#[cfg(not(feature = "lts"))]
use std::ptr::NonNull;

#[cfg(feature = "lts")]
pub(crate) unsafe fn get_tls() -> *mut jl_tls_states_t {
    jl_get_ptls_states()
}

#[cfg(not(feature = "lts"))]
pub(crate) unsafe fn get_tls() -> *mut jl_tls_states_t {
    NonNull::new_unchecked(jlrs_current_task()).as_ref().ptls
}
