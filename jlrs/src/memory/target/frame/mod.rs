//! Dynamically and statically-sized frames.
//!
//! Every scope has its own frame which can hold some number of roots. When the scope ends these
//! roots are removed from the set of roots, so all data rooted in a frame can safely be used
//! until its scope ends. This hold true even if the frame is dropped before its scope ends.
//!
//! For more information see the documentation in the [`memory`] and [`target`] modules.
//!
//! [`memory`]: crate::memory
//! [`target`]: crate::memory::target

#[cfg(feature = "async")]
pub mod async_frame;

use std::{marker::PhantomData, pin::Pin, ptr::NonNull};

use jl_sys::{RawGcFrame, UnsizedGcFrame, pop_frame};

#[cfg(feature = "async")]
pub use self::async_frame::*;
use super::{
    ExtendedTarget, Target,
    output::Output,
    reusable_slot::ReusableSlot,
    slot_ref::{LocalSlotRef, StackSlotRef},
    unrooted::Unrooted,
};
use crate::{
    data::managed::Managed,
    memory::{context::stack::Stack, scope::private::LocalScopePriv},
    prelude::{LocalScope, Scope},
    private::Private,
};

/// A dynamically-sized frame that can hold an arbitrary number of roots.
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
    #[inline]
    pub fn borrow<'borrow>(&'borrow mut self) -> BorrowedFrame<'borrow, 'scope, Self> {
        BorrowedFrame(self, PhantomData)
    }

    /// Borrow this frame as an `ExtendedTarget` with the provided `target`.
    #[inline]
    pub fn extended_target<'target, 'borrow, Tgt>(
        &'borrow mut self,
        target: Tgt,
    ) -> ExtendedTarget<'target, 'scope, 'borrow, Tgt>
    where
        Tgt: Target<'target>,
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
    ) -> ExtendedTarget<'scope, 'scope, 'borrow, Output<'scope, StackSlotRef<'scope>>> {
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
    pub fn output(&mut self) -> Output<'scope, StackSlotRef<'scope>> {
        unsafe {
            let offset = self.stack.reserve_slot();
            Output::new(StackSlotRef::new(self.stack, offset))
        }
    }

    /// Returns a `ReusableSlot` that targets the current frame.
    #[inline]
    pub fn reusable_slot(&mut self) -> ReusableSlot<'scope, StackSlotRef<'scope>> {
        unsafe {
            let offset = self.stack.reserve_slot();
            let slot = StackSlotRef::new(self.stack, offset);
            ReusableSlot::new(slot)
        }
    }

    /// Returns an `Unrooted` that targets the current frame.
    #[inline]
    pub const fn unrooted(&self) -> Unrooted<'scope> {
        unsafe { Unrooted::new() }
    }

    // Safety: ptr must be a valid pointer to T
    #[inline]
    #[track_caller]
    pub(crate) unsafe fn root<'data, T: Managed<'scope, 'data>>(
        &self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        unsafe {
            self.stack.push_root(ptr.cast());
            T::wrap_non_null(ptr, Private)
        }
    }

    #[inline]
    pub(crate) fn stack(&self) -> &Stack {
        self.stack
    }

    #[inline]
    pub(crate) unsafe fn nest<'nested>(&'nested mut self) -> (usize, GcFrame<'nested>) {
        let frame = GcFrame {
            stack: self.stack(),
            offset: self.stack.size(),
            _marker: PhantomData,
        };
        (self.stack.size(), frame)
    }

    // Safety: only one base frame can exist per `Stack`
    #[inline]
    pub(crate) unsafe fn base(stack: &'scope Stack) -> GcFrame<'scope> {
        debug_assert_eq!(stack.size(), 0);
        GcFrame {
            stack,
            offset: 0,
            _marker: PhantomData,
        }
    }

    // pub fn stack_addr(&self) -> *const c_void {
    //     self.stack as *const _ as *const _
    // }
}

