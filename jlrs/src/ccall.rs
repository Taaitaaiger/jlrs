//! Interact with Julia when calling Rust from Julia.
//!
//! This module is only available if the `ccall` feature is enabled.

use std::{ptr::NonNull, cell::RefCell};

use crate::{
    error::JlrsResult,
    memory::{
        // stack_page::StackPage,
        context::{Stack, ContextFrame},
        frame::{GcFrame, NullFrame},
        global::Global, ledger::Ledger,
    },
};

use cfg_if::cfg_if;
#[cfg(feature = "uv")]
use jl_sys::uv_async_send;

/// Use Julia from a Rust function called through `ccall`.
///
/// When you call Rust from Julia through `ccall`, Julia has already been initialized and trying to
/// initialize it again would cause a crash. In order to still be able to call Julia from Rust
/// and to borrow arrays (if you pass them as `Array` rather than `Ptr{Array}`), you'll need to
/// create a scope first. You can use this struct to do so. It must never be used outside
/// functions called through `ccall`, and only once for each `ccall`ed function.
///
/// If you only need to use a frame to borrow array data, you can use [`CCall::null_scope`].
/// Unlike the runtimes, `CCall` postpones the allocation of the stack that is used for managing
/// the GC until a `GcFrame` is created. If a null scope is created, this stack isn't allocated at
/// all.
pub struct CCall<'context> {
    // page: Option<StackPage>,
    context_frame: &'context ContextFrame,
    pub(crate) ledger: RefCell<Ledger>,
}

impl<'context> CCall<'context> {
    /// Create a new `CCall`. The stack is not allocated until a [`GcFrame`] is created.
    ///
    /// Safety: This function must never be called outside a function called through `ccall` from
    /// Julia and must only be called once during that call.
    pub unsafe fn new(context_frame: &'context ContextFrame) -> Self {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                let rtls = NonNull::new_unchecked(jl_sys::jl_get_ptls_states()).as_mut();
                context_frame.set_prev(rtls.pgcstack.cast());
                rtls.pgcstack = context_frame as *const _ as *mut _;
            } else {
                use jl_sys::{jl_get_current_task, jl_task_t};
                let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();
                context_frame.set_prev(task.gcstack.cast());
                task.gcstack = context_frame as *const _ as *mut _;
            }
        }

        CCall { context_frame, ledger: RefCell::default() }
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
        for<'base> F: FnOnce(Global<'base>, GcFrame<'base>) -> JlrsResult<T>,
    {
        unsafe {
            let ctx = self.get_context();
            let global = Global::new();
            let (frame, owner) = GcFrame::base(ctx, &self.ledger);
            let ret = func(global, frame);
            std::mem::drop(owner);
            ret
        }
    }

    /// Create a [`NullFrame`] and call the given closure.
    ///
    /// A [`NullFrame`] cannot be nested and cannot store any roots.
    pub fn null_scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(NullFrame<'base>) -> JlrsResult<T>,
    {
        unsafe {
            let frame = NullFrame::new(self);
            func(frame)
        }
    }

    fn get_context(&mut self) -> &'context Stack {
        if let Some(ctx) = self.context_frame.get() {
            return ctx;
        }

        unsafe {
            let ctx_ty = Stack::init();
            let ctx = Stack::new(ctx_ty);
            self.context_frame.set(ctx)
        }
    }
}

impl Drop for CCall<'_> {
    fn drop(&mut self) {
        unsafe {
            cfg_if::cfg_if! {
                if #[cfg(feature = "lts")] {
                    let rtls = NonNull::new_unchecked(jl_sys::jl_get_ptls_states()).as_mut();
                    rtls.pgcstack = self.context_frame.prev().cast();
                } else {
                    use jl_sys::{jl_get_current_task, jl_task_t};
                    let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();
                    task.gcstack = self.context_frame.prev().cast();
                }
            }
        }
    }
}
