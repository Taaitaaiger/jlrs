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
use crate::{
    catch::{Exception, catch_exceptions},
    memory::target::frame::PinnedLocalFrame,
};

struct Droppable<'a, 'scope, const N: usize>(&'a PinnedLocalFrame<'scope, N>);
impl<'a, 'scope, const N: usize> Drop for Droppable<'a, 'scope, N> {
    fn drop(&mut self) {
        unsafe {
            self.0.pop();
        }
    }
}

/// Create new local scopes, local scopes can store a prespecified number of roots.
pub unsafe trait LocalScope: private::LocalScopePriv {
    /// Create a local scope with capacity for `N` roots and call `func`.
    ///
    /// NB: It is UB for an unhandled Julia exception to be thrown by `func`; use
    /// [`LocalScope::exception_safe_local_scope`] if `func` may throw an exception.
    #[inline]
    fn local_scope<T, const N: usize>(
        &self,
        func: impl for<'scope> FnOnce(LocalGcFrame<'scope, N>) -> T,
    ) -> T {
        let mut local_frame = LocalFrame::new();
        unsafe {
            let pinned = Droppable(&local_frame.pin());
            let ret = func(LocalGcFrame::new(pinned.0));
            std::mem::drop(pinned);
            ret
        }
    }

    /// Create a local scope with capacity for `N` roots and call `func`.
    ///
    /// Safety:
    ///
    /// It is safe to jump out of this scope when a Julia exception is thrown in `func`.
    /// It is UB for `func` to panic; this corrupts the GC stack.
    unsafe fn exception_safe_local_scope<T, const N: usize>(
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

    /// Call [`LocalScope::exception_safe_local_scope`] inside [`catch_exceptions`].
    ///
    /// Safety:
    ///
    /// It is safe to jump out of this scope when a Julia exception is thrown in `func`.
    /// It is UB for `func` to panic; this corrupts the exception stack.
    unsafe fn catching_local_scope<T, E, const N: usize>(
        &self,
        func: impl for<'scope> FnOnce(LocalGcFrame<'scope, N>) -> T,
        exception_handler: impl for<'exc> FnOnce(Exception<'exc, '_>) -> E,
    ) -> Result<T, E> {
        let cb = || unsafe { self.exception_safe_local_scope(func) };
        unsafe { catch_exceptions(cb, exception_handler) }
    }

    /// Create a local scope with capacity for `size` roots and call `func`.
    ///
    /// Safety:
    ///
    /// It is safe to jump out of this scope when a Julia exception is thrown in `func`.
    /// It is UB for `func` to panic; this corrupts the GC stack. If `func` panics, the process is
    /// aborted.
    #[inline]
    unsafe fn unsized_local_scope<T>(
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

    /// Call [`LocalScope::unsized_local_scope`] inside [`catch_exceptions`].
    ///
    /// Safety:
    ///
    /// It is UB for `func` to panic; this corrupts the GC stack. If `func` panics, the process is
    /// aborted.
    unsafe fn catching_unsized_local_scope<T, E>(
        &self,
        size: usize,
        func: impl for<'scope> FnOnce(UnsizedLocalGcFrame<'scope>) -> T,
        exception_handler: impl for<'exc> FnOnce(Exception<'exc, '_>) -> E,
    ) -> Result<T, E> {
        let cb = || unsafe { self.unsized_local_scope(size, func) };
        unsafe { catch_exceptions(cb, exception_handler) }
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
            let pinned = Droppable(&local_frame.pin());
            let ret = func(self, LocalGcFrame::new(pinned.0));
            std::mem::drop(pinned);
            ret
        }
    }

    /// Create a local scope with capacity for `N` roots and call `func`.
    ///
    /// Safety:
    ///
    /// It is safe to jump out of this scope when a Julia exception is thrown in `func`.
    /// It is UB for `func` to panic; this corrupts the GC stack.
    unsafe fn with_exception_safe_local_scope<T, const N: usize>(
        self,
        func: impl for<'scope> FnOnce(Self, LocalGcFrame<'scope, N>) -> T,
    ) -> T {
        let mut local_frame = LocalFrame::new();
        unsafe {
            let pinned = local_frame.pin();
            let res = func(self, LocalGcFrame::new(&pinned));
            pinned.pop();
            res
        }
    }

    /// Create a new unsized local scope with capacity for `size` roots and call `func`.
    ///
    /// Safety:
    ///
    /// It is safe to jump out of this scope when a Julia exception is thrown in `func`.
    /// It is UB for `func` to panic; this corrupts the GC stack. If `func` panics, the process is
    /// aborted.
    unsafe fn with_unsized_local_scope<T>(
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
