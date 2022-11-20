//! Interact with Julia when calling Rust from Julia.
//!
//! This module is only available if the `ccall` feature is enabled.

use jl_sys::jl_throw;
#[cfg(feature = "uv")]
use jl_sys::uv_async_send;

use crate::{
    error::JlrsResult,
    memory::{
        context::stack::Stack,
        stack_frame::{PinnedFrame, StackFrame},
        target::{frame::GcFrame, unrooted::Unrooted},
    },
    private::Private,
    wrappers::ptr::{
        private::WrapperPriv,
        value::{Value, ValueRef},
    },
};

/// Use Julia from a Rust function called through `ccall`.
///
/// When you call Rust from Julia through `ccall`, Julia has already been initialized and trying to
/// initialize it again causes a crash. In order to still be able to call Julia from Rust
/// you must create a scope first. You can use this struct to do so. It must never be used outside
/// functions called through `ccall`, and only once for each `ccall`ed function.
///
/// Exceptions must only be thrown by calling [`CCall::throw_exception`].
pub struct CCall<'context> {
    frame: PinnedFrame<'context, 0>,
}

impl<'context> CCall<'context> {
    /// Create a new `CCall`
    ///
    /// Safety: This function must never be called outside a function called through `ccall` from
    /// Julia and must only be called once during that call.
    pub unsafe fn new(frame: &'context mut StackFrame<0>) -> Self {
        CCall { frame: frame.pin() }
    }

    /// Wake the task associated with `handle`.
    ///
    /// The handle must be the `handle` field of a `Base.AsyncCondition` in Julia. This can be
    /// used to call a long-running Rust function from Julia with `ccall` in another thread and
    /// wait for it to complete in Julia without blocking, an example is available in the
    /// repository: `ccall_with_threads`.
    ///
    /// This method is only available if the `uv` feature is enabled.
    ///
    /// Safety: the handle must be acquired from an `AsyncCondition`.
    #[cfg(feature = "uv")]
    pub unsafe fn uv_async_send(handle: *mut std::ffi::c_void) -> bool {
        uv_async_send(handle.cast()) == 0
    }

    /// Create a [`GcFrame`], call the given closure, and return its result.
    pub fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'scope> F: FnOnce(GcFrame<'scope>) -> JlrsResult<T>,
    {
        unsafe {
            let stack = self.frame.stack_frame().sync_stack();
            let (owner, frame) = GcFrame::base(stack);
            let ret = func(frame);
            std::mem::drop(owner);
            ret
        }
    }

    /// Create and throw an exception.
    ///
    /// This method calls `func` and throws the result as a Julia exception.
    ///
    /// Safety:
    ///
    /// Julia exceptions are implemented with `setjmp` / `longjmp`. This means that when an
    /// exception is thrown, control flow is returned to a `catch` block by jumping over
    /// intermediate stack frames. It's undefined behaviour to jump over frames that have pending
    /// drops, so you must take care to structure your code such that none of the intermediate
    /// frames have any pending drops.
    #[inline(never)]
    pub unsafe fn throw_exception<F>(mut self, func: F)
    where
        F: for<'scope> FnOnce(&mut GcFrame<'scope>) -> Value<'scope, 'static>,
    {
        let exception = construct_exception(self.frame.stack_frame().sync_stack(), func);
        // catch unwinds the GC stack, so it's okay to forget self.
        std::mem::forget(self);
        jl_throw(exception.ptr().as_ptr());
        unreachable!()
    }

    /// Create an [`Unrooted`], call the given closure, and return its result.
    ///
    /// Unlike [`CCall::scope`] this method doesn't allocate a stack.
    ///
    /// Safety: must only be called from a `ccall`ed function that doesn't need to root any data.
    pub unsafe fn stackless_scope<T, F>(func: F) -> JlrsResult<T>
    where
        for<'scope> F: FnOnce(Unrooted<'scope>) -> JlrsResult<T>,
    {
        func(Unrooted::new())
    }
}

#[inline(never)]
unsafe fn construct_exception<'stack, F>(stack: &'stack Stack, func: F) -> ValueRef<'stack, 'static>
where
    for<'scope> F: FnOnce(&mut GcFrame<'scope>) -> Value<'scope, 'static>,
{
    let (owner, mut frame) = GcFrame::base(stack);
    let ret = func(&mut frame);
    let rewrapped = ValueRef::wrap(ret.unwrap_non_null(Private));
    std::mem::drop(owner);
    rewrapped
}
