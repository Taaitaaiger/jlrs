//! GC-safe synchronization primitives.
//!
//! Naively using synchronization primitives like `OnceLock` or `Mutex` from a thread that belongs
//! to Julia is dangerous. If the GC needs to collect garbage while a thread is waiting to acquire
//! such a primitive you can end up with a deadlock. We can prevent this from happening by
//! entering a GC-safe state before blocking and leaving that state as soon as we wake up.
//!
//! This module offers the following GC-safe synchronization primitives: [`GcSafeOnceLock`],
//! [`GcSafeRwLock`], [`GcSafeMutex`], and [`GcSafeFairMutex`]. All of them guarantee the calling
//! thread is in a GC-safe state while it is blocked.

pub mod fair_mutex;
pub mod mutex;
pub mod once_lock;
mod raw_fair_mutex;
mod raw_mutex;
mod raw_rwlock;
pub mod rwlock;

pub use fair_mutex::GcSafeFairMutex;
pub use mutex::GcSafeMutex;
pub use once_lock::GcSafeOnceLock;
pub use raw_fair_mutex::RawGcSafeFairMutex;
pub use raw_mutex::RawGcSafeMutex;
pub use raw_rwlock::RawGcSafeRwLock;
pub use rwlock::GcSafeRwLock;
