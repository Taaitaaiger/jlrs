use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::GcFrame;
use crate::memory::context::stack::Stack;

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
    /// An async version of [`Scope::scope`] that takes an async closure.
    ///
    /// [`Scope::scope`]: crate::memory::scope::Scope::scope
    #[inline]
    pub async fn async_scope<T>(
        &mut self,
        func: impl for<'inner> AsyncFnOnce(AsyncGcFrame<'inner>) -> T,
    ) -> T {
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
    pub(crate) unsafe fn nest_async<'inner>(&'inner mut self) -> (usize, AsyncGcFrame<'inner>) {
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
