//! A handle that lets you call into Julia from the current thread.

use std::{fmt, marker::PhantomData};

use jl_sys::jl_atexit_hook;

use super::IsActive;
use crate::{
    memory::scope::{LocalReturning, LocalScope},
    runtime::state::set_exit,
};

/// A handle that lets you call into Julia from the current thread.
///
/// An `LocalHandle` can be created by calling [`Builder::start_local`]. Julia exits when this
/// handle is dropped.
///
/// [`Builder::start_local`]: crate::runtime::builder::Builder::start_local
pub struct LocalHandle {
    _marker: PhantomData<*mut ()>,
}

impl LocalHandle {
    pub(crate) unsafe fn new() -> Self {
        LocalHandle {
            _marker: PhantomData,
        }
    }
}

impl Drop for LocalHandle {
    fn drop(&mut self) {
        unsafe {
            jl_atexit_hook(0);
            set_exit();
        }
    }
}

impl fmt::Debug for LocalHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalHandle").finish()
    }
}

impl IsActive for LocalHandle {}

impl<'ctx> LocalReturning<'ctx> for LocalHandle {
    fn returning<T>(&mut self) -> &mut impl LocalScope<'ctx, T> {
        self
    }
}

impl<'ctx, T> LocalScope<'ctx, T> for LocalHandle {}
