//! Mostly internal trait to handle differences between the different runtime modes.

/// Mode used by the synchronous runtime.
#[derive(Clone, Copy)]
pub struct Sync;

/// This trait handles the differences between the runtime modes.
pub trait Mode: Copy + private::Mode {}

impl Mode for Sync {}

pub(crate) mod private {
    use crate::{memory::mode::Sync, private::Private};
    use jl_sys::jl_get_ptls_states;
    use std::ffi::c_void;
    use std::ptr::null_mut;

    pub trait Mode {
        unsafe fn push_frame(&self, raw_frame: &mut [*mut c_void], capacity: usize, _: Private);
        unsafe fn pop_frame(&self, raw_frame: &mut [*mut c_void], _: Private);
    }

    impl Mode for Sync {
        unsafe fn push_frame(&self, raw_frame: &mut [*mut c_void], capacity: usize, _: Private) {
            let rtls = &mut *jl_get_ptls_states();
            raw_frame[0] = (capacity << 1) as _;
            raw_frame[1] = rtls.pgcstack.cast();

            for i in 0..capacity {
                raw_frame[2 + i] = null_mut();
            }

            rtls.pgcstack = raw_frame[..].as_mut_ptr().cast();
        }

        unsafe fn pop_frame(&self, _: &mut [*mut c_void], _: Private) {
            let rtls = &mut *jl_get_ptls_states();
            rtls.pgcstack = (&*rtls.pgcstack).prev;
        }
    }
}
