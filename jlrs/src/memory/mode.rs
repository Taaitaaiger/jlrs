//! Modes handle differences between the different runtime modes.

/// Mode used by the sync runtime.
#[derive(Clone, Copy)]
pub struct Sync;

/// This trait handles the differences between the runtime modes.
pub trait Mode: Copy + private::Mode {}

impl Mode for Sync {}

pub(crate) mod private {
    use crate::{memory::mode::Sync, private::Private};
    #[cfg(not(feature = "lts"))]
    use jl_sys::{jl_get_current_task, jl_task_t};
    use std::ptr::{null_mut, NonNull};
    use std::{cell::Cell, ffi::c_void};

    pub trait Mode {
        unsafe fn push_frame(&self, raw_frame: &mut [Cell<*mut c_void>], _: Private);
        unsafe fn pop_frame(&self, raw_frame: &mut [Cell<*mut c_void>], _: Private);
    }

    impl Mode for Sync {
        #[cfg(not(feature = "lts"))]
        unsafe fn push_frame(&self, raw_frame: &mut [Cell<*mut c_void>], _: Private) {
            let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();
            raw_frame[0].set(null_mut());
            raw_frame[1].set(task.gcstack.cast());
            task.gcstack = raw_frame[..].as_mut_ptr().cast();
        }

        #[cfg(feature = "lts")]
        unsafe fn push_frame(&self, raw_frame: &mut [Cell<*mut c_void>], _: Private) {
            let rtls = NonNull::new_unchecked(jl_sys::jl_get_ptls_states()).as_mut();
            raw_frame[0].set(null_mut());
            raw_frame[1].set(rtls.pgcstack.cast());
            rtls.pgcstack = raw_frame[..].as_mut_ptr().cast();
        }

        #[cfg(not(feature = "lts"))]
        unsafe fn pop_frame(&self, _raw_frame: &mut [Cell<*mut c_void>], _: Private) {
            let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();
            task.gcstack = NonNull::new_unchecked(task.gcstack).as_ref().prev;
        }

        #[cfg(feature = "lts")]
        unsafe fn pop_frame(&self, _raw_frame: &mut [Cell<*mut c_void>], _: Private) {
            let rtls = NonNull::new_unchecked(jl_sys::jl_get_ptls_states()).as_mut();
            rtls.pgcstack = (&*rtls.pgcstack).prev;
        }
    }
}
