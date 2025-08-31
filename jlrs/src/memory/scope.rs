//! Local, dynamic, and async scopes.
//!
//! All interactions with Julia must happen inside a scope. Inside a scope, a frame can be used
//! to protect data from being garbage collected. The frame is dropped when the scope ends.
//!
//! A local scope is created on the stack, and can hold a definite number of roots. Dynamic and
//! async scopes use a heap-allocated stack to store their frames, their frames can grow to any
//! size. Async scopes support async operations, and are only used with the async runtime.

use jlrs_sys::unsized_local_scope;

use super::target::{
    Target,
    frame::{GcFrame, LocalFrame, LocalGcFrame, UnsizedLocalGcFrame},
};

/// Create new local scopes, local scopes can store a prespecified number of roots.
pub unsafe trait LocalScope: private::LocalScopePriv {
    /// Create a local scope with capacity for `N` roots and call `func`.
    #[inline]
    fn local_scope<T, const N: usize>(
        &self,
        func: impl for<'scope> FnOnce(LocalGcFrame<'scope, N>) -> T,
    ) -> T {
        let mut local_frame = LocalFrame::new();
        unsafe {
            let pinned = local_frame.pin();
            let res = func(LocalGcFrame::new(&pinned));
            pinned.pop();
            res
        }
    }

    /// Create a local scope with capacity for `size` roots and call `func`.
    #[inline]
    fn unsized_local_scope<T>(
        &self,
        size: usize,
        func: impl for<'scope> FnOnce(UnsizedLocalGcFrame<'scope>) -> T,
    ) -> T {
        let mut func = Some(func);
        unsafe {
            unsized_local_scope(size, |frame| {
                let frame = UnsizedLocalGcFrame::new(frame);
                func.take().unwrap()(frame)
            })
        }
    }
}

/// Create new local scopes from a target that propagate the target to the new scope.
pub unsafe trait LocalScopeExt<'target>: Target<'target> {
    /// Create a new local scope and call `func` with the target and new frame.
    ///
    /// The `LocalGcFrame` has capacity for `N` roots.
    #[inline]
    fn with_local_scope<T, const N: usize>(
        self,
        func: impl for<'inner> FnOnce(Self, LocalGcFrame<'inner, N>) -> T,
    ) -> T {
        let mut local_frame = LocalFrame::new();
        unsafe {
            let pinned = local_frame.pin();
            let res = func(self, LocalGcFrame::new(&pinned));
            pinned.pop();
            res
        }
    }

    /// Create a new unsized local scope and call `func` with target and new frame.
    ///
    /// The `UnsizedLocalGcFrame` has capacity for `size` roots.
    fn with_unsized_local_scope<T>(
        self,
        size: usize,
        func: impl for<'scope> FnOnce(Self, UnsizedLocalGcFrame<'scope>) -> T,
    ) -> T {
        let mut func = Some(func);
        let mut self_container = Some(self);

        unsafe {
            unsized_local_scope(size, |frame| {
                let frame = UnsizedLocalGcFrame::new(frame);
                func.take().unwrap()(self_container.take().unwrap(), frame)
            })
        }
    }
}

/// Create new dynamically-sized scopes.
pub unsafe trait Scope: LocalScope {
    /// Create a new dynamically-sized scope and call `func`.
    fn scope<T>(&mut self, func: impl for<'scope> FnOnce(GcFrame<'scope>) -> T) -> T;
}

/// Create new dynamically-sized, async scopes.
#[cfg(feature = "async")]
pub unsafe trait AsyncScope: Scope {
    /// An async version of [`Scope::scope`] that takes an async closure.
    fn async_scope<T>(
        &mut self,
        func: impl for<'inner> AsyncFnOnce(crate::prelude::AsyncGcFrame<'inner>) -> T,
    ) -> impl std::future::Future<Output = T>;
}

pub(crate) mod private {
    pub trait LocalScopePriv {}
}
