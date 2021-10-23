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
    use std::ptr::{null_mut, NonNull};
    use std::{cell::Cell, ffi::c_void};

    pub trait Mode {
        unsafe fn push_frame(&self, raw_frame: &mut [Cell<*mut c_void>], _: Private);
        unsafe fn pop_frame(&self, raw_frame: &mut [Cell<*mut c_void>], _: Private);
    }

    impl Mode for Sync {
        unsafe fn push_frame(&self, raw_frame: &mut [Cell<*mut c_void>], _: Private) {
            let task = jlrs_current_task().as_mut().unwrap();
            raw_frame[0].set(null_mut());
            raw_frame[1].set(task.gcstack.cast());
            task.gcstack = raw_frame[..].as_mut_ptr().cast();
        }

        unsafe fn pop_frame(&self, _raw_frame: &mut [Cell<*mut c_void>], _: Private) {
            let task = jlrs_current_task().as_mut().unwrap();
            task.gcstack = NonNull::new_unchecked(task.gcstack).as_ref().prev;
        }
    }
}
