//! Julia memory management.
//!
//! All Julia data (objects) is owned by the garbage collector (GC). The GC is a mark-and-sweep
//! GC, during the mark phase all objects that can be reached from a root are marked as reachable,
//! during the sweep phase all unreachable objects are freed.
//!
//! Roots are pointers to objects that are present on the GC stack. The GC stack is a linked list
//! of GC frames. A C function that needs to protect one or more objects allocates an
//! appropriately-sized GC frame and pushes it to the GC stack, typically at the beginning of the
//! function call, stores the pointers to objects that are returned by the Julia C API in that
//! frame, and pops the frame from the GC stack before returning from the function.
//!
//! There are several problems with this approach from a Rust perspective. The API is pretty
//! unsafe because it depends on manually pushing and popping GC frame to and from the GC stack,
//! and manually inserting pointers into the frame. More importantly, the GC frame is a
//! dynamically-sized type that is allocated on the stack with `alloca`. This is simply not
//! supported by Rust, at least not without jumping through a number awkward and limiting hoops.
//!
//! In order to work around these issues, jlrs doesn't store the roots in the frame, but uses a
//! custom object that contains a `Vec` of roots (stack). This stack is not part of the public
//! API, you can only interact with it indirectly by creating a scope first. The sync runtime lets
//! you create one directly with [`Julia::scope`], this method takes a closure which takes a
//! single argument, a [`GcFrame`]. This `GcFrame` can access the internal stack and push new
//! roots to it. The roots that are associated with a `GcFrame` are popped from the stack after
//! the closure returns.
//!
//! Rather than returning raw pointers to objects, jlrs wraps these pointers in types that
//! implement the [`Managed`] trait. Methods that return such types typically take an argument
//! that implements the [`Target`] trait, which ensures the object is rooted. The returned managed
//! type inherits the lifetime of the target to ensure this data can't be used after the scope
//! whose  `GcFrame` has rooted it ends. Mutable references to `GcFrame` implement `Target`, when
//! one is used as a target the returned data remains rooted until the scope ends.
//!
//! Often you'll need to create some Julia data that doesn't need to live as long as the current
//! scope. A nested scope, with its own `GcFrame`, can be created by calling [`GcFrame::scope`].
//! In order to return managed data from a child scope it has to be rooted in the `GcFrame` of a
//! parent scope, but this `GcFrame` can't be accessed from the child scope. Instead, you can
//! create an [`Output`] by reserving a slot on the stack by calling  [`GcFrame::output`]. When an
//! `Output` is used as a target, the reserved slot is used to root the data and the returned
//! managed type inherits the lifetime of the parent scope, allowing it to be returned from the
//! child scope.
//!
//! Several other target types exist, they can all be created through methods defined for
//! `GcFrame`. You can find more information about them in the [`target`] module.
//!
//! Not all targets root the returned data. If you never need to use the data (e.g. because you
//! call a function that returns `nothing`), or you access a global value in a module that is
//! never mutated while you're using it, the result doesn't need to be rooted. A reference to a
//! rooting target is guaranteed to be a valid non-rooting target. When a non-rooting target is
//! used, the function doesn't return an instance of a managed type, but a [`Ref`] to a managed
//! type to indicate the data has not been rooted.
//!
//! [`Julia::scope`]: crate::runtime::sync_rt::Julia::scope
//! [`GcFrame`]: crate::memory::target::frame::GcFrame
//! [`Output`]: crate::memory::target::output::Output
//! [`GcFrame::scope`]: crate::memory::target::frame::GcFrame::scope
//! [`GcFrame::output`]: crate::memory::target::frame::GcFrame::output
//! [`Target`]: crate::memory::target::Target
//! [`Ref`]: crate::data::managed::Ref
//! [`Managed`]: crate::data::managed::Managed

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
