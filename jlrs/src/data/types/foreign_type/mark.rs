//! Mark functions for exported foreign types
//!
//! Foreign types have to mark all their references to managed data. Any type that implements
//! `Mark` can be annotated with the `mark` attribute, other types require a custom marking
//! function that can be set with the `mark_with` attribute. These attributes do nothing unless
//! `ForeignType` is derived.

use jl_sys::{jl_gc_mark_queue_obj, jl_gc_mark_queue_objarray};

use super::ForeignType;
use crate::{data::managed::Weak, memory::PTls, prelude::Managed};

/// Mark all references to Julia data.
///
/// All references to Julia data in a foreign type must be marked in its mark method. Any type
/// that implements this trait can be annotated with the `mark` attribute.
///
/// Safety:
///
/// This trait may be assumed to be implemented correctly by unsafe code.
pub unsafe trait Mark {
    /// Mark all references to Julia data in `self`.
    ///
    /// Safety:
    ///
    /// All references to Julia data must be marked, `parent` must be managed by Julia, and this
    /// method must only be called by the GC.
    unsafe fn mark<P: ForeignType>(&self, ptls: PTls, parent: &P) -> usize;
}

unsafe impl<T: Managed<'static, 'static>> Mark for Weak<'static, 'static, T> {
    #[inline(always)]
    unsafe fn mark<P: ForeignType>(&self, ptls: PTls, _parent: &P) -> usize {
        unsafe { jl_gc_mark_queue_obj(ptls, self.ptr().as_ptr().cast()) as usize }
    }
}

unsafe impl<T: Managed<'static, 'static>> Mark for Option<Weak<'static, 'static, T>> {
    #[inline(always)]
    unsafe fn mark<P: ForeignType>(&self, ptls: PTls, _parent: &P) -> usize {
        match self {
            Some(weak) => jl_gc_mark_queue_obj(ptls, weak.ptr().as_ptr().cast()) as usize,
            None => 0,
        }
    }
}

unsafe impl<M: Mark, const N: usize> Mark for [M; N] {
    #[inline(always)]
    unsafe fn mark<P: ForeignType>(&self, ptls: PTls, parent: &P) -> usize {
        unsafe {
            jl_gc_mark_queue_objarray(
                ptls,
                parent as *const _ as *mut _,
                self.as_ptr() as *mut _,
                N as _,
            );
        }
        0
    }
}

unsafe impl<M: Mark, const N: usize> Mark for &[M; N] {
    #[inline(always)]
    unsafe fn mark<P: ForeignType>(&self, ptls: PTls, parent: &P) -> usize {
        unsafe {
            jl_gc_mark_queue_objarray(
                ptls,
                parent as *const _ as *mut _,
                self.as_ptr() as *mut _,
                N as _,
            );
        }
        0
    }
}

unsafe impl<M: Mark> Mark for &[M] {
    #[inline(always)]
    unsafe fn mark<P: ForeignType>(&self, ptls: PTls, parent: &P) -> usize {
        unsafe {
            jl_gc_mark_queue_objarray(
                ptls,
                parent as *const _ as *mut _,
                self.as_ptr() as *mut _,
                self.len() as _,
            );
        }
        0
    }
}

unsafe impl<M: Mark> Mark for Vec<M> {
    #[inline(always)]
    unsafe fn mark<P: ForeignType>(&self, ptls: PTls, parent: &P) -> usize {
        unsafe {
            jl_gc_mark_queue_objarray(
                ptls,
                parent as *const _ as *mut _,
                self.as_ptr() as *mut _,
                self.len() as _,
            );
        }
        0
    }
}

unsafe impl<T: ForeignType> Mark for T {
    unsafe fn mark<P: ForeignType>(&self, ptls: PTls, parent: &P) -> usize {
        T::mark(ptls, self, parent)
    }
}
