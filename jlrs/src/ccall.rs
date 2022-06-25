//! Interact with Julia when calling Rust from Julia.
//!
//! This module is only available if the `ccall` feature is enabled.

use crate::{
    error::JlrsResult,
    memory::{
        frame::{GcFrame, NullFrame},
        global::Global,
        mode::Sync,
        stack_page::StackPage,
    },
};

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
pub struct CCall {
    page: Option<StackPage>,
}

impl CCall {
    /// Create a new `CCall`. The stack is not allocated until a [`GcFrame`] is created.
    ///
    /// Safety: This function must never be called outside a function called through `ccall` from
    /// Julia and must only be called once during that call.
    pub unsafe fn new() -> Self {
        CCall { page: None }
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
        for<'base> F: FnOnce(Global<'base>, GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let page = self.get_init_page();
            let global = Global::new();
            let (frame, owner) = GcFrame::new(page.as_ref(), Sync);
            let ret = func(global, frame);
            std::mem::drop(owner);
            ret
        }
    }

    /// Create a [`GcFrame`] with capacity for at least `capacity` roots, call the given closure
    /// and return its result.
    pub fn scope_with_capacity<T, F>(&mut self, capacity: usize, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let page = self.get_init_page();
            let global = Global::new();
            if capacity + 2 > page.size() {
                *page = StackPage::new(capacity + 2);
            }
            let (frame, owner) = GcFrame::new(page.as_ref(), Sync);
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

    #[inline(always)]
    fn get_init_page(&mut self) -> &mut StackPage {
        if self.page.is_none() {
            self.page = Some(StackPage::default());
        }

        self.page.as_mut().unwrap()
    }
}
