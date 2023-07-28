//! A GC-safe `OnceLock`.
//!
//! The API matches that of [`once_cell::sync::OnceCell`].

use std::fmt;

use jl_sys::{jlrs_gc_safe_enter, jlrs_gc_safe_leave, jlrs_gc_unsafe_enter, jlrs_gc_unsafe_leave};
use once_cell::sync::OnceCell;

use crate::memory::get_tls;

/// A GC-safe `OnceLock`. See [`once_cell::sync::OnceCell`] for more information.
#[derive(Clone)]
pub struct GcSafeOnceLock<T> {
    inner: OnceCell<T>,
}

impl<T> Default for GcSafeOnceLock<T> {
    fn default() -> GcSafeOnceLock<T> {
        GcSafeOnceLock::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for GcSafeOnceLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> From<T> for GcSafeOnceLock<T> {
    fn from(value: T) -> Self {
        Self::with_value(value)
    }
}

impl<T: PartialEq> PartialEq for GcSafeOnceLock<T> {
    fn eq(&self, other: &GcSafeOnceLock<T>) -> bool {
        self.get() == other.get()
    }
}

impl<T: Eq> Eq for GcSafeOnceLock<T> {}

impl<T> GcSafeOnceLock<T> {
    #[inline]
    pub const fn new() -> Self {
        GcSafeOnceLock {
            inner: OnceCell::new(),
        }
    }

    /// Creates a new initialized cell.
    #[inline]
    pub const fn with_value(value: T) -> GcSafeOnceLock<T> {
        GcSafeOnceLock {
            inner: OnceCell::with_value(value),
        }
    }

    #[inline]
    pub fn get(&self) -> Option<&T> {
        self.inner.get()
    }

    #[inline]
    pub fn wait(&self) -> &T {
        unsafe {
            let ptls = get_tls();
            let state = jlrs_gc_safe_enter(ptls);
            let res = self.inner.wait();
            jlrs_gc_safe_leave(ptls, state);
            res
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.inner.get_mut()
    }

    #[inline]
    pub unsafe fn get_unchecked(&self) -> &T {
        self.inner.get_unchecked()
    }

    #[inline]
    pub fn set(&self, value: T) -> Result<(), T> {
        unsafe {
            let ptls = get_tls();
            let state = jlrs_gc_safe_enter(ptls);
            let res = self.inner.set(value);
            jlrs_gc_safe_leave(ptls, state);
            res
        }
    }

    #[inline]
    pub fn try_insert(&self, value: T) -> Result<&T, (&T, T)> {
        self.inner.try_insert(value)
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if let Some(v) = self.get() {
            return v;
        }

        unsafe {
            let ptls = get_tls();
            let state = jlrs_gc_safe_enter(ptls);

            let res = self.inner.get_or_init(|| {
                let state = jlrs_gc_unsafe_enter(ptls);
                let res = f();
                jlrs_gc_unsafe_leave(ptls, state);
                res
            });

            jlrs_gc_safe_leave(ptls, state);
            res
        }
    }

    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if let Some(v) = self.get() {
            return Ok(v);
        }

        unsafe {
            let ptls = get_tls();
            let state = jlrs_gc_safe_enter(ptls);

            let res = self.inner.get_or_try_init(|| {
                let state = jlrs_gc_unsafe_enter(ptls);
                let res = f();
                jlrs_gc_unsafe_leave(ptls, state);
                res
            });

            jlrs_gc_unsafe_leave(ptls, state);
            res
        }
    }

    #[inline]
    pub fn take(&mut self) -> Option<T> {
        self.inner.take()
    }

    #[inline]
    pub fn into_inner(self) -> Option<T> {
        self.inner.into_inner()
    }
}
