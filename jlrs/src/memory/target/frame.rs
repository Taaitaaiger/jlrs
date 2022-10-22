//! A frame roots data until its scope ends.
//!
//! Every scope has its own frame which can hold an arbitrary number of roots. When the scope
//! ends these roots are removed from the set of roots, so all data rooted in a frame can safely
//! be used until its scope ends. This hold true even if the frame is dropped before its scope
//! ends.
//!
//! In addition to being usable as targets, frames can also be used to create [`Output`]s and
//! [`Global`]s, and child scopes with their own frame.

use std::{marker::PhantomData, ptr::NonNull};

use crate::{
    memory::{
        context::stack::Stack,
        target::{ExtendedTarget, Target},
    },
    prelude::JlrsResult,
    private::Private,
    wrappers::ptr::Wrapper,
};
use cfg_if::cfg_if;

use super::{global::Global, output::Output};

/// A frame associated with a scope.
///
/// Mutable references to a `GcFrame` can be used as a target, in this case the data will be
/// rooted until the frame's scope ends.  Other targets can be created through a frame. For
/// example, [`GcFrame::output`] creates a new `Output` that targets the current frame.
pub struct GcFrame<'scope> {
    stack: &'scope Stack,
    offset: usize,
}

impl<'scope> GcFrame<'scope> {
    /// Returns a mutable reference to this frame.
    pub fn as_mut(&mut self) -> &mut Self {
        self
    }

    /// Reserve capacity for at least `additional` roots.
    pub fn reserve(&mut self, additional: usize) {
        self.stack.reserve(additional)
    }

    /// Borrow this frame as an `ExtendedTarget` with the provided `target`.
    pub fn extended_target<'target, 'borrow, 'data, T, W>(
        &'borrow mut self,
        target: T,
    ) -> ExtendedTarget<'target, 'scope, 'borrow, 'data, T, W>
    where
        T: Target<'target, 'data, W>,
        W: Wrapper<'target, 'data>,
    {
        ExtendedTarget {
            target,
            frame: self,
            _data_marker: PhantomData,
            _target_marker: PhantomData,
            _wrapper_marker: PhantomData,
        }
    }

    /// Borrow this frame as an `ExtendedTarget` with an `Output` that targets this frame.
    pub fn as_extended_target<'borrow, 'data, W>(
        &'borrow mut self,
    ) -> ExtendedTarget<'scope, 'scope, 'borrow, 'data, Output<'scope>, W>
    where
        W: Wrapper<'scope, 'data>,
    {
        let target = self.output();
        ExtendedTarget {
            target,
            frame: self,
            _data_marker: PhantomData,
            _target_marker: PhantomData,
            _wrapper_marker: PhantomData,
        }
    }

    /// Returns the number of values rooted in this frame.
    pub fn n_roots(&self) -> usize {
        self.stack.size() - self.offset
    }

    /// Returns an `Output` that targets the current frame.
    pub fn output(&self) -> Output<'scope> {
        unsafe {
            let offset = self.stack.reserve_slot();
            Output {
                stack: self.stack,
                offset,
            }
        }
    }

    /// Returns a `Global` that targets the current frame.
    pub fn global(&self) -> Global<'scope> {
        unsafe { Global::new() }
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
    /// # let julia = julia.instance(&mut frame);
    ///   julia.scope(|mut frame| {
    ///       let output = frame.output();
    ///
    ///       let _sum = frame.scope(|mut frame| {
    ///           let i = Value::new(&mut frame, 1u64);
    ///           let j = Value::new(&mut frame, 2u64);
    ///
    ///           Module::base(&frame)
    ///               .function(&mut frame, "+")?
    ///               .call2(output, i, j)
    ///               .into_jlrs_result()
    ///       })?;
    ///
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
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
    pub(crate) unsafe fn root<'data, T: Wrapper<'scope, 'data>>(
        &self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.stack.push_root(ptr.cast());
        T::wrap_non_null(ptr, Private)
    }

    pub(crate) fn stack<'nested>(&'nested self) -> &'nested Stack {
        self.stack
    }

    pub(crate) fn nest<'nested>(&'nested mut self) -> (GcFrameOwner<'nested>, GcFrame<'nested>) {
        let owner = GcFrameOwner {
            stack: self.stack(),
            offset: self.stack.size(),
        };
        let frame = GcFrame {
            stack: self.stack(),
            offset: self.stack.size(),
        };
        (owner, frame)
    }

    // Safety: only one base frame can exist per `Stack`
    pub(crate) unsafe fn base<'base>(stack: &'base Stack) -> (GcFrameOwner<'base>, GcFrame<'base>) {
        debug_assert_eq!(stack.size(), 0);
        let owner = GcFrameOwner { stack, offset: 0 };
        let frame = GcFrame { stack, offset: 0 };
        (owner, frame)
    }
}

cfg_if! {
    if #[cfg(feature = "async")] {
        use std::{future::Future, ops::{Deref, DerefMut}};

        /// A frame associated with an async scope.
        ///
        /// The only difference between a `GcFrame` and an `AsyncGcFrame` is that the latter
        /// allows calling several async methods, most importantly those of the [`CallAsync`]. An
        /// `AsyncGcFrame` can be (mutably) dereferenced as a `GcFrame`, so all methods of `GcFrame`
        /// are available to `AsyncGcFrame`.
        pub struct AsyncGcFrame<'scope> {
            scope_context: GcFrame<'scope>,
        }

        impl<'scope> AsyncGcFrame<'scope> {
            /// An async version of [`GcFrame::scope`].
            ///
            /// The closure `func` must return an async block. Note that the returned value is
            /// required to live at least as long the current frame.
            #[inline(never)]
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
            #[inline(never)]
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
            pub(crate) unsafe fn base<'base>(
                stack: &'base Stack,
            ) -> (GcFrameOwner<'base>, AsyncGcFrame<'base>) {
                let owner = GcFrameOwner {
                    stack,
                    offset: 0,
                };
                let frame = AsyncGcFrame {
                    scope_context: GcFrame {
                        stack,
                        offset: 0,
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
                        scope_context: frame,
                    },
                )
            }
        }

        impl<'scope> Deref for AsyncGcFrame<'scope> {
            type Target = GcFrame<'scope>;

            fn deref(&self) -> &Self::Target {
                &self.scope_context
            }
        }

        impl<'scope> DerefMut for AsyncGcFrame<'scope> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.scope_context
            }
        }
    }
}

pub(crate) struct GcFrameOwner<'scope> {
    stack: &'scope Stack,
    offset: usize,
}

impl<'scope> GcFrameOwner<'scope> {
    #[cfg(feature = "async")]
    pub(crate) unsafe fn reconstruct(&self, offset: usize) -> AsyncGcFrame<'scope> {
        self.stack.pop_roots(offset);
        AsyncGcFrame {
            scope_context: GcFrame {
                stack: self.stack,
                offset,
            },
        }
    }
}

impl Drop for GcFrameOwner<'_> {
    fn drop(&mut self) {
        unsafe { self.stack.pop_roots(self.offset) }
    }
}
