use std::time::{Duration, Instant};

use jl_sys::{jlrs_gc_safe_enter, jlrs_gc_safe_leave};
use parking_lot::RawMutex;

use crate::memory::get_tls;

/// See [`parking_lot::RawFairMutex`] for more information.
pub struct RawGcSafeFairMutex {
    inner: RawMutex,
}

unsafe impl lock_api::RawMutex for RawGcSafeFairMutex {
    const INIT: Self = RawGcSafeFairMutex {
        inner: <RawMutex as lock_api::RawMutex>::INIT,
    };

    type GuardMarker = <RawMutex as lock_api::RawMutex>::GuardMarker;

    #[inline]
    fn lock(&self) {
        unsafe {
            if self.try_lock() {
                return;
            }

            let ptls = get_tls();
            let state = jlrs_gc_safe_enter(ptls);
            self.inner.lock();
            jlrs_gc_safe_leave(ptls, state);
        }
    }

    #[inline]
    fn try_lock(&self) -> bool {
        self.inner.try_lock()
    }

    #[inline]
    unsafe fn unlock(&self) {
        self.inner.unlock()
    }

    #[inline]
    fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
}

unsafe impl lock_api::RawMutexFair for RawGcSafeFairMutex {
    #[inline]
    unsafe fn unlock_fair(&self) {
        self.inner.unlock_fair()
    }

    #[inline]
    unsafe fn bump(&self) {
        self.inner.bump()
    }
}

unsafe impl lock_api::RawMutexTimed for RawGcSafeFairMutex {
    type Duration = Duration;
    type Instant = Instant;

    #[inline]
    fn try_lock_until(&self, timeout: Instant) -> bool {
        self.inner.try_lock_until(timeout)
    }

    #[inline]
    fn try_lock_for(&self, timeout: Duration) -> bool {
        self.inner.try_lock_for(timeout)
    }
}
