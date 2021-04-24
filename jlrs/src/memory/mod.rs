//! Structs and traits to protect data from being garbage collected.
//!
//! Julia is a garbage-collected programming language, whenever Julia is called through its C API
//! the user is responsible for ensuring the garbage collector is made aware when this value is in
//! use. In practice, this comes down to managing a linked list of gc frames which contain
//! pointers to values that are in use; these values have been rooted. While a value is rooted, it
//! and all other values it contains pointers to will not be freed by the garbage collector.
//! Several macros are available in C that create a new frame and push it to the stack, and a
//! macro to pop that frame from the stack. These macros cannot be used in Rust because they use
//! `alloca` to allocate the dynamically-sized raw frame on the stack. This module provides a
//! reimplementation of this system which is backed by a stack of pages, each page can store
//! multiple frames.
//!
//! In order to call Julia from Rust with jlrs, the methods [`Julia::scope`] or
//! [`Julia::scope_with_slots`] must be used. These methods take a closure that take a
//! [`Global`] and mutable reference to a [`GcFrame`]; before calling the closure the frame is
//! created and pushed the stack, and it's popped when it's dropped. This means that any value
//! that is rooted in that frame will be protected from garbage collection while inside the
//! closure. [`Global`] is an access token for global data in Julia, like modules and their
//! contents.
//!
//! Most functionality provided by the [`GcFrame`] is available through three traits; [`Frame`],
//! [`Scope`] and [`ScopeExt`]. [`Frame`] provides access to the number of roots and slots a frame
//! has, and its capacity, while the other two provide methods to create a nested scope. The
//! simplest kind of these methods is [`ScopeExt::scope`], like [`Julia::scope`] it creates a new
//! frame and pushes it to the stack, calls the given closure with a mutable reference to that new
//! frame, the frame is popped after the closure returns. The main limitation of this method is
//! that it can't be used to create a new Julia value and return it from the scope. This
//! functionality is provided by the [`Scope`] trait. The methods [`Scope::value_scope`] and
//! [`Scope::result_scope`] can be used to return a value or the result of a function call from a
//! closure and postpone rooting that result until the target frame can be used again.
//!
//! The closure that these two methods will call doesn't only provide a new frame, but also an
//! [`Output`]. The closures must return a value of a specific type. The frame can be used
//! to allocate temporary values. When all temporary values have been created, the [`Output`] must
//! be converted to an [`OutputScope`] by calling [`Output::into_scope`]. The frame can now no
//! longer be used because this method will borrow it for the rest of the closure, but because it
//! implements [`Scope`] it can create a nested scope which propagates the output to this new
//! scope, and be used to create a new value or call a Julia function. This result can be
//! returned from the closure. It is rooted if the parent scope is a frame, and left unrooted if
//! it's an [`OutputScope`].
//!
//! There are two other kinds of frame, [`NullFrame`] and [`AsyncGcFrame`]. The first of these
//! can be used when calling a Rust function from Julia with `ccall`. It doesn't support creating
//! nested scopes and can't root any values, but it can be used to access array data. Accessing
//! this data requires a frame to prevent mutable aliasing. The other is available when the
//! `async` feature flag is enabled. [`AsyncGcFrame`] offers the same methods as [`GcFrame`],
//! implements the same traits, but also provides async variations of [`Scope::value_scope`],
//! [`Scope::result_scope`], and [`ScopeExt::scope`]. This frame type can be used by implementing
//! the [`JuliaTask`] trait.
//!
//! [`Julia::scope`]: crate::Julia::scope
//! [`Julia::scope_with_slots`]: crate::Julia::scope_with_slots
//! [`Global`]: crate::memory::global::Global
//! [`GcFrame`]: crate::memory::frame::GcFrame
//! [`Frame`]: crate::memory::traits::frame::Frame
//! [`Scope`]: crate::memory::traits::scope::Scope
//! [`ScopeExt`]: crate::memory::traits::scope::ScopeExt
//! [`ScopeExt::scope`]: crate::memory::traits::scope::ScopeExt::scope
//! [`Scope::value_scope`]: crate::memory::traits::scope::Scope::value_scope
//! [`Scope::result_scope`]: crate::memory::traits::scope::Scope::result_scope
//! [`Output`]: crate::memory::output::Output
//! [`OutputScope`]: crate::memory::output::OutputScope
//! [`Output::into_scope`]: crate::memory::output::Output::into_scope
//! [`NullFrame`]: crate::memory::frame::NullFrame
//! [`AsyncGcFrame`]: crate::memory::frame::AsyncGcFrame
//! [`JuliaTask`]: crate::multitask::julia_task::JuliaTask

pub mod frame;
pub mod global;
pub mod mode;
pub mod output;
pub(crate) mod stack;
pub mod traits;
