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
//! In particular, when you use jlrs all interactions with Julia happen inside a scope. A new
//! scope can be created with the methods [`Julia::scope`] and [`Julia::scope_with_slots`]. These
//! methods take a closure which is called inside this scope. This closure is provided with two
//! arguments, a [`Global`] and a mutable reference to a [`GcFrame`]. The first of these is an
//! access token that can be used to access Julia modules and their contents, the second is the
//! frame that is used to store roots. The frame is popped from the stack when leaving the scope,
//! so any value rooted in that frame can be used until the closure returns.
//!
//! Whenever a new value is created, it's usually rooted automatically by jlrs. Methods that
//! create new values either require an argument that implements the [`Scope`] trait, or a mutable
//! reference to something that implements the [`Frame`] trait. All mutable references to an
//! implementation of [`Frame`] implement [`Scope`].
//!
//! Scopes can be nested, which can be used to ensure temporary values you no longer need can be
//! freed by the garbage collector. The more roots there are, the more time the garbage collector
//! will need to find all reachable values. The simplest way to create a nested scope is by
//! calling [`ScopeExt::scope`], the [`ScopeExt`] trait is implemented for mutable references to
//! [`Frame`]s. The main limitation of this method is that it can't return a Julia value that is
//! rooted in the parent scope's frame. In order to do so, [`Scope::value_scope`] or
//! [`Scope::result_scope`] must be used. The closure provided to these methods takes an
//! [`Output`] and a mutable reference to a [`GcFrame`]. The frame can be used to root values
//! for the duration of the inner scope, the output can be converted to an [`OutputScope`] by
//! calling [`Output::into_scope`]. After this method is called, the frame can no longer be used.
//! Because [`OutputScope`] also implements [`Scope`], it can be used in combination with many
//! methods that create new values. In this case the result of these methods can be returned from
//! the closure, and is rooted in the parent scope's frame.
//!
//! [`Julia::scope`]: crate::Julia::scope
//! [`Julia::scope_with_slots`]: crate::Julia::scope_with_slots
//! [`Global`]: global::Global
//! [`GcFrame`]: frame::GcFrame
//! [`Scope`]: traits::scope::Scope
//! [`Frame`]: traits::frame::Frame
//! [`ScopeExt::scope`]: traits::scope::Scope::scope
//! [`ScopeExt`]: traits::scope::ScopeExt
//! [`Scope::value_scope`]: traits::scope::Scope::value_scope
//! [`Scope::result_scope`]: traits::scope::Scope::result_scope
//! [`Output`]: output::Output
//! [`OutputScope`]: output::OutputScope
//! [`Output::into_scope`]: output::Output::into_scope
pub mod frame;
pub mod global;
pub mod mode;
pub mod output;
pub(crate) mod stack;
pub mod traits;
