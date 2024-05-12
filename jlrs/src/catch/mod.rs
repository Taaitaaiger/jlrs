//! Try-catch blocks.
//!
//! Many functions in Julia can throw exceptions, jlrs provides checked and unchecked variants of
//! such functions. The checked variant calls the function in a try-catch block and returns a
//! `Result` to indicate whether or not the operation succeeded, while the unchecked variant
//! simply calls the function. If an exception is thrown and it isn't caught the application is
//! aborted. The main disadvantage of the checked variants is that a new try-catch block is
//! created every time the function is called and creating such a block is relatively expensive.
//!
//! Instead of using the checked variants you can create a try-catch block from Rust with
//! [`catch_exceptions`]. This function takes two closures, think of them as the content of the
//! try and catch blocks respectively.
//!
//! Because exceptions work by jumping to the nearest enclosing catch block, you must guarantee
//! that there are no pending drops when an exception is thrown. See this [blog post] for more
//! information.
//!
//! Only local scopes may be created in the try-block, Julia's unwinding mechanism ensures that
//! any scope we jump out of is removed from the GC stack. Dynamic scopes (i.e. scopes that
//! provide a `GcFrame`) depend on `Drop` so jumping out of them is not sound.
//!
//! [blog post]: https://blog.rust-lang.org/inside-rust/2021/01/26/ffi-unwind-longjmp.html#pofs-and-stack-deallocating-functions

use std::ptr::NonNull;

#[path = "impl_stable.rs"]
mod imp;

pub use imp::catch_exceptions;
use jl_sys::jl_value_t;

use crate::{data::managed::private::ManagedPriv, prelude::Value, private::Private};

#[inline]
pub(crate) fn unwrap_exc(exc: Value) -> NonNull<jl_value_t> {
    exc.unwrap_non_null(Private)
}
