//! Local and dynamic scopes

use jl_sys::unsized_local_scope;

use super::target::{
    frame::{GcFrame, LocalFrame, LocalGcFrame, UnsizedLocalGcFrame},
    Target,
};

/// Set the return type of a local scope.
pub trait LocalReturning<'ctx> {
    /// Set the return type of the scope to `T`.
    ///
    /// This method can be used instead of setting the return type in the closure provided to a
    /// scope.
    fn returning<T>(&mut self) -> &mut impl LocalScope<'ctx, T>;
}

/// Set the return type of a scope.
pub trait Returning<'ctx> {
    /// Set the return type of the scope to `T`.
    ///
    /// This method can be used instead of setting the return type in the closure provided to a
    /// scope.
    fn returning<T>(&mut self) -> &mut impl Scope<'ctx, T>;
}

/// Create new local scopes.
pub trait LocalScope<'a, T> {
    /// Create a local scope with capacity for `N` roots and call `func`.
    #[inline]
    fn local_scope<const N: usize>(
        &self,
        func: impl for<'scope> FnOnce(LocalGcFrame<'scope, N>) -> T,
    ) -> T {
        unsafe {
            let mut local_frame = LocalFrame::new();
            let pinned = local_frame.pin();
            let res = func(LocalGcFrame::new(&pinned));
            pinned.pop();
            res
        }
    }

    /// Create a local scope with capacity for `size` roots and call `func`.
    #[inline]
    fn unsized_local_scope(
        &self,
        size: usize,
        func: impl for<'scope> FnOnce(UnsizedLocalGcFrame<'scope>) -> T,
    ) -> T {
        unsafe {
            let mut func = Some(func);
            unsized_local_scope(size, |frame| {
                let frame = UnsizedLocalGcFrame::new(frame);
                func.take().unwrap()(frame)
            })
        }
    }
}

/// Create new local scopes from a target.
pub trait LocalScopeExt<'target, T>: LocalScope<'target, T> + Target<'target> {
    /// Create a new local scope and call `func`.
    ///
    /// The `LocalGcFrame` has capacity for `N` roots, `self` is propagated to
    /// the closure.
    #[inline]
    fn with_local_scope<const N: usize>(
        self,
        func: impl for<'inner> FnOnce(Self, LocalGcFrame<'inner, N>) -> T,
    ) -> T {
        unsafe {
            let mut local_frame = LocalFrame::new();
            let pinned = local_frame.pin();
            let res = func(self, LocalGcFrame::new(&pinned));
            pinned.pop();
            res
        }
    }

    /// Create a new local scope and call `func`.
    ///
    /// The `LocalGcFrame` has capacity for `size` roots, `self` is propagated to
    /// the closure.
    #[inline]
    fn with_unsized_local_scope(
        self,
        size: usize,
        func: impl for<'scope> FnOnce(Self, UnsizedLocalGcFrame<'scope>) -> T,
    ) -> T {
        unsafe {
            let mut func = Some(func);
            let mut self_container = Some(self);
            unsized_local_scope(size, |frame| {
                let frame = UnsizedLocalGcFrame::new(frame);
                func.take().unwrap()(self_container.take().unwrap(), frame)
            })
        }
    }
}

/// Create new scopes.
pub trait Scope<'a, T>: LocalScope<'a, T> {
    /// Create a new scope and call `func`.
    fn scope(&mut self, func: impl for<'scope> FnOnce(GcFrame<'scope>) -> T) -> T;
}
