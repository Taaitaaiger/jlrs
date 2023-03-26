//! A frame roots data until its scope ends.
//!
//! Every scope has its own frame which can hold an arbitrary number of roots. When the scope
//! ends these roots are removed from the set of roots, so all data rooted in a frame can safely
//! be used until its scope ends. This hold true even if the frame is dropped before its scope
//! ends.
//!
//! In addition to being usable as targets, frames can also be used to create [`Output`]s,
//! [`ReusableSlot`]s, [`Unrooted`]s, and child scopes with their own frame.

use std::{marker::PhantomData, ptr::NonNull};

use cfg_if::cfg_if;

use super::{output::Output, reusable_slot::ReusableSlot, unrooted::Unrooted};
use crate::{
    data::managed::Managed,
    error::JlrsResult,
    memory::{
        context::stack::Stack,
        target::{ExtendedTarget, Target},
    },
    private::Private,
};

/// A frame associated with a scope.
///
/// Mutable references to a `GcFrame` can be used as a target, in this case the data will be
/// rooted until the frame's scope ends.  Other targets can be created through a frame. For
/// example, [`GcFrame::output`] creates a new `Output` that targets the current frame.
pub struct GcFrame<'scope> {
    stack: &'scope Stack,
    offset: usize,
    _marker: PhantomData<&'scope mut &'scope ()>,
}

impl<'scope> GcFrame<'scope> {
    /// Returns a mutable reference to this frame.
    #[inline]
    pub fn as_mut(&mut self) -> &mut Self {
        self
    }

    /// Reserve capacity for at least `additional` roots.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.stack.reserve(additional)
    }

    /// Borrow the current frame.
    ///
    /// When a frame is borrowed, no more roots can be pushed until a new scope has been created.
    /// This is useful when a function needs to root Julia data but doesn't return Julia data.
    #[inline]
    pub fn borrow<'borrow>(&'borrow mut self) -> BorrowedFrame<'borrow, 'scope, Self> {
        BorrowedFrame(self, PhantomData)
    }

    /// Borrow this frame as an `ExtendedTarget` with the provided `target`.
    #[inline]
    pub fn extended_target<'target, 'borrow, T>(
        &'borrow mut self,
        target: T,
    ) -> ExtendedTarget<'target, 'scope, 'borrow, T>
    where
        T: Target<'target>,
    {
        ExtendedTarget {
            target,
            frame: self,
            _target_marker: PhantomData,
        }
    }

    /// Borrow this frame as an `ExtendedTarget` with an `Output` that targets this frame.
    #[inline]
    pub fn as_extended_target<'borrow>(
        &'borrow mut self,
    ) -> ExtendedTarget<'scope, 'scope, 'borrow, Output<'scope>> {
        let target = self.output();
        ExtendedTarget {
            target,
            frame: self,
            _target_marker: PhantomData,
        }
    }

    /// Returns the number of values rooted in this frame.
    #[inline]
    pub fn n_roots(&self) -> usize {
        self.stack_size() - self.offset
    }

    /// Returns the number of values rooted in this frame.
    #[inline]
    pub fn stack_size(&self) -> usize {
        self.stack.size()
    }

    /// Returns an `Output` that targets the current frame.
    #[inline]
    pub fn output(&self) -> Output<'scope> {
        unsafe {
            let offset = self.stack.reserve_slot();
            Output {
                stack: self.stack,
                offset,
            }
        }
    }

    /// Returns a `ReusableSlot` that targets the current frame.
    #[inline]
    pub fn reusable_slot(&self) -> ReusableSlot<'scope> {
        unsafe {
            let offset = self.stack.reserve_slot();
            ReusableSlot {
                stack: self.stack,
                offset,
            }
        }
    }

    /// Returns a `Unrooted` that targets the current frame.
    #[inline]
    pub fn unrooted(&self) -> Unrooted<'scope> {
        unsafe { Unrooted::new() }
    }

    /// Create a temporary scope and call `func` with that scope's `GcFrame`.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::test::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    /// # let mut frame = StackFrame::new();
    /// # let mut julia = julia.instance(&mut frame);
    /// julia
    ///     .scope(|mut frame| {
    ///         let output = frame.output();
    ///
    ///         let _sum = frame.scope(|mut frame| {
    ///             let i = Value::new(&mut frame, 1u64);
    ///             let j = Value::new(&mut frame, 2u64);
    ///
    ///             unsafe {
    ///                 Module::base(&frame)
    ///                     .function(&mut frame, "+")?
    ///                     .call2(output, i, j)
    ///                     .into_jlrs_result()
    ///             }
    ///         })?;
    ///
    ///         Ok(())
    ///     })
    ///     .unwrap();
    /// # });
    /// # }
    /// ```

    #[inline]
    pub fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(GcFrame<'inner>) -> JlrsResult<T>,
    {
        let (owner, nested) = self.nest();
        let res = func(nested);
        std::mem::drop(owner);
        res
    }

    // Safety: ptr must be a valid pointer to T
    pub(crate) unsafe fn root<'data, T: Managed<'scope, 'data>>(
        &self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.stack.push_root(ptr.cast());
        T::wrap_non_null(ptr, Private)
    }

    pub(crate) fn stack(&self) -> &Stack {
        self.stack
    }

    pub(crate) fn nest<'nested>(&'nested mut self) -> (GcFrameOwner<'nested>, GcFrame<'nested>) {
        let owner = GcFrameOwner {
            stack: self.stack(),
            offset: self.stack.size(),
            _marker: PhantomData,
        };
        let frame = GcFrame {
            stack: self.stack(),
            offset: self.stack.size(),
            _marker: PhantomData,
        };
        (owner, frame)
    }

    // Safety: only one base frame can exist per `Stack`
    pub(crate) unsafe fn base(stack: &'scope Stack) -> (GcFrameOwner<'scope>, GcFrame<'scope>) {
        debug_assert_eq!(stack.size(), 0);
        let owner = GcFrameOwner {
            stack,
            offset: 0,
            _marker: PhantomData,
        };
        let frame = GcFrame {
            stack,
            offset: 0,
            _marker: PhantomData,
        };
        (owner, frame)
    }
}