unsafe impl<'f> Scope for GcFrame<'f> {
    #[inline]
    fn scope<T>(&mut self, func: impl for<'scope> FnOnce(GcFrame<'scope>) -> T) -> T {
        unsafe {
            let (offset, nested) = self.nest();
            let res = func(nested);
            self.stack.pop_roots(offset);
            res
        }
    }
}

/// A statically-sized frame that can hold `N` roots.
pub struct LocalGcFrame<'scope, const N: usize> {
    frame: &'scope PinnedLocalFrame<'scope, N>,
    offset: usize,
}

impl<'scope, const N: usize> LocalGcFrame<'scope, N> {
    /// Returns a mutable reference to this frame.
    #[inline]
    pub fn as_mut(&mut self) -> &mut Self {
        self
    }

    /// Returns the number of values rooted in this frame.
    #[inline]
    pub fn n_roots(&self) -> usize {
        self.offset
    }

    /// Returns the number of values that can be rooted in this frame.
    #[inline]
    pub const fn frame_size(&self) -> usize {
        N
    }

    /// Returns an `Output` that targets the current frame.
    #[inline]
    pub fn output(&mut self) -> Output<'scope, LocalSlotRef<'scope>> {
        unsafe {
            let slot = self.frame.frame.raw.get_root(self.offset);
            self.offset += 1;
            Output::new(LocalSlotRef::new(slot))
        }
    }

    /// Returns a `ReusableSlot` that targets the current frame.
    #[inline]
    pub fn reusable_slot(&mut self) -> ReusableSlot<'scope, LocalSlotRef<'scope>> {
        unsafe {
            let slot = self.frame.frame.raw.get_root(self.offset);
            let slot = LocalSlotRef::new(slot);
            self.offset += 1;
            ReusableSlot::new(slot)
        }
    }

    /// Returns a `Unrooted` that targets the current frame.
    #[inline]
    pub const fn unrooted(&self) -> Unrooted<'scope> {
        unsafe { Unrooted::new() }
    }

    #[inline]
    pub(crate) unsafe fn new(frame: &'scope PinnedLocalFrame<'scope, N>) -> Self {
        LocalGcFrame { frame, offset: 0 }
    }

    #[inline]
    #[track_caller]
    pub(crate) unsafe fn root<'data, T: Managed<'scope, 'data>>(
        &mut self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        unsafe {
            self.frame
                .frame
                .raw
                .set_root(self.offset, ptr.as_ptr().cast());
            self.offset += 1;
            T::wrap_non_null(ptr, Private)
        }
    }
}

pub struct UnsizedLocalGcFrame<'scope> {
    frame: UnsizedGcFrame<'scope>,
    offset: usize,
}

impl<'scope> UnsizedLocalGcFrame<'scope> {
    pub(crate) fn new(frame: UnsizedGcFrame<'scope>) -> Self {
        UnsizedLocalGcFrame { frame, offset: 0 }
    }

    /// Returns a mutable reference to this frame.
    #[inline]
    pub fn as_mut(&mut self) -> &mut Self {
        self
    }

    /// Returns the number of values rooted in this frame.
    #[inline]
    pub fn n_roots(&self) -> usize {
        self.offset
    }

    /// Returns the number of values that can be rooted in this frame.
    #[inline]
    pub fn frame_size(&self) -> usize {
        self.frame.size()
    }

    /// Returns an `Output` that targets the current frame.
    #[inline]
    pub fn output(&mut self) -> Output<'scope, LocalSlotRef<'scope>> {
        unsafe {
            let slot = self.frame.get_root(self.offset);
            self.offset += 1;
            Output::new(LocalSlotRef::new(slot))
        }
    }

    /// Returns a `ReusableSlot` that targets the current frame.
    #[inline]
    pub fn reusable_slot(&mut self) -> ReusableSlot<'scope, LocalSlotRef<'scope>> {
        unsafe {
            let slot = self.frame.get_root(self.offset);
            let slot = LocalSlotRef::new(slot);
            self.offset += 1;
            ReusableSlot::new(slot)
        }
    }

