//! A raw, stack-allocated GC frame.

use std::{
    cell::Cell,
    ffi::c_void,
    pin::Pin,
    ptr::{null_mut, NonNull},
};

#[julia_version(since = "1.7")]
use jl_sys::{jl_get_current_task, jl_task_t};
use jlrs_macros::julia_version;

use super::context::stack::{Stack, STACK_TYPE_NAME};
use crate::{
    data::managed::{private::ManagedPriv, value::Value},
    private::Private,
};

const ROOT: Cell<*mut c_void> = Cell::new(null_mut());

/// A raw, stack-allocated GC frame.
///
/// When the [sync runtime] or [`CCall`] is used a `StackFrame` must be provided so the GC can
/// find all references to Julia data that exist in Rust code.
///
/// [sync runtime]: crate::runtime::sync_rt::Julia
/// [`CCall`]: crate::ccall::CCall
#[repr(C)]
pub struct StackFrame<const N: usize> {
    len: *mut c_void,
    prev: *mut c_void,
    sync: Cell<*mut c_void>,
    roots: [Cell<*mut c_void>; N],
}

impl StackFrame<0> {
    /// Returns a new `StackFrame`.
    pub fn new() -> Self {
        Self::new_n()
    }
}

impl<const N: usize> StackFrame<N> {
    pub(crate) fn new_n() -> Self {
        StackFrame {
            len: ((N + 1) << 2) as *mut c_void,
            prev: null_mut(),
            sync: ROOT,
            roots: [ROOT; N],
        }
    }

    // Safety: Must only be called once, if a new frame is pushed it must be popped before
    // this one is.
    pub(crate) unsafe fn pin<'scope>(&'scope mut self) -> PinnedFrame<'scope, N> {
        PinnedFrame::new(self)
    }
}

pub(crate) struct PinnedFrame<'scope, const N: usize> {
    raw: Pin<&'scope StackFrame<N>>,
}

impl<'scope, const N: usize> PinnedFrame<'scope, N> {
    #[julia_version(until = "1.6")]
    unsafe fn new(raw: &'scope mut StackFrame<N>) -> Self {
        let ptls = jl_sys::jl_get_ptls_states();
        let mut pgcstack = NonNull::new_unchecked(jl_sys::jlrs_pgcstack(ptls));
        let gcstack_ref: &mut *mut c_void = pgcstack.as_mut();

        #[cfg(feature = "mem-debug")]
        eprintln!("Push raw frame: {:p} -> {:p}", gcstack_ref, raw);
        raw.prev = *gcstack_ref;
        *gcstack_ref = raw as *mut _ as *mut _;

        PinnedFrame { raw: Pin::new(raw) }
    }

    #[julia_version(since = "1.7")]
    unsafe fn new(raw: &'scope mut StackFrame<N>) -> Self {
        let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();
        raw.prev = task.gcstack.cast();

        #[cfg(feature = "mem-debug")]
        eprintln!("Push raw frame: {:p} -> {:p}", task.gcstack, raw);
        task.gcstack = raw as *mut _ as *mut _;

        PinnedFrame { raw: Pin::new(raw) }
    }

    pub(crate) unsafe fn stack_frame<'inner>(
        &'inner mut self,
    ) -> JlrsStackFrame<'scope, 'inner, N> {
        JlrsStackFrame::new(self)
    }

    pub(crate) unsafe fn set_sync_root(&self, root: *mut c_void) {
        self.raw.sync.set(root);
    }

    pub(crate) unsafe fn clear_roots(&self) {
        self.raw.sync.set(null_mut());
        for r in self.raw.roots.as_ref() {
            r.set(null_mut());
        }
    }
}

impl<'scope, const N: usize> Drop for PinnedFrame<'scope, N> {
    #[julia_version(until = "1.6")]
    fn drop(&mut self) {
        unsafe {
            let ptls = jl_sys::jl_get_ptls_states();
            let mut pgcstack = NonNull::new_unchecked(jl_sys::jlrs_pgcstack(ptls));
            let gcstack_ref: &mut *mut c_void = pgcstack.as_mut();

            #[cfg(feature = "mem-debug")]
            eprintln!("Pop raw frame: {:p} -> {:p}", gcstack_ref, self.raw.prev);
            *gcstack_ref = self.raw.prev.cast();
            self.clear_roots();
        }
    }

    #[julia_version(since = "1.7")]
    fn drop(&mut self) {
        unsafe {
            use jl_sys::{jl_get_current_task, jl_task_t};
            let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();

            #[cfg(feature = "mem-debug")]
            eprintln!("Pop raw frame: {:p} -> {:p}", task.gcstack, self.raw.prev);
            task.gcstack = self.raw.prev.cast();
            self.clear_roots();
        }
    }
}

pub(crate) struct JlrsStackFrame<'scope, 'inner, const N: usize> {
    pinned: &'inner mut PinnedFrame<'scope, N>,
}

impl<'scope, 'inner, const N: usize> JlrsStackFrame<'scope, 'inner, N> {
    unsafe fn new(pinned: &'inner mut PinnedFrame<'scope, N>) -> Self {
        if !Self::is_init(&pinned) {
            {
                let ptr = Stack::alloc();
                pinned.raw.sync.set(ptr.cast());
            }
            for i in 0..N {
                let ptr = Stack::alloc();
                pinned.raw.roots[i].set(ptr.cast());
            }
        }

        JlrsStackFrame { pinned }
    }

    pub(crate) unsafe fn sync_stack(&self) -> &'scope Stack {
        NonNull::new_unchecked(self.pinned.raw.sync.get())
            .cast()
            .as_ref()
    }

    #[cfg(feature = "async")]
    pub(crate) unsafe fn nth_stack(&self, n: usize) -> &'scope Stack {
        NonNull::new_unchecked(self.pinned.raw.roots[n].get())
            .cast()
            .as_ref()
    }

    fn is_init(pinned: &PinnedFrame<'_, N>) -> bool {
        let ptr = pinned.raw.sync.get();
        if !ptr.is_null() {
            let v = unsafe { Value::wrap_non_null(NonNull::new_unchecked(ptr).cast(), Private) };
            let sym = STACK_TYPE_NAME.as_symbol();
            return v.datatype_name().unwrap_or("") == sym.as_str().unwrap();
        }

        false
    }
}