cfg_if! {
    if #[cfg(feature = "async")] {
        use std::{future::Future, ops::{Deref, DerefMut}};

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
            /// An async version of [`GcFrame::scope`].
            ///
            /// The closure `func` must return an async block. Note that the returned value is
            /// required to live at least as long the current frame.

            #[inline]
            pub async fn async_scope<'nested, T, F, G>(&'nested mut self, func: F) -> JlrsResult<T>
            where
                T: 'scope,
                G: Future<Output = JlrsResult<T>>,
                F: FnOnce(AsyncGcFrame<'nested>) -> G,
            {
                // Safety: the lifetime of the borrow is extended, but it's valid during the call
                // to func and data returned from func must live longer.
                let (owner, nested) = self.nest_async();
                let ret = func(nested).await;
                std::mem::drop(owner);
                ret
            }

            /// `AsyncGcFrame::async_scope` with less strict lifeitme bounds on the return value.
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
                let (owner, nested) = self.nest_async();
                let ret = func(nested).await;
                std::mem::drop(owner);
                ret
            }

            // Safety: only one base frame can exist per `Stack`
            pub(crate) unsafe fn base(
                stack: &'scope Stack,
            ) -> (GcFrameOwner<'scope>, AsyncGcFrame<'scope>) {
                let owner = GcFrameOwner {
                    stack,
                    offset: 0,
                    _marker: PhantomData,
                };
                let frame = AsyncGcFrame {
                    frame: GcFrame {
                        stack,
                        offset: 0,
                        _marker: PhantomData,
                    },
                };
                (owner, frame)
            }

            pub(crate) fn nest_async<'nested>(
                &'nested mut self,
            ) -> (GcFrameOwner<'nested>, AsyncGcFrame<'nested>) {
                let (owner, frame) = self.nest();
                (
                    owner,
                    AsyncGcFrame {
                        frame: frame,
                    },
                )
            }
        }

        impl<'scope> Deref for AsyncGcFrame<'scope> {
            type Target = GcFrame<'scope>;

            fn deref(&self) -> &Self::Target {
                &self.frame
            }
        }

        impl<'scope> DerefMut for AsyncGcFrame<'scope> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.frame
            }
        }
    }
}

pub(crate) struct GcFrameOwner<'scope> {
    stack: &'scope Stack,
    offset: usize,
    _marker: PhantomData<&'scope mut &'scope ()>,
}

impl<'scope> GcFrameOwner<'scope> {
    #[cfg(feature = "ccall")]
    pub(crate) fn restore(&self) -> GcFrame<'scope> {
        GcFrame {
            stack: self.stack,
            offset: self.stack.size(),
            _marker: PhantomData,
        }
    }

    #[cfg(feature = "async")]
    pub(crate) unsafe fn reconstruct(&self, offset: usize) -> AsyncGcFrame<'scope> {
        self.stack.pop_roots(offset);
        AsyncGcFrame {
            frame: GcFrame {
                stack: self.stack,
                offset,
                _marker: PhantomData,
            },
        }
    }
}

impl Drop for GcFrameOwner<'_> {
    fn drop(&mut self) {
        unsafe { self.stack.pop_roots(self.offset) }
    }
}

/// A frame that has been borrowed. A new scope must be created before it can be used as a target
/// again.
// TODO privacy
pub struct BorrowedFrame<'borrow, 'current, F>(
    pub(crate) &'borrow mut F,
    pub(crate) PhantomData<&'current ()>,
);

impl<'borrow, 'current> BorrowedFrame<'borrow, 'current, GcFrame<'current>> {
    /// Create a temporary scope by calling [`GcFrame::scope`].

    #[inline]
    pub fn scope<T, F>(self, func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(GcFrame<'inner>) -> JlrsResult<T>,
    {
        self.0.scope(func)
    }
}

#[cfg(feature = "async")]
impl<'borrow, 'current> BorrowedFrame<'borrow, 'current, AsyncGcFrame<'current>> {
    /// Create a temporary scope by calling [`GcFrame::scope`].

    #[inline]
    pub fn scope<T, F>(self, func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(GcFrame<'inner>) -> JlrsResult<T>,
    {
        self.0.scope(func)
    }

    /// Create a temporary scope by calling [`AsyncGcFrame::async_scope`].

    #[inline]
    pub async fn async_scope<'nested, T, F, G>(self, func: F) -> JlrsResult<T>
    where
        'borrow: 'nested,
        T: 'current,
        G: Future<Output = JlrsResult<T>>,
        F: FnOnce(AsyncGcFrame<'nested>) -> G,
    {
        self.0.async_scope(func).await
    }

    /// Create a temporary scope by calling [`AsyncGcFrame::relaxed_async_scope`].
    #[inline]
    pub async unsafe fn relaxed_async_scope<'nested, T, F, G>(self, func: F) -> JlrsResult<T>
    where
        'borrow: 'nested,
        T: 'nested,
        G: Future<Output = JlrsResult<T>>,
        F: FnOnce(AsyncGcFrame<'nested>) -> G,
    {
        self.0.relaxed_async_scope(func).await
    }
}
