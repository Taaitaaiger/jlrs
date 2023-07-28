//! A GC-safe `RwLock`.
//!
//! The API matches that of [`parking_lot::RwLock`].

use super::raw_rwlock::RawGcSafeRwLock;

/// A GC-safe fair `RwLock`. See [`parking_lot::RwLock`] for more information.
pub type GcSafeRwLock<T> = lock_api::RwLock<RawGcSafeRwLock, T>;

/// Create a new GC-safe `RwLock`. See [`parking_lot::const_rwlock`] for more information.
pub const fn const_gc_safe_rwlock<T>(val: T) -> GcSafeRwLock<T> {
    GcSafeRwLock::const_new(<RawGcSafeRwLock as lock_api::RawRwLock>::INIT, val)
}

/// A GC-safe `RwLockReadGuard`. See [`parking_lot::RwLockReadGuard`] for more information.
pub type GcSafeRwLockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, RawGcSafeRwLock, T>;

/// A GC-safe `RwLockWriteGuard`. See [`parking_lot::RwLockWriteGuard`] for more information.
pub type GcSafeRwLockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, RawGcSafeRwLock, T>;

/// A GC-safe `MappedRwLockReadGuard`. See [`parking_lot::MappedRwLockReadGuard`] for more
/// information.
pub type MappedGcSafeRwLockReadGuard<'a, T> =
    lock_api::MappedRwLockReadGuard<'a, RawGcSafeRwLock, T>;

/// A GC-safe `MappedRwLockWriteGuard`. See [`parking_lot::MappedRwLockWriteGuard`] for more
/// information.
pub type MappedGcSafeRwLockWriteGuard<'a, T> =
    lock_api::MappedRwLockWriteGuard<'a, RawGcSafeRwLock, T>;

/// A GC-safe `RwLockUpgradableReadGuard`. See [`parking_lot::RwLockUpgradableReadGuard`] for more
/// information.
pub type GcSafeRwLockUpgradableReadGuard<'a, T> =
    lock_api::RwLockUpgradableReadGuard<'a, RawGcSafeRwLock, T>;
