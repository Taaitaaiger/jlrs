//! Julia memory management.
//!
//! This module contains most of the structs and traits that are used to deal with memory
//! management and enforcing some degree of compile-time memory safety by applying reasonable
//! lifetime bounds to Julia data. The rest of the documentation in this module provides
//! information about how the design of this module has been influenced by the Julia C API
//! (C API).
//!
//! Julia is a garbage-collected programming language. The garbage collector (GC) is a tracing GC,
//! it has a mark and a sweep phase; during the mark phase the GC uses a set of root pointers
//! (roots) to mark all data that can be reached from these roots, unreachable data is freed
//! during the sweep phase. Internally, this set of pointers is a stack, or a linked list, of
//! dynamically sized GC frames (frames). Two imporant invariants that must be maintained are that
//! each scope must only create one frame, and that a frame must be popped before returning from
//! the scope that created it.
//!
//! Whenever a function from the C API returns data owned by the GC, it's likely that this data is
//! unreachable. In order to root the data a frame must be created first. While the C API provides
//! several macros to create a frame, these macros can't be converted to Rust because they
//! allocate a dynamically-sized array on the stack.
//!
//! In order to work around the issue that frames are dynamically sized, jlrs allocates some
//! memory on the heap to store one or more frames and manipulates the contents to ensure the
//! GC sees a consistent stack. Several methods to create a new scope are available, these methods
//! often take a closure which takes a mutable reference to a [`GcFrame`]. After creating a frame,
//! the closure is called with a mutable reference to that frame. When the closure returns the
//! frame is popped from the stack.
//!
//! The C API requires you to manually root data in a frame, methods in jlrs that return Julia
//! data ensure this data is automatically rooted in some frame. All frame types provided by jlrs
//! have a lifetime because they borrow a slice from the memory allocated to store the frame
//! stack. Types that wrap a pointer to rooted Julia data, pointer wrappers, have at least one
//! lifetime. This lifetime is determined by the lifetime of the frame used to root the data. As a
//! result of this lifetime, this data can't be returned from the scope whose frame roots it.
//!
//! Two other features provided by frames in jlrs is the ability to reserve outputs and create
//! nested scopes. The main use case of nested scopes is ensuring temporary data isn't rooted
//! longer than necessary. In order to return rooted data from this nested scope, it must have
//! been rooted in a parent frame. Inside the nested scope only the current frame can be accessed,
//! in order to be able to root data in a parent frame an [`Output`] must first be reserved in
//! that frame.
//!
//! Methods provided by jlrs that return rooted Julia data take either a [`PartialScope`] or
//! [`Scope`]. The main difference is that methods that take a `PartialScope` only need to
//! allocate a single value, while methods that take a `Scope` need to root one or more temporary
//! values in addition to the result. The `Scope` trait is implemented for all mutable references
//! to a frame and for [`OutputScope`], which contains an `Output` and a mutable reference to a
//! frame. All these types implement `PartialScope`, `Output` does too. Because a wrapper inherits
//! its lifetime from the frame that roots it, data rooted in a parent scope can be returned from
//! a nested scope.
//!
//! Examples and more information can be found in the [`frame`] and [`scope`] modules.
//!
//! [`Global`]: global::Global
//! [`GcFrame`]: frame::GcFrame
//! [`PartialScope`]: scope::PartialScope
//! [`Scope`]: scope::Scope
//! [`Frame`]: frame::Frame
//! [`Output`]: output::Output
//! [`OutputScope`]: output::OutputScope
//! [`Output::into_scope`]: output::Output::into_scope
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
