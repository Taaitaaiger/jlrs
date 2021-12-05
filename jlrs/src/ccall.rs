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

/// When you call Rust from Julia through `ccall`, Julia has already been initialized and trying to
/// initialize it again would cause a crash. In order to still be able to call Julia from Rust
/// and to borrow arrays (if you pass them as `Array` rather than `Ptr{Array}`), you'll need to
/// create a frame first. You can use this struct to do so. It must never be used outside
/// functions called through `ccall`, and only once for each `ccall`ed function.
///
/// If you only need to use a frame to borrow array data, you can use [`CCall::null_scope`].
/// Unlike [`Julia`], `CCall` postpones the allocation of the stack that is used for managing the
/// GC until a `GcFrame` is created. In the case of a null scope, this stack isn't allocated at
/// all.
///
/// [`Julia`]: crate::julia::Julia
pub struct CCall {
    page: Option<StackPage>,
}

impl CCall {
    /// Create a new `CCall`. This function must never be called outside a function called through
    /// `ccall` from Julia and must only be called once during that call. The stack is not
    /// allocated until a [`GcFrame`] is created.
    pub unsafe fn new() -> Self {
        CCall { page: None }
    }

    /// Wake the task associated with `handle`. The handle must be the `handle` field of a
    /// `Base.AsyncCondition` in Julia. This can be used to call a long-running Rust function from
    /// Julia with ccall in another thread and wait for it to complete in Julia without blocking,
    /// there's an example available in the repository: ccall_with_threads.
    ///
    /// This method is only available if the `uv` feature is enabled.
    #[cfg(feature = "uv")]
    pub unsafe fn uv_async_send(handle: *mut std::ffi::c_void) -> bool {
        uv_async_send(handle.cast()) == 0
    }

    /// Creates a [`GcFrame`], calls the given closure, and returns its result.
    pub fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, &mut GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let page = self.get_init_page();
            let global = Global::new();
            let mut frame = GcFrame::new(page.as_mut(), Sync);
            func(global, &mut frame)
        }
    }

    /// Creates a [`GcFrame`] with  capacity for at least `slots` roots, calls the given closure,
    /// and returns its result.
    pub fn scope_with_slots<T, F>(&mut self, slots: usize, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, &mut GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let page = self.get_init_page();
            let global = Global::new();
            if slots + 2 > page.size() {
                *page = StackPage::new(slots + 2);
            }
            let mut frame = GcFrame::new(page.as_mut(), Sync);
            func(global, &mut frame)
        }
    }

    /// Create a [`NullFrame`] and call the given closure. A [`NullFrame`] cannot be nested and
    /// can only be used to (mutably) borrow array data. Unlike other scope-methods, no `Global`
    /// is provided to the closure.
    pub fn null_scope<'base, 'julia: 'base, T, F>(&'julia mut self, func: F) -> JlrsResult<T>
    where
        F: FnOnce(&mut NullFrame<'base>) -> JlrsResult<T>,
    {
        unsafe {
            let mut frame = NullFrame::new(self);
            func(&mut frame)
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
