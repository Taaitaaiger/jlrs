use std::{
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::GcFrame;
use crate::{memory::context::stack::Stack, prelude::JlrsResult};

/// A frame associated with an async scope.
///
/// The only difference between a `GcFrame` and an `AsyncGcFrame` is that the latter
/// allows calling several async methods, most importantly those of [`CallAsync`]. An
/// `AsyncGcFrame` can be (mutably) dereferenced as a `GcFrame`, so all methods of `GcFrame`
/// are available to `AsyncGcFrame`.
///
/// [`CallAsync`]: crate::call::CallAsync
pub struct AsyncGcFrame<'scope> {
    frame: GcFrame<'scope>,
}

impl<'scope> AsyncGcFrame<'scope> {
    // pub fn stack_addr(&self) -> *const c_void {
    //     self.frame.stack_addr()
    // }

    /// An async version of [`Scope::scope`].
    ///
    /// The closure `func` must return an async block. Note that the returned value is
    /// required to live at least as long the current frame.
    ///
    /// If you can target at least Rust 1.85, it's recommended to enable the `async-closure`
    /// feature and use `AsyncGcFrame::async_scope_closure` instead. This method will be removed
    /// in the future.
    ///
    /// [`Scope::scope`]: crate::memory::scope::Scope::scope

    #[inline]
    pub async fn async_scope<'nested, T, F, G>(&'nested mut self, func: F) -> JlrsResult<T>
    where
        T: 'scope,
        G: Future<Output = JlrsResult<T>>,
        F: FnOnce(AsyncGcFrame<'nested>) -> G,
    {
        // Safety: the lifetime of the borrow is extended, but it's valid during the call
        // to func and data returned from func must live longer.
        unsafe {
            let stack = self.stack;
            let (offset, nested) = self.nest_async();
            let ret = func(nested).await;
            stack.pop_roots(offset);
            ret
        }
    }

    /// `AsyncGcFrame::async_scope` with less strict lifetime bounds on the return value.
    ///
    /// If you can target at least Rust 1.85, it's recommended to enable the `async-closure`
    /// feature and use `AsyncGcFrame::async_scope_closure` instead. This method will be removed
    /// in the future.
    ///
    /// Safety: because this method only requires that the returned data lives at least as
    /// long as the borrow of `self`, it's possible to return data rooted in that scope
    /// which you must not do.

    #[inline]
    pub async unsafe fn relaxed_async_scope<'nested, T, F, G>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<T>
    where
        T: 'nested,
        G: Future<Output = JlrsResult<T>>,
        F: FnOnce(AsyncGcFrame<'nested>) -> G,
    {
        let stack = self.stack;
        let (offset, nested) = self.nest_async();
        let ret = func(nested).await;
        unsafe {
            stack.pop_roots(offset);
        }
        ret
    }

    /// An async version of [`Scope::scope`] that takes an async closure.
    ///
    /// This method is only available when the `async-closures` feature is enabled and requires
    /// using at least Rust 1.85.
    #[cfg(feature = "async-closure")]
    #[inline]
    pub async fn async_scope_closure<T, F>(&mut self, func: F) -> T
    where
        for<'nested> F: AsyncFnOnce(AsyncGcFrame<'nested>) -> T,
    {
        unsafe {
            let stack = self.stack;
            let (offset, nested) = self.nest_async();
            let ret = func(nested).await;
            stack.pop_roots(offset);
            ret
        }
    }

    // Safety: only one base frame may exist per `Stack`
    #[inline]
    pub(crate) unsafe fn base(stack: &'scope Stack) -> AsyncGcFrame<'scope> {
        AsyncGcFrame {
            frame: GcFrame {
                stack,
                offset: stack.size(),
                _marker: PhantomData,
            },
        }
    }

    #[inline]
    pub(crate) unsafe fn nest_async<'nested>(&'nested mut self) -> (usize, AsyncGcFrame<'nested>) {
        let (offset, frame) = self.nest();
        (offset, AsyncGcFrame { frame: frame })
    }

    #[inline]
    pub(crate) fn stack(&self) -> &'scope Stack {
        self.frame.stack
    }
}

impl<'scope> Deref for AsyncGcFrame<'scope> {
    type Target = GcFrame<'scope>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl<'scope> DerefMut for AsyncGcFrame<'scope> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frame
    }
}
