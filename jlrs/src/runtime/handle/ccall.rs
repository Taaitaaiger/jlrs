//! Interact with Julia when calling Rust from Julia.
//!
//! This module is only available if the `ccall` feature is enabled, most functionality is
//! deprecated in favor of using [`weak_handle`] instead.
//!
//! [`weak_handle`]: crate::weak_handle

use jl_sys::jl_throw;
use jlrs_sys::unsized_local_scope;

use crate::{
    InstallJlrsCore,
    data::managed::{module::JlrsCore, private::ManagedPriv, value::ValueRet},
    init_jlrs,
    memory::{
        stack_frame::{PinnedFrame, StackFrame},
        target::{
            frame::{GcFrame, LocalFrame, LocalGcFrame, UnsizedLocalGcFrame},
            unrooted::Unrooted,
        },
    },
    private::Private,
    runtime::state::set_started_from_julia,
};

/// Interact with Julia from a Rust function called through `ccall`. You should use
/// [`weak_handle`] instead.
///
/// When you call Rust from Julia through `ccall`, Julia has already been initialized and trying to
/// initialize it again causes a crash. In order to still be able to call Julia from Rust
/// you must create a scope first. You can use this struct to do so. It must never be used outside
/// functions called through `ccall`, and only once for each `ccall`ed function.
///
/// [`weak_handle`]: crate::weak_handle
pub struct CCall<'context> {
    frame: PinnedFrame<'context, 0>,
}

impl<'context> CCall<'context> {
    /// Create a new `CCall`
    ///
    /// Safety: This function must never be called outside a function called through `ccall` from
    /// Julia and must only be called once during that call.
    #[inline]
    #[deprecated = "Use weak_handle instead"]
    pub unsafe fn new(frame: &'context mut StackFrame<0>) -> Self {
        unsafe { CCall { frame: frame.pin() } }
    }

    /// Create a [`GcFrame`], call the given closure, and return its result.
    #[deprecated = "Use weak_handle and WithStack instead"]
    pub fn scope<T, F>(&mut self, func: F) -> T
    where
        for<'scope> F: FnOnce(GcFrame<'scope>) -> T,
    {
        unsafe {
            let stack = self.frame.stack_frame().sync_stack();
            let frame = GcFrame::base(stack);
            let ret = func(frame);
            stack.pop_roots(0);
            ret
        }
    }

    /// Create a [`LocalGcFrame`], call the given closure, and return its result.
    #[inline]
    #[deprecated = "Use weak_handle instead"]
    pub unsafe fn local_scope<T, F, const N: usize>(func: F) -> T
    where
        for<'scope> F: FnOnce(LocalGcFrame<'scope, N>) -> T,
    {
        unsafe {
            let mut local_frame = LocalFrame::new();

            let pinned = local_frame.pin();
            let res = func(LocalGcFrame::new(&pinned));
            pinned.pop();
            res
        }
    }

    /// Create a new unsized local scope and call `func`.
    ///
    /// The `LocalGcFrame` provided to `func` has capacity for `size` roots.
    #[inline]
    #[deprecated = "Use weak_handle instead"]
    pub unsafe fn unsized_local_scope<T, F>(&self, size: usize, func: F) -> T
    where
        for<'inner> F: FnOnce(UnsizedLocalGcFrame<'inner>) -> T,
    {
        unsafe {
            let mut func = Some(func);
            unsized_local_scope(size, |frame| {
                let frame = UnsizedLocalGcFrame::new(frame);
                func.take().unwrap()(frame)
            })
        }
    }
}

/// Throw an exception.
///
/// Must never be called from a function exported with `julia_module`, return a `Result` instead.
///
/// Safety:
///
/// Don't jump over any frames that have pendings drops. Julia exceptions are implemented with
/// `setjmp` / `longjmp`. This means that when an exception is thrown, control flow is
/// returned to a `catch` block by jumping over intermediate stack frames.
#[inline]
pub unsafe fn throw_exception(exception: ValueRet) -> ! {
    unsafe { jl_throw(exception.ptr().as_ptr()) }
}

#[doc(hidden)]
#[inline]
pub unsafe fn throw_borrow_exception() -> ! {
    unsafe {
        let unrooted = Unrooted::new();
        let err = JlrsCore::borrow_error(&unrooted).instance().unwrap();
        jl_throw(err.unwrap(Private))
    }
}

/// This function must be called before jlrs can be used. When the `julia_module` macro is
/// used this function is called automatically.
///
/// A module can be provided to allow setting the size of the internal thread pool from Julia
/// by calling `JlrsCore.set_pool_size`.
#[inline(never)]
pub unsafe fn init_jlrs_wrapped(install_jlrs_core: &InstallJlrsCore) {
    unsafe {
        set_started_from_julia();
        init_jlrs(install_jlrs_core, false);
    }
}
