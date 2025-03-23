//! Julia memory management.
//!
//! As you might already know Julia has a garbage collector (GC). Whenever new data, like an
//! `Array` or a `String` is created, the GC is responsible for freeing that data when it has
//! become unreachable. While Julia is aware of references to data existing in Julia code, it is
//! unaware of references existing outside of Julia code.
//!
//! To make Julia aware of such foreign references we'll need to tell it they exist and that the
//! GC needs to leave that data alone. This is called rooting. While a reference is rooted, the GC
//! won't free its data. Any Julia data referenced by rooted data is also safe from being freed.
//!
//! Before data can be rooted a scope has to be created. Functions that call into Julia can
//! only be called from a scope. A new scope can be created by calling [`LocalScope::local_scope`]
//! or [`Scope::scope`]. These functions takes a closure which contains the code called inside
//! that scope. A local scope must be annotated with its number of slots, (dynamically-sized)
//! scopes require a dynamic stack.
//!
//! The closure takes a single argument, a frame, which lets you root data. Methods
//! in jlrs that return Julia data can be called with a mutable reference to a frame. This uses
//! one of the frame's slots. The returned data is guaranteed to be rooted until you leave the
//! scope that provided the frame:
//!
//! ```
//! # use jlrs::prelude::*;
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//! // One slot is used
//! julia.local_scope::<1>(|mut frame| {
//!     // This data is guaranteed to live at least until we leave this scope
//!     let i = Value::new(&mut frame, 1u64);
//! });
//! # }
//! ```
//!
//! If you tried to return `i` from the scope in the example above, the code would fail to
//! compile. A frae has a lifetime that outlives the closure but doesn't outlive the scope.
//! This lifetime is propagated to types like `Value` to prevent the data from being used after it
//! has become unrooted.
//!
//! A frame can also be used to create a nested scope:
//!
//! ```
//! # use jlrs::prelude::*;
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//! julia.local_scope::<1>(|mut frame| {
//!     let i = Value::new(&mut frame, 1u64);
//!
//!     // One slot is used
//!     frame.local_scope::<1>(|mut frame| {
//!         let j = Value::new(&mut frame, 2u64);
//!         // j can't be returned from this scope, but i can.
//!         i
//!     });
//! });
//! # }
//! ```
//!
//! As you can see in that example, `i` can be returned from the inner scope because `i` is
//! guaranteed to outlive it, while `j` can't because it doesn't. In many cases, though, we want
//! to create a scope and return data created in that scope. In that case, we'll need to
//! create an [`Output`] in the targeted scope:
//!
//! ```
//! # use jlrs::prelude::*;
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//!
//! // One slot is used to create an output
//! julia.local_scope::<1>(|mut frame| {
//!     let output = frame.output();
//!
//!     // This frame uses no slots, the output was reserved in advance
//!     frame.local_scope::<0>(|_| Value::new(output, 2u64));
//! });
//! # }
//! ```
//!
//! This works because an `Output` roots data in the frame that was used to create it, and
//! the lifetime of that frame is propageted to the result. We can also see that `Value::new`
//! can't just take a mutable reference to a frame, but any type that implements [`Target`].
//! These types are called targets.
//!
//! Each target has a lifetime that enforces the result can't outlive the scope that roots it. In
//! addition to rooting targets like `&mut LocalGcFrame` and `Output` that we've already seen,
//! there also exist non-rooting targets. Non-rooting targets exist because it isn't always
//! necessary to root data: we might be able to guarantee it's already rooted or reachable and
//! avoid creating another root, or never use the data at all. Using unrooted data is always
//! unsafe.
//!
//! Rooting targets can be used as a non-rooting target by using a reference to the target.
//! Rooting and non-rooting targets return different types when they're used: `Value::new`
//! returns a [`Value`] when a rooting target is used, but a [`WeakValue`] when a non-rooting one
//! is used instead. Every type that represents managed Julia data has a rooted and an unrooted
//! variant, which are named similarly to `Value` and `WeakValue`. There are two type aliases for
//! each managed type that can be used as the return type of functions that take a target
//! generically. For `Value` they are [`ValueData`] and [`ValueResult`]. `ValueData` is a `Value`
//! if a rooting target is used and a `WeakValue` otherwise. `ValueResult` is a `Result` that
//! contains `ValueData` in both its `Ok` and `Err` variants, if an `Err` is returned the
//! operation failed and an exception was caught.
//!
//! Functions that take a `Target` and return Julia data have signatures like this:
//!
//! ```
//! # use jlrs::prelude::*;
//! fn takes_target<'target, Tgt>(target: Tgt, /* other args */) -> ValueData<'target, 'static, Tgt>
//! where
//!     Tgt: Target<'target>,
//! {
//! # todo!()
//! }
//! ```
//!
//! The frames and scopes we've seen so far are local, they are allocated on the stack and have
//! a maximum number of roots they can store. One major drawback is that we need to count the
//! number of slots that will be used. It's possible to create a dynamically-sized stack, and use
//!  dynamically-sized scopes and frames by calling `WithStack::with_stack` and `Scope::scope`.
//! Using local scopes is recommended over using dynamically-sized scopes.
//!
//! In general, it's highly advised that you only write function that take a target generically.
//! Each use of `&mut frame` in a closure will take one slot, and all you need to do is count how
//!  often `&mut frame` is used to find the required size of the `LocalGcFrame`.
//!
//! [`Scope::scope`]: crate::memory::scope::Scope::scope
//! [`LocalScope::local_scope`]: crate::memory::scope::LocalScope::local_scope
//! [`GcFrame`]: crate::memory::target::frame::GcFrame
//! [`LocalGcFrame`]: crate::memory::target::frame::LocalGcFrame
//! [`Output`]: crate::memory::target::output::Output
//! [`GcFrame::scope`]: crate::memory::target::frame::GcFrame::scope
//! [`GcFrame::output`]: crate::memory::target::frame::GcFrame::output
//! [`Target`]: crate::memory::target::Target
//! [`Target::into_extended_target`]: crate::memory::target::Target::into_extended_target
//! [`Value`]: crate::data::managed::value::Value
//! [`WeakValue`]: crate::data::managed::value::WeakValue
//! [`ValueData`]: crate::data::managed::value::ValueData
//! [`ValueResult`]: crate::data::managed::value::ValueResult
//! [`Managed`]: crate::data::managed::Managed
//! [`ExtendedTarget`]: crate::memory::target::ExtendedTarget

pub(crate) mod context;
pub mod gc;
pub mod scope;
pub mod stack_frame;
pub mod target;

use jl_sys::{jl_tls_states_t, jlrs_get_ptls_states};

pub type PTls = *mut jl_tls_states_t;

#[inline]
pub(crate) unsafe fn get_tls() -> PTls {
    jlrs_get_ptls_states()
}
