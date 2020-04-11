// In order to prevent the GC from freeing things that are in use, the Julia C API offers a few
// macros that should be used. These macros allocate some space on the stack with alloca and use
// it to construct a struct of type jl_gcframe_t. The first two fields contain the number of
// protected values (times two) and a pointer to the previous frame, then pointers to the values
// that should not be freed.
//
// Rust doesn't really like dynamically sized types, and as far as I'm aware something like alloca
// is unavailable. As a workaround jlrs creates a boxed array to contain these frames. A new frame
// is pushed when a frame is created, and is popped when the frame is dropped.
//
// Compared to the possibilities of the macros, jlrs is a bit more flexible. For example, the
// DynamicFrame dynamically grows its associated GC frame which is not possible in the C API and
// Outputs allow you to protect the result of a function call until the output's frame is dropped.
// My driving assumption is that when the GC runs, it can't make any assumptions about the
// contents of the GC stack based on earlier runs. Dynamically growing the frame does not make
// sense in C because alloca is used but there's no technical reason preventing such a feature
// from existing in Rust. Similarly, thanks to lifetimes we can enforce that a value can't live
// longer than its frame while C can offer no such guarantees.

use crate::error::{JlrsError, JlrsResult};
use crate::frame::{Scope, Output};
use crate::value::{Value, Values};
use jl_sys::jl_get_ptls_states;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::null_mut;

pub(crate) enum Static {}
pub(crate) enum Dynamic {}

#[derive(Copy, Clone, Default)]
pub(crate) struct FrameIdx(usize);

pub(crate) struct RawStack(Box<[*mut c_void]>);

impl RawStack {
    pub(crate) unsafe fn new(stack_size: usize) -> Self {
        let mut raw = vec![null_mut(); stack_size];
        raw[0] = 1 as _;
        let boxed = raw.into_boxed_slice();
        RawStack(boxed)
    }

    pub(crate) fn size(&self) -> usize {
        self.0.len()
    }
}

impl AsMut<[*mut c_void]> for RawStack {
    fn as_mut(&mut self) -> &mut [*mut c_void] {
        &mut self.0
    }
}

pub(crate) struct StackView<'stack, V> {
    stack: &'stack mut [*mut c_void],
    _marker: PhantomData<V>,
}

impl<'stack, V> StackView<'stack, V> {
    pub(crate) fn size(&self) -> usize {
        self.stack[0] as _
    }

    pub(crate) fn print_memory(&self) {
        println!("{:?}", &self.stack);
    }

    pub(crate) unsafe fn pop_frame(&mut self, idx: FrameIdx) {
        let rtls = &mut *jl_get_ptls_states();
        rtls.pgcstack = (&*rtls.pgcstack).prev;
        self.stack[0] = (idx.0 - 2) as _;
    }

    pub(crate) unsafe fn nest_static<'nested>(&'nested mut self, _: &'nested mut Scope) -> StackView<'nested, Static> {
        StackView {
            stack: self.stack,
            _marker: PhantomData,
        }
    }

    pub(crate) unsafe fn nest_dynamic<'nested>(&'nested mut self, _: &'nested mut Scope) -> StackView<'nested, Dynamic> {
        StackView {
            stack: self.stack,
            _marker: PhantomData,
        }
    }

    pub(crate) unsafe fn as_values<'output>(
        &mut self,
        idx: FrameIdx,
        offset: usize,
        n: usize,
    ) -> Values<'output> {
        let ptr = self.stack[idx.0 + offset..].as_mut_ptr();
        Values::wrap(ptr as _, n)
    }
}

impl<'stack> StackView<'stack, Dynamic> {
    pub(crate) unsafe fn new(stack: &'stack mut [*mut c_void]) -> Self {
        StackView {
            stack,
            _marker: PhantomData,
        }
    }

