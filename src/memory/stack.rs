/*
Julia is a garbage-collected programming language, and its runtime is by
default unaware of any references existing on our side of the fence. In order
to ensure the data we're working with survives the GC doing its job, we need
to make it aware of the fact that we need this stuff.

The C API offers several macros that take care of this, but they depend on
allocating dynamically-sized data on the stack which is not possible to do in
Rust as far as I'm aware. Fortunately, what these macros do can also be
accomplished by allocating some space on the heap and just giving Julia what
it expects to see.

I've tested if this works by explicitly triggering the GC at several points
and it seems to be fine...
*/

use crate::error::{JlrsError, JlrsResult};
use jl_sys::jl_get_ptls_states;
use std::ffi::c_void;
use std::ptr::null_mut;

pub(crate) struct Stack {
    inner: Box<[*mut c_void]>,
    len: usize,
}

impl Stack {
    pub(crate) fn new(raw_size: usize) -> Self {
        Stack {
            inner: vec![null_mut(); raw_size].into(),
            len: 0,
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.inner.len()
    }

    pub(crate) fn current_offset(&self) -> usize {
        self.len + 2
    }

    pub(crate) fn free_slots(&self) -> usize {
        self.inner.len().saturating_sub(self.current_offset())
    }

    pub(crate) unsafe fn set_value(&mut self, index: usize, value: *mut c_void) {
        debug_assert!(index < self.len - 1);
        debug_assert!(self.inner[index] == null_mut());
        self.inner[index] = value;
    }

    pub(crate) unsafe fn get_value(&self, index: usize) -> *mut c_void {
        debug_assert!(index + 1 < self.len);
        self.inner[index]
    }

    pub(crate) unsafe fn get_values(&self, index: usize, n: usize) -> *mut *mut c_void {
        debug_assert!(index + n + 1 < self.len);
        &self.inner[index] as *const *mut c_void as _
    }

    pub(crate) unsafe fn push_frame(&mut self, slots: usize) -> JlrsResult<usize> {
        let pending = self.prepare_new_frame(slots)?;

        // Don't push a frame if nothing new is allocated
        if slots > 0 {
            let rtls = &mut *jl_get_ptls_states();
            self.set_previous_frame(rtls.pgcstack as _);
            rtls.pgcstack = self.inner[self.len..].as_ptr() as _;
        }

        self.len += pending;
        Ok(self.len + 2)
    }

    // For unit testing
    #[cfg(test)]
    unsafe fn push_frame_no_gc(&mut self, slots: usize) -> JlrsResult<usize> {
        let pending = self.prepare_new_frame(slots)?;

        self.len += pending;
        Ok(self.len + 2)
    }

    pub(crate) unsafe fn pop_frame(&mut self) {
        debug_assert!(self.len != 0);

        // If nothing was allocated, only a single 1 rather than a full frame was written
        if self.inner[self.len - 1] as usize != 1 {
            let rtls = &mut *jl_get_ptls_states();
            rtls.pgcstack = (&*rtls.pgcstack).prev;
        }

        self.rewind_to_previous_frame();
    }

    // For unit testing
    #[cfg(test)]
    unsafe fn pop_frame_no_gc(&mut self) {
        debug_assert!(self.len != 0);
        self.rewind_to_previous_frame();
    }

    pub(crate) unsafe fn pop_all(&mut self) {
        while self.len > 0 {
            self.pop_frame()
        }
    }

    unsafe fn set_previous_frame(&mut self, prev: *mut c_void) {
        self.inner[self.len + 1] = prev;
    }

    unsafe fn prepare_new_frame(&mut self, slots: usize) -> JlrsResult<usize> {
        if slots == 0 {
            if self.len == self.inner.len() {
                return Err(JlrsError::StackSizeExceeded.into());
            }

            // Only write a single 1
            self.inner[self.len] = 1 as _;
            Ok(1)
        } else {
            if self.len + slots + 3 > self.inner.len() {
                return Err(JlrsError::StackSizeExceeded.into());
            }
            self.inner[self.len] = (slots << 1) as _;
            for i in self.len + 1..self.len + 2 + slots {
                self.inner[i] = null_mut();
            }
            let pending = slots + 3;
            self.inner[self.len + 2 + slots] = pending as _;
            Ok(pending)
        }
    }

    unsafe fn rewind_to_previous_frame(&mut self) {
        self.len -= self.inner[self.len - 1] as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_stack() {
        let stack = Stack::new(32);
        assert_eq!(stack.free_slots(), 30);
        assert_eq!(stack.current_offset(), 2);
        assert_eq!(stack.len, 0);
    }

    #[test]
    fn push_empty_frame() {
        unsafe {
            let mut stack = Stack::new(32);
            let offset = stack.push_frame_no_gc(0).unwrap();
            assert_eq!(offset, 3);
            assert_eq!(stack.inner[0] as usize, 1);
            assert_eq!(stack.free_slots(), 29);
            assert_eq!(stack.current_offset(), 3);
            assert_eq!(stack.len, 1);
        }
    }

    #[test]
    fn pop_empty_frame() {
        unsafe {
            let mut stack = Stack::new(32);
            stack.push_frame_no_gc(0).unwrap();
            stack.pop_frame();
            assert_eq!(stack.free_slots(), 30);
            assert_eq!(stack.current_offset(), 2);
            assert_eq!(stack.len, 0);
        }
    }

    #[test]
    fn push_nonempty_frame() {
        unsafe {
            let mut stack = Stack::new(32);
            let offset = stack.push_frame_no_gc(1).unwrap();
            assert_eq!(offset, 6);
            assert_eq!(stack.inner[0] as usize, 2);
            assert_eq!(stack.inner[3] as usize, 4);
            assert_eq!(stack.free_slots(), 26);
            assert_eq!(stack.current_offset(), 6);
            assert_eq!(stack.len, 4);
        }
    }

    #[test]
    fn push_two_frames() {
        unsafe {
            let mut stack = Stack::new(32);
            stack.push_frame_no_gc(1).unwrap();
            let offset = stack.push_frame_no_gc(1).unwrap();
            assert_eq!(offset, 10);
            assert_eq!(stack.inner[4] as usize, 2);
            assert_eq!(stack.inner[7] as usize, 4);
            assert_eq!(stack.free_slots(), 22);
            assert_eq!(stack.current_offset(), 10);
            assert_eq!(stack.len, 8);
        }
    }

    #[test]
    fn pop_two_frames() {
        unsafe {
            let mut stack = Stack::new(32);
            stack.push_frame_no_gc(1).unwrap();
            stack.push_frame_no_gc(1).unwrap();

            stack.pop_frame_no_gc();
            assert_eq!(stack.free_slots(), 26);
            assert_eq!(stack.current_offset(), 6);
            assert_eq!(stack.len, 4);

            stack.pop_frame_no_gc();
            assert_eq!(stack.free_slots(), 30);
            assert_eq!(stack.current_offset(), 2);
            assert_eq!(stack.len, 0);
        }
    }
}
