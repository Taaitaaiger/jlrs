//! A GC frame that can be used with async functions.

use super::mode::Async;
use crate::memory::frame::private;
use crate::{
    error::{AllocError, JlrsError, JlrsResult},
    memory::{
        frame::{GcFrame, MIN_FRAME_CAPACITY},
        mode::private::Mode,
    },
    private::Private,
};
use crate::{
    memory::{frame::Frame, output::Output, reusable_slot::ReusableSlot, stack_page::StackPage},
    prelude::Wrapper,
};
use futures::Future;
use jl_sys::jl_value_t;
use std::{
    cell::Cell,
    ffi::c_void,
    ptr::{null_mut, NonNull},
};

/// A frame is used to root Julia data, which guarantees the garbage collector doesn't free the
/// data while the frame has not been dropped. More information about this topic can be found in
/// the [`memory`] module.
///
/// An `AsyncGcFrame` offers the same functionality as a [`GcFrame`], and some additional async
/// methods that can be used to create nested async scopes. It can also be used to call the trait
/// methods of [`CallAsync`].
///
/// [`CallAsync`]: crate::multitask::call_async::CallAsync
/// [`memory`]: crate::memory
pub struct AsyncGcFrame<'frame> {
    raw_frame: &'frame mut [Cell<*mut c_void>],
    page: Option<StackPage>,
    mode: Async<'frame>,
}

impl<'frame> AsyncGcFrame<'frame> {
    /// An async version of [`Frame::scope`]. Rather than a closure, it takes an async closure
    /// that provides a new `AsyncGcFrame`.
    pub async fn async_scope<'nested, T, F, G>(&'nested mut self, func: F) -> JlrsResult<T>
    where
        T: 'frame,
        G: Future<Output = JlrsResult<T>>,
        F: FnOnce(&'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async(0);
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            func(r_nested).await
        }
    }

    /// An async version of [`Frame::scope_with_capacity`]. Rather than a closure, it takes an
    /// async closure that provides a new `AsyncGcFrame`.
    pub async fn async_scope_with_capacity<'nested, T, F, G>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T>
    where
        T: 'frame,
        G: Future<Output = JlrsResult<T>>,
        F: FnOnce(&'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async(capacity);
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            func(r_nested).await
        }
    }

    /// Returns the number of values currently rooted in this frame.
    pub fn n_roots(&self) -> usize {
        self.raw_frame[0].get() as usize >> 2
    }

    /// Returns the maximum number of slots this frame can use.
    pub fn capacity(&self) -> usize {
        self.raw_frame.len() - 2
    }

    // Safety: this frame must be dropped in the same scope it has been created and raw_frame must
    // have 2 + slots capacity available.
    pub(crate) unsafe fn new(
        raw_frame: &'frame mut [Cell<*mut c_void>],
        mode: Async<'frame>,
    ) -> Self {
        // Is popped when this frame is dropped
        mode.push_frame(raw_frame, Private);

        AsyncGcFrame {
            raw_frame,
            page: None,
            mode,
        }
    }

    // Safety: capacity >= n_slots
    pub(crate) unsafe fn set_n_roots(&mut self, n_slots: usize) {
        debug_assert!(n_slots <= self.capacity());
        self.raw_frame.get_unchecked_mut(0).set((n_slots << 2) as _);
    }

    // Safety: this frame must be dropped in the same scope it has been created.
    pub(crate) unsafe fn nest<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> GcFrame<'nested, Async<'frame>> {
        let used = self.n_roots() + 2;
        let new_frame_size = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let raw_frame = if self.page.is_some() {
            if new_frame_size <= self.page.as_ref().unwrap().size() {
                self.page.as_mut().unwrap().as_mut()
            } else {
                self.page = Some(StackPage::new(new_frame_size));
                self.page.as_mut().unwrap().as_mut()
            }
        } else if used + new_frame_size <= self.raw_frame.len() {
            &mut self.raw_frame[used..]
        } else {
            self.page = Some(StackPage::new(new_frame_size));
            self.page.as_mut().unwrap().as_mut()
        };

        GcFrame::new(raw_frame, self.mode)
    }

    // Safety: this frame must be dropped in the same scope it has been created.
    pub(crate) unsafe fn nest_async<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> AsyncGcFrame<'nested> {
        let used = self.n_roots() + 2;
        let new_frame_size = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let raw_frame = if self.page.is_some() {
            if new_frame_size <= self.page.as_ref().unwrap().size() {
                self.page.as_mut().unwrap().as_mut()
            } else {
                self.page = Some(StackPage::new(new_frame_size));
                self.page.as_mut().unwrap().as_mut()
            }
        } else if used + new_frame_size <= self.raw_frame.len() {
            &mut self.raw_frame[used..]
        } else {
            self.page = Some(StackPage::new(new_frame_size));
            self.page.as_mut().unwrap().as_mut()
        };

        AsyncGcFrame::new(raw_frame, self.mode)
    }

    // Safety: n_roots < capacity
    pub(crate) unsafe fn root(&mut self, value: NonNull<jl_value_t>) {
        debug_assert!(self.n_roots() < self.capacity());

        let n_roots = self.n_roots();
        self.raw_frame
            .get_unchecked_mut(n_roots + 2)
            .set(value.cast().as_ptr());
        self.set_n_roots(n_roots + 1);
    }
}

impl<'frame> Drop for AsyncGcFrame<'frame> {
    fn drop(&mut self) {
        // The frame was pushed when the frame was created.
        unsafe { self.mode.pop_frame(self.raw_frame, Private) }
    }
}

impl<'frame> Frame<'frame> for AsyncGcFrame<'frame> {
    fn reusable_slot(&mut self) -> JlrsResult<ReusableSlot<'frame>> {
        ReusableSlot::new(self)
    }

    fn n_roots(&self) -> usize {
        self.n_roots()
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn reserve_output(&mut self) -> JlrsResult<Output<'frame>> {
        unsafe {
            let n_roots = self.n_roots();
            if n_roots == self.capacity() {
                Err(JlrsError::alloc_error(AllocError::FrameOverflow(
                    1, n_roots,
                )))?;
            }

            let mut_slot = self.raw_frame.get_unchecked_mut(n_roots + 2);
            mut_slot.set(null_mut());
            let mut_slot = mut_slot as *mut _;
            self.set_n_roots(n_roots + 1);

            Ok(Output::new(self, mut_slot))
        }
    }
}

impl<'frame> private::Frame<'frame> for AsyncGcFrame<'frame> {
    type Mode = Async<'frame>;

    unsafe fn push_root<'data, X: Wrapper<'frame, 'data>>(
        &mut self,
        value: NonNull<X::Wraps>,
        _: Private,
    ) -> Result<X, AllocError> {
        let n_roots = self.n_roots();
        if n_roots == self.capacity() {
            return Err(AllocError::FrameOverflow(1, n_roots));
        }

        self.root(value.cast());
        Ok(X::wrap_non_null(value, Private))
    }

    unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<*const Cell<*mut c_void>> {
        let n_roots = self.n_roots();
        if n_roots == self.capacity() {
            Err(JlrsError::alloc_error(AllocError::FrameOverflow(
                1, n_roots,
            )))?;
        }

        self.raw_frame
            .get_unchecked_mut(n_roots + 2)
            .set(null_mut());
        self.set_n_roots(n_roots + 1);

        Ok(self.raw_frame.get_unchecked_mut(n_roots + 2))
    }

    unsafe fn nest<'nested>(
        &'nested mut self,
        capacity: usize,
        _: Private,
    ) -> GcFrame<'nested, Self::Mode> {
        self.nest(capacity)
    }
}
