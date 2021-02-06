//! A trait for handling the differences between the runtime modes.

#[cfg(all(feature = "async", target_os = "linux"))]
use crate::memory::mode::Async;
use crate::memory::mode::Sync;

/// This trait handles the differences between the runtime modes.
pub trait Mode: Copy + private::Mode {}

impl Mode for Sync {}

#[cfg(all(feature = "async", target_os = "linux"))]
impl<'a> Mode for Async<'a> {}

pub(crate) mod private {
    #[cfg(all(feature = "async", target_os = "linux"))]
    use crate::memory::mode::Async;
    use crate::{memory::mode::Sync, value::traits::private::Internal};
    use jl_sys::jl_get_ptls_states;
    use std::ffi::c_void;
    use std::ptr::null_mut;

    pub trait Mode {
        unsafe fn push_frame(&self, raw_frame: &mut [*mut c_void], capacity: usize, _: Internal);
        unsafe fn pop_frame(&self, raw_frame: &mut [*mut c_void], _: Internal);
    }

    impl Mode for Sync {
        unsafe fn push_frame(&self, raw_frame: &mut [*mut c_void], capacity: usize, _: Internal) {
            let rtls = &mut *jl_get_ptls_states();
            raw_frame[0] = (capacity << 1) as _;
            raw_frame[1] = rtls.pgcstack.cast();

            for i in 0..capacity {
                raw_frame[2 + i] = null_mut();
            }

            rtls.pgcstack = raw_frame[..].as_mut_ptr().cast();
        }

        unsafe fn pop_frame(&self, _: &mut [*mut c_void], _: Internal) {
            let rtls = &mut *jl_get_ptls_states();
            rtls.pgcstack = (&*rtls.pgcstack).prev;
        }
    }

    #[cfg(all(feature = "async", target_os = "linux"))]
    impl<'a> Mode for Async<'a> {
        unsafe fn push_frame(&self, raw_frame: &mut [*mut c_void], capacity: usize, _: Internal) {
            raw_frame[0] = (capacity << 1) as _;
            raw_frame[1] = self.0.get();

            for i in 0..capacity {
                raw_frame[2 + i] = null_mut();
            }

            self.0.set(raw_frame[..].as_mut_ptr().cast());
        }

        unsafe fn pop_frame(&self, raw_frame: &mut [*mut c_void], _: Internal) {
            self.0.set(raw_frame[1]);
        }
    }
}
