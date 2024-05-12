//! Interact with Julia when calling Rust from Julia.
//!
//! This module is only available if the `ccall` feature is enabled.

use jl_sys::{jl_throw, unsized_local_scope};

use crate::{
    convert::ccall_types::CCallReturn,
    data::{
        managed::{module::JlrsCore, private::ManagedPriv, value::ValueRet},
        types::construct_type::ConstructType,
    },
    error::JlrsResult,
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
    InstallJlrsCore,
};

/// Interact with Julia from a Rust function called through `ccall`.
///
/// When you call Rust from Julia through `ccall`, Julia has already been initialized and trying to
/// initialize it again causes a crash. In order to still be able to call Julia from Rust
/// you must create a scope first. You can use this struct to do so. It must never be used outside
/// functions called through `ccall`, and only once for each `ccall`ed function.
pub struct CCall<'context> {
    frame: PinnedFrame<'context, 0>,
}

impl<'context> CCall<'context> {
    /// Create a new `CCall`
    ///
    /// Safety: This function must never be called outside a function called through `ccall` from
    /// Julia and must only be called once during that call.
    #[inline]
    pub unsafe fn new(frame: &'context mut StackFrame<0>) -> Self {
        CCall { frame: frame.pin() }
    }

    /// Create a [`GcFrame`], call the given closure, and return its result.
    pub fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'scope> F: FnOnce(GcFrame<'scope>) -> JlrsResult<T>,
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
    pub unsafe fn local_scope<T, F, const N: usize>(func: F) -> JlrsResult<T>
    where
        for<'scope> F: FnOnce(LocalGcFrame<'scope, N>) -> JlrsResult<T>,
    {
        let mut local_frame = LocalFrame::new();

        let pinned = local_frame.pin();
        let res = func(LocalGcFrame::new(&pinned));
        pinned.pop();
        res
    }

    /// Create a new unsized local scope and call `func`.
    ///
    /// The `LocalGcFrame` provided to `func` has capacity for `size` roots.
    #[inline]
    pub unsafe fn unsized_local_scope<T, F>(&self, size: usize, func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(UnsizedLocalGcFrame<'inner>) -> JlrsResult<T>,
    {
        let mut func = Some(func);
        unsized_local_scope(size, |frame| {
            let frame = UnsizedLocalGcFrame::new(frame);
            func.take().unwrap()(frame)
        })
    }

    /// Create a [`LocalGcFrame`], call the given closure, and return its result.
    #[inline]
    pub unsafe fn infallible_local_scope<T, F, const N: usize>(func: F) -> T
    where
        for<'scope> F: FnOnce(LocalGcFrame<'scope, N>) -> T,
    {
        let mut local_frame = LocalFrame::new();

        let pinned = local_frame.pin();
        let res = func(LocalGcFrame::new(&pinned));
        pinned.pop();
        res
    }

    /// Create an instance of `CCall` and use it to invoke the provided closure.
    ///
    /// Safety: this method must only be called from `ccall`ed functions. The returned data is
    /// unrooted and must be returned to Julia immediately.
    #[inline(never)]
    pub unsafe fn invoke<T, F>(func: F) -> T
    where
        T: 'static + CCallReturn,
        for<'scope> F: FnOnce(GcFrame<'scope>) -> T,
    {
        let mut frame = StackFrame::new();
        let mut ccall = CCall::new(&mut frame);

        let stack = ccall.frame.stack_frame().sync_stack();
        let frame = GcFrame::base(stack);
        let ret = func(frame);
        stack.pop_roots(0);
        std::mem::drop(ccall);
        ret
    }

    /// Create an instance of `CCall` and use it to invoke the provided closure.
    ///
    /// Safety: this method must only be called from `ccall`ed functions. The returned data is
    /// unrooted and must be returned to Julia immediately.
    pub unsafe fn invoke_fallible<T, F>(func: F) -> JlrsResult<T>
    where
        T: ConstructType,
        for<'scope> F: FnOnce(GcFrame<'scope>) -> JlrsResult<T>,
    {
        let mut frame = StackFrame::new();
        let mut ccall = CCall::new(&mut frame);

        let stack = ccall.frame.stack_frame().sync_stack();
        let frame = GcFrame::base(stack);

        let ret = func(frame);
        stack.pop_roots(0);
        std::mem::drop(ccall);
        ret
    }

    /// Invoke the provided closure.
    ///
    /// Safety: this method must only be called from `ccall`ed functions. The returned data is
    /// unrooted and must be returned to Julia immediately.
    #[inline]
    pub unsafe fn stackless_invoke<T, F>(func: F) -> T
    where
        T: 'static + CCallReturn,
        for<'scope> F: FnOnce(Unrooted<'scope>) -> T,
    {
        func(Unrooted::new())
    }

    /// Throw an exception.
    ///
    /// Safety:
    ///
    /// Don't jump over any frames that have pendings drops. Julia exceptions are implemented with
    /// `setjmp` / `longjmp`. This means that when an exception is thrown, control flow is
    /// returned to a `catch` block by jumping over intermediate stack frames.
    #[inline]
    pub unsafe fn throw_exception(exception: ValueRet) -> ! {
        jl_throw(exception.ptr().as_ptr())
    }

    #[inline]
    pub unsafe fn throw_borrow_exception() -> ! {
        let unrooted = Unrooted::new();
        let err = JlrsCore::borrow_error(&unrooted).instance().unwrap();
        jl_throw(err.unwrap(Private))
    }

    /// Create an [`Unrooted`], call the given closure, and return its result.
    ///
    /// Unlike [`CCall::scope`] this method doesn't allocate a stack.
    ///
    /// Safety: must only be called from a `ccall`ed function that doesn't need to root any data.
    #[inline]
    pub unsafe fn stackless_scope<T, F>(func: F) -> JlrsResult<T>
    where
        for<'scope> F: FnOnce(Unrooted<'scope>) -> JlrsResult<T>,
    {
        func(Unrooted::new())
    }

    /// This function must be called before jlrs can be used. When the `julia_module` macro is
    /// used this function is called automatically.
    ///
    /// A module can be provided to allow setting the size of the internal thread pool from Julia
    /// by calling `JlrsCore.set_pool_size`.
    #[inline(never)]
    pub unsafe fn init_jlrs(&mut self, install_jlrs_core: &InstallJlrsCore) {
        set_started_from_julia();
        init_jlrs(install_jlrs_core);
    }
}
