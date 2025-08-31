//! A GC-safe `Mutex`.
//!
//! The API matches that of [`parking_lot::Mutex`].

use super::raw_mutex::RawGcSafeMutex;

/// A GC-safe `Mutex`. See [`parking_lot::Mutex`] for more information.
pub type GcSafeMutex<T> = lock_api::Mutex<RawGcSafeMutex, T>;

/// Create a new GC-safe mutex. See [`parking_lot::const_mutex`] for more information.
pub const fn const_gc_safe_mutex<T>(val: T) -> GcSafeMutex<T> {
    GcSafeMutex::const_new(<RawGcSafeMutex as lock_api::RawMutex>::INIT, val)
}

/// A GC-safe `MutexGuard`. See [`parking_lot::MutexGuard`] for more information.
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawGcSafeMutex, T>;

/// A GC-safe `MappedMutexGuard`. See [`parking_lot::MappedMutexGuard`] for more information.
pub type MappedMutexGuard<'a, T> = lock_api::MappedMutexGuard<'a, RawGcSafeMutex, T>;
