//! Frames protect values from garbage collection.
//!
//! Several kinds of frame exist in jlrs. The simplest one is [`NullFrame`], which is only used
//! when writing `ccall`able functions. It doesn't let you root any values or create a nested
//! scope, but can be used to (mutably) borrow array data. If you neither use the async runtime
//! nor write Rust functions that Julia will call, the only frame type you will use is
//! [`GcFrame`]; this frame can be used to root a relatively arbitrary number of values, and new
//! frames can always be pushed on top of it.
//!
//! A [`GcFrame`] can preallocate a number of slots, each slot can root one value. By
//! preallocating the slots less work has to be done to root a value, more slots are allocated if
//! this is necessary. The maximum number of slots that can be allocated is the frame's capacity.
//! The capacity of a frame is at least 16. When a new frame is pushed to the GC stack, the
//! current frame's remaining capacity will be used to store this new frame. If the remaining
//! capacity is insufficient, more stack space is allocated.
//!
//! Frames are pushed to the GC stack when they're created, and popped when they're dropped. It's
//! not possible to create a frame directly, rather the methods [`ScopeExt::scope`],
//! [`Scope::value_scope`], and [`Scope::result_scope`] all take a closure which provides you with a
//! mutable reference to a new frame.
//!
//! [`Scope::value_scope`]: crate::memory::traits::scope::Scope::value_scope
//! [`Scope::result_scope`]: crate::memory::traits::scope::Scope::result_scope
//! [`ScopeExt::scope`]: crate::memory::traits::scope::ScopeExt::scope

use super::{stack::StackPage, traits::mode::Mode};
use crate::{private::Private, CCall};
use jl_sys::jl_value_t;
use std::{
    ffi::c_void,
    marker::PhantomData,
    ptr::{null_mut, NonNull},
};

pub(crate) const MIN_FRAME_CAPACITY: usize = 16;

/// A frame that can be used to root values.
///
/// Roots are stored in slots, each slot can contain one root. Frames created with slots will
/// preallocate that number of slots. Frames created without slots will dynamically create new
/// slots as needed. If a frame is created without slots it is able to create at least 16 slots.
///
/// If there is sufficient capacity available, a new frame will use this remaining capacity. If
/// the capacity is insufficient, more stack space is allocated.
///
/// [`Julia::scope`]: crate::Julia::scope
/// [`ScopeExt::scope`]: crate::memory::traits::scope::ScopeExt::scope
/// [`Scope::value_scope`]: crate::memory::traits::scope::Scope::value_scope
/// [`Scope::result_scope`]: crate::memory::traits::scope::Scope::result_scope
pub struct GcFrame<'frame, M: Mode> {
    raw_frame: &'frame mut [*mut c_void],
    page: Option<StackPage>,
    n_roots: usize,
    mode: M,
}

impl<'frame, M: Mode> GcFrame<'frame, M> {
    /// Returns the number of values currently rooted in this frame.
    pub fn n_roots(&self) -> usize {
        self.n_roots
    }

    /// Returns the number of slots that are currently allocated to this frame.
    pub fn n_slots(&self) -> usize {
        self.raw_frame[0] as usize >> 1
    }

    /// Returns the maximum number of slots this frame can use.
    pub fn capacity(&self) -> usize {
        self.raw_frame.len() - 2
    }

    /// Try to allocate `additional` slots in the current frame. Returns `true` on success, or
    /// `false` if `self.n_slots() + additional > self.capacity()`.
    #[must_use]
    pub fn alloc_slots(&mut self, additional: usize) -> bool {
        let slots = self.n_slots();
        if additional + slots > self.capacity() {
            return false;
        }

        for idx in slots + 2..slots + additional + 2 {
            self.raw_frame[idx] = null_mut();
        }

        // The new number of slots does not exceed the capacity, and the new slots have been cleared
        unsafe { self.set_n_slots(slots + additional) }
        true
    }

    // Safety: this frame must be dropped in the same scope it has been created.
    pub(crate) unsafe fn nest<'nested>(&'nested mut self, capacity: usize) -> GcFrame<'nested, M> {
        let used = self.n_slots() + 2;
        let new_frame_size = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let raw_frame = if used + new_frame_size > self.raw_frame.len() {
            if self.page.is_none() || self.page.as_ref().unwrap().size() < new_frame_size {
                self.page = Some(StackPage::new(new_frame_size));
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[used..]
        };

        GcFrame::new(raw_frame, capacity, self.mode)
    }

    // Safety: this frame must be dropped in the same scope it has been created.
    pub(crate) unsafe fn new(raw_frame: &'frame mut [*mut c_void], slots: usize, mode: M) -> Self {
        mode.push_frame(raw_frame, slots, Private);

        GcFrame {
            raw_frame,
            page: None,
            n_roots: 0,
            mode,
        }
    }

    // Safety: capacity >= n_slots
    pub(crate) unsafe fn set_n_slots(&mut self, n_slots: usize) {
        debug_assert!(self.capacity() >= n_slots);
        self.raw_frame[0] = (n_slots << 1) as _;
    }

    // Safety: capacity > n_roots
    pub(crate) unsafe fn root(&mut self, value: NonNull<jl_value_t>) {
        debug_assert!(self.n_roots() < self.capacity());

        let n_roots = self.n_roots();
        self.raw_frame[n_roots + 2] = value.cast().as_ptr();
        if n_roots == self.n_slots() {
            self.set_n_slots(n_roots + 1);
        }
    }
}

impl<'frame, M: Mode> Drop for GcFrame<'frame, M> {
    fn drop(&mut self) {
        // The frame was pushed when the frame was created.
        unsafe { self.mode.pop_frame(self.raw_frame, Private) }
    }
}

/// A `NullFrame` can be used if you call Rust from Julia through `ccall` and want to borrow array
/// data but not perform any allocations. It can't be used to created a nested scope or for functions
/// that allocate (like creating new values or calling functions). Functions that depend on
/// allocation will return `JlrsError::NullFrame` if you call them with a `NullFrame`.
pub struct NullFrame<'frame>(PhantomData<&'frame ()>);

impl<'frame> NullFrame<'frame> {
    pub(crate) unsafe fn new(_: &'frame mut CCall) -> Self {
        NullFrame(PhantomData)
    }
}
