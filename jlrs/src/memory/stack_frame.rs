//! Raw, inactive GC frame.

use std::{
    ffi::c_void,
    pin::Pin,
    ptr::{null_mut, NonNull},
};

use jl_sys::{pop_frame, SplitGcFrame};

use super::context::stack::{Stack, STACK_TYPE_NAME};
use crate::{
    data::managed::{private::ManagedPriv, value::Value},
    private::Private,
};

/// A raw, inactive GC frame.
///
/// When the [local runtime] or [`CCall`] is used a `StackFrame` must be provided so the GC can
/// find all data rooted in active [`GcFrame`]s.
///
/// [local runtime]: crate::runtime::sync_rt::Julia
/// [`CCall`]: crate::runtime::handle::ccall::CCall
/// [`GcFrame`]: crate::memory::target::frame::GcFrame
#[repr(C)]
pub struct StackFrame<const N: usize> {
    s: SplitGcFrame<1, N>,
}

impl StackFrame<0> {
    /// Returns a new `StackFrame`.
    #[inline]
    pub fn new() -> Self {
        Self::new_n()
    }
}

impl<const N: usize> StackFrame<N> {
    #[inline]
    pub(crate) fn new_n() -> Self {
        unsafe {
            StackFrame {
                s: SplitGcFrame::new(),
            }
        }
    }

    // Safety: Must only be called once, if a new frame is pushed it must be popped before
    // this one is.
    #[inline]
    #[cfg_attr(
        not(any(feature = "local-rt", feature = "async-rt", feature = "ccall")),
        allow(unused)
    )]
    pub(crate) unsafe fn pin<'scope>(&'scope mut self) -> PinnedFrame<'scope, N> {
        self.s.get_head_root(0).set(null_mut());
        for i in 0..N {
            self.s.set_tail_root(i, null_mut());
        }
        PinnedFrame::new(self)
    }
}

pub(crate) struct PinnedFrame<'scope, const N: usize> {
    raw: Pin<&'scope StackFrame<N>>,
}

impl<'scope, const N: usize> PinnedFrame<'scope, N> {
    #[inline]
    #[cfg_attr(
        not(any(feature = "local-rt", feature = "async-rt", feature = "ccall")),
        allow(unused)
    )]
    unsafe fn new(raw: &'scope mut StackFrame<N>) -> Self {
        raw.s.push_frame();
        PinnedFrame {
            raw: Pin::new_unchecked(raw),
        }
    }

    #[inline]
    #[cfg_attr(
        not(any(feature = "local-rt", feature = "async-rt", feature = "ccall")),
        allow(unused)
    )]
    pub(crate) unsafe fn stack_frame<'inner>(
        &'inner mut self,
    ) -> JlrsStackFrame<'scope, 'inner, N> {
        JlrsStackFrame::new(self)
    }

    #[inline]
    #[allow(unused)]
    pub(crate) unsafe fn set_sync_root(&self, root: *mut c_void) {
        self.raw.s.set_head_root(0, root);
    }
}

impl<'scope, const N: usize> Drop for PinnedFrame<'scope, N> {
    fn drop(&mut self) {
        unsafe { pop_frame() }
    }
}

#[cfg_attr(
    not(any(feature = "local-rt", feature = "async-rt", feature = "ccall")),
    allow(unused)
)]
pub(crate) struct JlrsStackFrame<'scope, 'inner, const N: usize> {
    pinned: &'inner mut PinnedFrame<'scope, N>,
}

impl<'scope, 'inner, const N: usize> JlrsStackFrame<'scope, 'inner, N> {
    #[inline]
    #[cfg_attr(
        not(any(feature = "local-rt", feature = "async-rt", feature = "ccall")),
        allow(unused)
    )]
    unsafe fn new(pinned: &'inner mut PinnedFrame<'scope, N>) -> Self {
        if !Self::is_init(&pinned) {
            {
                let ptr = Stack::alloc();
                pinned.raw.s.set_head_root(0, ptr.cast());
            }

            for i in 0..N {
                let ptr = Stack::alloc();
                pinned.raw.s.set_tail_root(i, ptr.cast());
            }
        }

        JlrsStackFrame { pinned }
    }

    #[inline]
    #[cfg_attr(
        not(any(feature = "local-rt", feature = "async-rt", feature = "ccall")),
        allow(unused)
    )]
    pub(crate) unsafe fn sync_stack(&self) -> &'scope Stack {
        NonNull::new_unchecked(self.pinned.raw.s.get_head_root(0).get())
            .cast()
            .as_ref()
    }

    #[cfg(feature = "async")]
    #[inline]
    pub(crate) unsafe fn nth_stack(&self, n: usize) -> &'scope Stack {
        NonNull::new_unchecked(self.pinned.raw.s.get_tail_root(n).get())
            .cast()
            .as_ref()
    }

    #[inline]
    #[cfg_attr(
        not(any(feature = "local-rt", feature = "async-rt", feature = "ccall")),
        allow(unused)
    )]
    fn is_init(pinned: &PinnedFrame<'_, N>) -> bool {
        unsafe {
            let ptr = pinned.raw.s.get_head_root(0).get();
            if !ptr.is_null() {
                let v = Value::wrap_non_null(NonNull::new_unchecked(ptr).cast(), Private);
                let sym = STACK_TYPE_NAME.as_symbol();
                return v.datatype_name() == sym.as_str().unwrap();
            }

            false
        }
    }
}
