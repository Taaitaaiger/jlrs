//! A GC-safe `FairMutex`.
//!
//! The API matches that of [`parking_lot::FairMutex`].

use super::raw_fair_mutex::RawGcSafeFairMutex;

/// A GC-safe fair mutex. See [`parking_lot::FairMutex`] for more information.
pub type GcSafeFairMutex<T> = lock_api::Mutex<RawGcSafeFairMutex, T>;

/// Create a new GC-safe fair mutex. See [`parking_lot::const_fair_mutex`] for more information.
pub const fn const_gc_safe_fair_mutex<T>(val: T) -> GcSafeFairMutex<T> {
    GcSafeFairMutex::const_new(<RawGcSafeFairMutex as lock_api::RawMutex>::INIT, val)
}

/// A GC-safe `FairMutexGuard`. See [`parking_lot::FairMutexGuard`] for more information.
pub type GcSafeFairMutexGuard<'a, T> = lock_api::MutexGuard<'a, RawGcSafeFairMutex, T>;

/// A GC-safe `MappedFairMutexGuard`. See [`parking_lot::MappedFairMutexGuard`] for more
/// information.
pub type MappedGcSafeFairMutexGuard<'a, T> = lock_api::MappedMutexGuard<'a, RawGcSafeFairMutex, T>;