    /// Returns a `Unrooted` that targets the current frame.
    #[inline]
    pub const fn unrooted(&self) -> Unrooted<'scope> {
        unsafe { Unrooted::new() }
    }

    #[inline]
    #[track_caller]
    pub(crate) unsafe fn root<'data, T: Managed<'scope, 'data>>(
        &mut self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        unsafe {
            self.frame.get_root(self.offset).set(ptr.as_ptr().cast());
            self.offset += 1;
            T::wrap_non_null(ptr, Private)
        }
    }
}

/// A frame that has been borrowed. A new scope must be created before it can be used as a target
/// again.
// TODO privacy
pub struct BorrowedFrame<'borrow, 'current, F>(
    pub(crate) &'borrow mut F,
    pub(crate) PhantomData<&'current ()>,
);

impl<'borrow, 'current> LocalScopePriv for BorrowedFrame<'borrow, 'current, GcFrame<'current>> {}

unsafe impl<'borrow, 'current> LocalScope for BorrowedFrame<'borrow, 'current, GcFrame<'current>> {}

unsafe impl<'borrow, 'current> Scope for BorrowedFrame<'borrow, 'current, GcFrame<'current>> {
    #[inline]
    fn scope<T>(&mut self, func: impl for<'scope> FnOnce(GcFrame<'scope>) -> T) -> T {
        self.0.scope(func)
    }
}

#[cfg(feature = "async")]
impl<'borrow, 'current> LocalScopePriv
    for BorrowedFrame<'borrow, 'current, AsyncGcFrame<'current>>
{
}

#[cfg(feature = "async")]
unsafe impl<'borrow, 'current> LocalScope
    for BorrowedFrame<'borrow, 'current, AsyncGcFrame<'current>>
{
}

#[cfg(feature = "async")]
unsafe impl<'borrow, 'current> Scope for BorrowedFrame<'borrow, 'current, AsyncGcFrame<'current>> {
    #[inline]
    fn scope<T>(&mut self, func: impl for<'scope> FnOnce(GcFrame<'scope>) -> T) -> T {
        self.0.scope(func)
    }
}

#[cfg(feature = "async")]
unsafe impl<'borrow, 'current> crate::prelude::AsyncScope
    for BorrowedFrame<'borrow, 'current, AsyncGcFrame<'current>>
{
    #[inline]
    async fn async_scope<T>(
        &mut self,
        func: impl for<'scope> AsyncFnOnce(AsyncGcFrame<'scope>) -> T,
    ) -> T {
        self.0.async_scope(func).await
    }
}

#[repr(C)]
pub(crate) struct LocalFrame<const N: usize> {
    raw: RawGcFrame<N>,
}

impl<const N: usize> LocalFrame<N> {
    #[inline]
    pub(crate) const fn new() -> Self {
        unsafe {
            LocalFrame {
                raw: RawGcFrame::new(),
            }
        }
    }

    #[inline]
    pub(crate) unsafe fn pin<'scope>(&'scope mut self) -> PinnedLocalFrame<'scope, N> {
        unsafe { PinnedLocalFrame::new(self) }
    }
}

pub(crate) struct PinnedLocalFrame<'scope, const N: usize> {
    frame: Pin<&'scope mut LocalFrame<N>>,
    _marker: PhantomData<&'scope mut &'scope ()>,
}

impl<'scope, const N: usize> PinnedLocalFrame<'scope, N> {
    #[inline]
    unsafe fn new(frame: &'scope mut LocalFrame<N>) -> Self {
        unsafe {
            if N > 0 {
                frame.raw.push_frame();
            }

            PinnedLocalFrame {
                frame: Pin::new_unchecked(frame),
                _marker: PhantomData,
            }
        }
    }

    #[inline]
    pub(crate) unsafe fn pop(&self) {
        unsafe {
            if N > 0 {
                pop_frame()
            }
        }
    }
}
