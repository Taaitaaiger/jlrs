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

    // Safety: invariants required by the GC must be maintained when changing the contents of a
    // stack page.
    pub(crate) unsafe fn as_ref(&self) -> &[Cell<*mut c_void>] {
        self.raw.as_ref()
    }
}

impl Default for StackPage {
    fn default() -> Self {
        Self::new(MIN_PAGE_SIZE)
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
        use std::pin::Pin;
        use std::ptr::NonNull;
        use cfg_if::cfg_if;

        #[derive(Debug)]
        pub(crate) struct AsyncStackPage {
            top: Pin<Box<[Cell<*mut c_void>; 2]>>,
            page: StackPage,
        }

        // Not actually true, but we need to be able to send a page back after completing a task. The page
        // is never (and must never be) shared across threads.
        unsafe impl Send for AsyncStackPage {}
        unsafe impl Sync for AsyncStackPage {}

        impl AsyncStackPage {
            // Safety: the page must be linked into the stack with AsyncStackpages::link_stacks
            // before it can be used.
            pub(crate) unsafe fn new() -> Pin<Box<Self>> {
                let stack = AsyncStackPage {
                    top: Box::pin([Cell::new(null_mut()), Cell::new(null_mut())]),
                    page: StackPage::default(),
                };

                Box::pin(stack)
            }

            // Safety: Must only be called when the async runtime is initialized.
            pub(crate) unsafe fn link_stacks(stacks: &[Option<Pin<Box<Self>>>]) {
                cfg_if! {
                    if #[cfg(all(feature = "lts", not(feature = "all-features-override")))] {
                        for stack in stacks.iter() {
                            let stack = stack.as_ref().unwrap_unchecked();
                            let rtls = NonNull::new_unchecked(jl_sys::jl_get_ptls_states()).as_mut();
                            stack.top[1].set(rtls.pgcstack.cast());
                            rtls.pgcstack = stack.top[0..].as_ptr() as *const _ as *mut _;
                        }
                    } else {
                        use jl_sys::{jl_get_current_task, jl_task_t};

                        for stack in stacks.iter() {
                            let stack = stack.as_ref().unwrap_unchecked();
                            let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();

                            stack.top[1].set(task.gcstack.cast());
                            task.gcstack = stack.top[0..].as_ptr() as *const _ as *mut _;
                        }
                    }
                }
            }

            // Safety: invariants required by the GC must be maintained when changing the contents of a
            // stack page.
            pub(crate) unsafe fn page(&self) -> &[Cell<*mut c_void>] {
                self.page.as_ref()
            }

            pub(crate) fn size(&self) -> usize {
                self.page.size()
            }

            // Safety: invariants required by the GC must be maintained when changing the contents of a
            // stack page. A page can only be replaced it it's not in use.
            pub(crate) unsafe fn page_mut(&mut self) -> &mut StackPage {
                &mut self.page
            }

            // Safety: whenever a new frame is pushed to this stack, this pointer has to be used
            // as if it's pointer to the top of the GC stack to ensure a single nested hierarchy
            // of scopes is maintained.
            pub(crate) unsafe fn top(&self) -> &Cell<*mut c_void> {
                &self.top[1]
            }
        }
    }
}
