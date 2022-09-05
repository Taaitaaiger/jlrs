use std::{cell::Cell, ffi::c_void, ptr::null_mut};

const MIN_PAGE_SIZE: usize = 64;

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Slot {
    cell: Cell<*mut c_void>,
}

impl Slot {
    // Safety: Whenever a slot is updated the stack must remain valid.
    pub(crate) unsafe fn set(&self, val: *mut c_void) {
        self.cell.set(val)
    }

    pub(crate) fn get(&self) -> *mut c_void {
        self.cell.get()
    }
}

impl Default for Slot {
    fn default() -> Self {
        let cell = Cell::new(null_mut());
        Slot { cell }
    }
}

#[derive(Debug)]
pub(crate) struct StackPage {
    raw: Box<[Slot]>,
}

impl StackPage {
    pub(crate) fn new(min_capacity: usize) -> Self {
        let raw = vec![Slot::default(); MIN_PAGE_SIZE.max(min_capacity)].into_boxed_slice();
        StackPage { raw }
    }

    pub(crate) fn size(&self) -> usize {
        self.raw.len()
    }

    // Safety: invariants required by the GC must be maintained when changing the contents of a
    // stack page.
    pub(crate) unsafe fn as_ref(&self) -> &[Slot] {
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
        use std::ptr::NonNull;
        use cfg_if::cfg_if;

        #[derive(Debug)]
        pub(crate) struct AsyncStackPage {
            top: Box<[Slot; 2]>,
            page: StackPage,
        }

        // Not actually true, but we need to be able to send a page back after completing a task. The page
        // is never (and must never be) shared across threads.
        unsafe impl Send for AsyncStackPage {}
        unsafe impl Sync for AsyncStackPage {}

        impl AsyncStackPage {
            // Safety: the page must be linked into the stack with AsyncStackpages::link_stacks
            // before it can be used.
            pub(crate) unsafe fn new() -> Box<Self> {
                let stack = AsyncStackPage {
                    top: Box::new([Slot::default(), Slot::default()]),
                    page: StackPage::default(),
                };

                Box::new(stack)
            }

            // Safety: Must only be called when the async runtime is initialized.
            pub(crate) unsafe fn link_stacks(stacks: &[Option<Box<Self>>]) {
                cfg_if! {
                    if #[cfg(feature = "lts")] {
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
            pub(crate) unsafe fn slots(&self) -> &[Slot] {
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
            pub(crate) unsafe fn top(&self) -> &Slot {
                &self.top[1]
            }
        }
    }
}
