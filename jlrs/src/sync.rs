use crate::frame::FrameIdx;
use jl_sys::jl_get_ptls_states;
use std::ffi::c_void;
use std::ptr::null_mut;

pub enum Sync {}

#[cfg(feature = "async")]
pub enum Async {}

/// This trait is used to allow pushing and popping GC frames to behave differently in the two
/// available modes. You must never use its methods directly.
pub unsafe trait Mode {
    unsafe fn new_frame(stack: &mut [*mut c_void], size: usize, capacity: usize) -> FrameIdx;
    unsafe fn new_dynamic_frame(stack: &mut [*mut c_void], size: usize) -> FrameIdx;
    unsafe fn pop_frame(stack: &mut [*mut c_void], idx: FrameIdx);
}

unsafe impl Mode for Sync {
    #[inline(always)]
    unsafe fn new_frame(stack: &mut [*mut c_void], size: usize, capacity: usize) -> FrameIdx {
        let rtls = &mut *jl_get_ptls_states();
        stack[size] = (capacity << 1) as _;
        stack[size + 1] = rtls.pgcstack.cast();

        for i in 0..capacity {
            stack[size + 2 + i] = null_mut();
        }

        rtls.pgcstack = stack[size..].as_mut_ptr().cast();
        stack[0] = (size + capacity + 2) as _;

        FrameIdx(size + 2)
    }

    #[inline(always)]
    unsafe fn new_dynamic_frame(stack: &mut [*mut c_void], size: usize) -> FrameIdx {
        let rtls = &mut *jl_get_ptls_states();
        stack[size] = 0 as _;
        stack[size + 1] = rtls.pgcstack.cast();

        rtls.pgcstack = stack[size..].as_mut_ptr().cast();
        stack[0] = (size + 2) as _;

        FrameIdx(size + 2)
    }

    #[inline(always)]
    unsafe fn pop_frame(stack: &mut [*mut c_void], idx: FrameIdx) {
        let rtls = &mut *jl_get_ptls_states();
        rtls.pgcstack = (&*rtls.pgcstack).prev;
        stack[0] = (idx.0 - 2) as _;
    }
}

#[cfg(feature = "async")]
unsafe impl Mode for Async {
    // In the async mode we're managing multiple stacks at the same time. These stacks are
    // chained together by using a frame with no slots that always exists for each frame. 
    // Schematically, it works like this:
    // [
    //     [3, 0, 0, 0...],
    //     [3, 0, p0, 0...],
    //     [3, 0, p1, 0...],
    //     ...
    // ]
    // 
    // [
    //     [3, 0, 0, 0...],
    //     [3, 0, pa, 1, p0, x, 0...],
    //     [3, 0, p1, 0...],
    //     ...
    // ] 
    // 
    // [
    //     [3, 0, 0, 0...],
    //     [3, 0, pb, 1, p0, x, 1, pa, y, 0...],
    //     [3, 0, p1, 0...],
    //     ...
    // ] 
    // 
    // [
    //     [3, 0, 0, 0...],
    //     [3, 0, pc, 1, p0, x, 1, pa, y, 1, pb, z, 0...],
    //     [3, 0, p1, 0...],
    //     ...
    // ]

    #[inline(always)]
    unsafe fn new_frame(stack: &mut [*mut c_void], size: usize, capacity: usize) -> FrameIdx {
        stack[size] = (capacity << 1) as _;
        stack[size + 1] = stack[2];

        for i in 0..capacity {
            stack[size + 2 + i] = null_mut();
        }

        stack[2] = stack[size..].as_mut_ptr().cast();
        stack[0] = (size + capacity + 2) as _;

        FrameIdx(size + 2)
    }

    #[inline(always)]
    unsafe fn new_dynamic_frame(stack: &mut [*mut c_void], size: usize) -> FrameIdx {
        stack[size] = 0 as _;
        stack[size + 1] = stack[2];

        stack[2] = stack[size..].as_mut_ptr().cast();
        stack[0] = (size + 2) as _;

        FrameIdx(size + 2)
    }

    #[inline(always)]
    unsafe fn pop_frame(stack: &mut [*mut c_void], idx: FrameIdx) {
        stack[0] = (idx.0 - 2) as _;
        stack[2] = stack[idx.0 - 1];
    }
}
