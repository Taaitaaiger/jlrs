use std::{cell::Cell, ffi::c_void, ptr::null_mut};

const MIN_PAGE_SIZE: usize = 64;

#[derive(Debug)]
pub(crate) struct StackPage {
    raw: Box<[Cell<*mut c_void>]>,
}

impl StackPage {
    pub(crate) fn new(min_capacity: usize) -> Self {
        let raw = vec![Cell::new(null_mut()); MIN_PAGE_SIZE.max(min_capacity)];
        StackPage {
            raw: raw.into_boxed_slice(),
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.raw.len()
    }
}

impl Default for StackPage {
    fn default() -> Self {
        Self::new(MIN_PAGE_SIZE)
    }
}

impl AsMut<[Cell<*mut c_void>]> for StackPage {
    fn as_mut(&mut self) -> &mut [Cell<*mut c_void>] {
        self.raw.as_mut()
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
        use std::pin::Pin;
        use std::ptr::NonNull;
        use cfg_if::cfg_if;

        #[derive(Debug)]
        pub(crate) struct AsyncStackPage {
            pub(crate) top: Pin<Box<[Cell<*mut c_void>; 2]>>,
            pub(crate) page: StackPage,
        }

        // Not actually true, but we need to be able to send a page back after completing a task. The page
        // is never (and must never be) shared across threads.
        unsafe impl Send for AsyncStackPage {}
        unsafe impl Sync for AsyncStackPage {}

        impl AsyncStackPage {
            pub(crate) unsafe fn new() -> Pin<Box<Self>> {
                let stack = AsyncStackPage {
                    top: Box::pin([Cell::new(null_mut()), Cell::new(null_mut())]),
                    page: StackPage::default(),
                };

                Box::pin(stack)
            }

            pub(crate) unsafe fn link_stacks(stacks: &mut [Option<Pin<Box<Self>>>]) {
                cfg_if! {
                    if #[cfg(all(feature = "lts", not(feature = "all-features-override")))] {
                        for stack in stacks.iter_mut() {
                            let stack = stack.as_mut().unwrap();
                            let rtls = NonNull::new_unchecked(jl_sys::jl_get_ptls_states()).as_mut();
                            stack.top[1].set(rtls.pgcstack.cast());
                            rtls.pgcstack = stack.top[0..].as_mut_ptr().cast();
                        }
                    } else {
                        use jl_sys::{jl_get_current_task, jl_task_t};

                        for stack in stacks.iter_mut() {
                            let stack = stack.as_mut().unwrap();
                            let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();

                            stack.top[1].set(task.gcstack.cast());
                            task.gcstack = stack.top[0..].as_mut_ptr().cast();
                        }
                    }
                }
            }
        }
    }
}