    pub(crate) unsafe fn new_frame(&mut self) -> JlrsResult<FrameIdx> {
        if self.size() + 2 >= self.stack.len() {
            return Err(JlrsError::StackSizeExceeded.into());
        }

        let rtls = &mut *jl_get_ptls_states();
        self.stack[self.size()] = 0 as _;
        self.stack[self.size() + 1] = rtls.pgcstack as _;

        rtls.pgcstack = self.stack[self.size() + 1..].as_ptr() as _;
        let idx = FrameIdx(self.size() + 2);
        self.stack[0] = (self.size() + 2) as _;

        Ok(idx)
    }

    pub(crate) unsafe fn new_output<'output>(
        &mut self,
        idx: FrameIdx,
    ) -> JlrsResult<Output<'output>> {
        if self.size() >= self.stack.len() {
            return Err(JlrsError::StackSizeExceeded.into());
        }

        let sz = self.size();
        self.stack[sz] = null_mut();
        self.stack[idx.0 - 2] = (self.stack[idx.0 - 2] as usize + 2) as _;
        self.stack[0] = (self.size() + 1) as _;
        Ok(Output::new(sz))
    }

    pub(crate) unsafe fn protect<'output>(
        &mut self,
        idx: FrameIdx,
        value: *mut c_void,
    ) -> JlrsResult<Value<'output, 'static>> {
        if self.size() == self.stack.len() {
            return Err(JlrsError::StackSizeExceeded.into());
        }

        self.stack[self.size()] = value.cast::<_>();
        self.stack[idx.0 - 2] = (self.stack[idx.0 - 2] as usize + 2) as _;
        self.stack[0] = (self.size() + 1) as _;
        Ok(Value::wrap(value.cast::<_>()))
    }

    pub(crate) unsafe fn protect_output<'output>(
        &mut self,
        output: Output,
        value: *mut c_void,
    ) -> Value<'output, 'static> {
        self.stack[output.offset] = value.cast::<_>();
        Value::wrap(value.cast::<_>())
    }
}

impl<'stack> StackView<'stack, Static> {
    pub(crate) unsafe fn new(stack: &'stack mut [*mut c_void]) -> Self {
        StackView {
            stack,
            _marker: PhantomData,
        }
    }

    pub(crate) unsafe fn new_frame(&mut self, capacity: usize) -> JlrsResult<FrameIdx> {
        if self.size() + capacity + 2 >= self.stack.len() {
            return Err(JlrsError::StackSizeExceeded.into());
        }

        let rtls = &mut *jl_get_ptls_states();
        self.stack[self.size()] = (capacity << 1) as _;
        self.stack[self.size() + 1] = rtls.pgcstack as _;

        for i in 0..capacity {
            self.stack[self.size() + 2 + i] = null_mut();
        }

        rtls.pgcstack = self.stack[self.size()..].as_ptr() as _;
        let idx = FrameIdx(self.size() + 2);
        self.stack[0] = (self.size() + capacity + 2) as _;

        Ok(idx)
    }

    pub(crate) unsafe fn new_output<'output>(
        &mut self,
        idx: FrameIdx,
        offset: usize,
    ) -> Output<'output> {
        Output::new(idx.0 + offset)
    }

    pub(crate) unsafe fn protect<'output>(
        &mut self,
        idx: FrameIdx,
        offset: usize,
        value: *mut c_void,
    ) -> Value<'output, 'static> {
        self.stack[idx.0 + offset] = value;
        Value::wrap(value as _)
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_stack() {
        let stack = Stack::new(32);
        assert_eq!(stack.free_slots(), 30);
        assert_eq!(stack.current_offset(), 2);
        assert_eq!(stack.top, 0);
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
            assert_eq!(stack.top, 1);
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
            assert_eq!(stack.top, 0);
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
            assert_eq!(stack.top, 4);
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
            assert_eq!(stack.top, 8);
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
            assert_eq!(stack.top, 4);

            stack.pop_frame_no_gc();
            assert_eq!(stack.free_slots(), 30);
            assert_eq!(stack.current_offset(), 2);
            assert_eq!(stack.top, 0);
        }
    }
}

*/
