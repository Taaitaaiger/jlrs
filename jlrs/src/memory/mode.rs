//! Modes handle differences between the different runtime modes.

/// Mode used by the synchronous runtime.
#[derive(Clone, Copy)]
pub struct Sync;

/// This trait handles the differences between the runtime modes.
pub trait Mode: Copy + private::Mode {}

impl Mode for Sync {}

pub(crate) mod private {
    use crate::{memory::mode::Sync, private::Private};
    use jl_sys::jlrs_current_task;
    use std::ffi::c_void;
    use std::ptr::{null_mut, NonNull};

    pub trait Mode {
        unsafe fn push_frame(&self, raw_frame: &mut [*mut c_void], capacity: usize, _: Private);
        unsafe fn pop_frame(&self, raw_frame: &mut [*mut c_void], _: Private);
    }

    impl Mode for Sync {
        unsafe fn push_frame(&self, raw_frame: &mut [*mut c_void], capacity: usize, _: Private) {
            let task = jlrs_current_task().as_mut().unwrap();
            raw_frame[0] = (capacity << 1) as _;
            raw_frame[1] = task.gcstack.cast();

            for i in 0..capacity {
                raw_frame[2 + i] = null_mut();
            }

            task.gcstack = raw_frame[..].as_mut_ptr().cast();
        }

        unsafe fn pop_frame(&self, _: &mut [*mut c_void], _: Private) {
            let task = jlrs_current_task().as_mut().unwrap();
            task.gcstack = NonNull::new_unchecked(task.gcstack).as_ref().prev;
        }
    }
}
