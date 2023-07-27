use std::time::{Duration, Instant};

use jl_sys::{jlrs_gc_safe_enter, jlrs_gc_safe_leave};
use parking_lot::RawRwLock;

use crate::memory::get_tls;

/// See [`parking_lot::RawRwLock`] for more information.
pub struct RawGcSafeRwLock {
    inner: RawRwLock,
}

unsafe impl lock_api::RawRwLock for RawGcSafeRwLock {
    const INIT: Self = RawGcSafeRwLock {
        inner: RawRwLock::INIT,
    };

    type GuardMarker = <RawRwLock as lock_api::RawRwLock>::GuardMarker;

    #[inline]
    fn lock_shared(&self) {
        unsafe {
            if self.try_lock_shared() {
                return;
            }

            let ptls = get_tls();
            let state = jlrs_gc_safe_enter(ptls);
            self.inner.lock_shared();
            jlrs_gc_safe_leave(ptls, state);
        }
    }

    #[inline]
    fn try_lock_shared(&self) -> bool {
        self.inner.try_lock_shared()
    }

    #[inline]
    unsafe fn unlock_shared(&self) {
        self.inner.unlock_shared();
    }

    #[inline]
    fn lock_exclusive(&self) {
        unsafe {
            if self.try_lock_exclusive() {
                return;
            }

            let ptls = get_tls();
            let state = jlrs_gc_safe_enter(ptls);
            self.inner.lock_exclusive();
            jlrs_gc_safe_leave(ptls, state);
        }
    }

    #[inline]
    fn try_lock_exclusive(&self) -> bool {
        self.inner.try_lock_exclusive()
    }

    #[inline]
    unsafe fn unlock_exclusive(&self) {
        self.inner.unlock_exclusive();
    }
}

unsafe impl lock_api::RawRwLockFair for RawGcSafeRwLock {
    #[inline]
    unsafe fn unlock_shared_fair(&self) {
        self.inner.unlock_shared_fair()
    }

    #[inline]
    unsafe fn unlock_exclusive_fair(&self) {
        self.inner.unlock_exclusive_fair()
    }

    #[inline]
    unsafe fn bump_shared(&self) {
        self.inner.bump_shared()
    }

    #[inline]
    unsafe fn bump_exclusive(&self) {
        self.inner.bump_exclusive()
    }
}

unsafe impl lock_api::RawRwLockDowngrade for RawGcSafeRwLock {
    #[inline]
    unsafe fn downgrade(&self) {
        self.inner.downgrade()
    }
}

unsafe impl lock_api::RawRwLockTimed for RawGcSafeRwLock {
    type Duration = Duration;
    type Instant = Instant;

    #[inline]
    fn try_lock_shared_for(&self, timeout: Self::Duration) -> bool {
        self.inner.try_lock_shared_for(timeout)
    }

    #[inline]
    fn try_lock_shared_until(&self, timeout: Self::Instant) -> bool {
        self.inner.try_lock_shared_until(timeout)
    }

    #[inline]
    fn try_lock_exclusive_for(&self, timeout: Duration) -> bool {
        self.inner.try_lock_exclusive_for(timeout)
    }

    #[inline]
    fn try_lock_exclusive_until(&self, timeout: Instant) -> bool {
        self.inner.try_lock_exclusive_until(timeout)
    }
}

unsafe impl lock_api::RawRwLockRecursive for RawGcSafeRwLock {
    #[inline]
    fn lock_shared_recursive(&self) {
        self.inner.lock_shared_recursive()
    }

    #[inline]
    fn try_lock_shared_recursive(&self) -> bool {
        self.inner.try_lock_shared_recursive()
    }
}

unsafe impl lock_api::RawRwLockRecursiveTimed for RawGcSafeRwLock {
    #[inline]
    fn try_lock_shared_recursive_for(&self, timeout: Self::Duration) -> bool {
        self.inner.try_lock_shared_recursive_for(timeout)
    }

    #[inline]
    fn try_lock_shared_recursive_until(&self, timeout: Self::Instant) -> bool {
        self.inner.try_lock_shared_recursive_until(timeout)
    }
}

unsafe impl lock_api::RawRwLockUpgrade for RawGcSafeRwLock {
    #[inline]
    fn lock_upgradable(&self) {
        unsafe {
            if self.try_lock_upgradable() {
                return;
            }

            let ptls = get_tls();
            let state = jlrs_gc_safe_enter(ptls);
            self.inner.lock_upgradable();
            jlrs_gc_safe_leave(ptls, state);
        }
    }

    #[inline]
    fn try_lock_upgradable(&self) -> bool {
        self.inner.try_lock_upgradable()
    }

    #[inline]
    unsafe fn unlock_upgradable(&self) {
        self.inner.unlock_upgradable()
    }

    #[inline]
    unsafe fn upgrade(&self) {
        unsafe {
            if self.try_upgrade() {
                return;
            }

            let ptls = get_tls();
            let state = jlrs_gc_safe_enter(ptls);
            self.inner.upgrade();
            jlrs_gc_safe_leave(ptls, state);
        }
    }

    #[inline]
    unsafe fn try_upgrade(&self) -> bool {
        self.inner.try_upgrade()
    }
}

unsafe impl lock_api::RawRwLockUpgradeFair for RawGcSafeRwLock {
    #[inline]
    unsafe fn unlock_upgradable_fair(&self) {
        self.inner.unlock_upgradable_fair()
    }

    #[inline]
    unsafe fn bump_upgradable(&self) {
        self.inner.bump_upgradable()
    }
}

unsafe impl lock_api::RawRwLockUpgradeDowngrade for RawGcSafeRwLock {
    #[inline]
    unsafe fn downgrade_upgradable(&self) {
        self.inner.downgrade_upgradable()
    }

    #[inline]
    unsafe fn downgrade_to_upgradable(&self) {
        self.inner.downgrade_to_upgradable()
    }
}

unsafe impl lock_api::RawRwLockUpgradeTimed for RawGcSafeRwLock {
    #[inline]
    fn try_lock_upgradable_until(&self, timeout: Instant) -> bool {
        self.inner.try_lock_upgradable_until(timeout)
    }

    #[inline]
    fn try_lock_upgradable_for(&self, timeout: Duration) -> bool {
        self.inner.try_lock_upgradable_for(timeout)
    }

    #[inline]
    unsafe fn try_upgrade_until(&self, timeout: Instant) -> bool {
        self.inner.try_upgrade_until(timeout)
    }

    #[inline]
    unsafe fn try_upgrade_for(&self, timeout: Duration) -> bool {
        self.inner.try_upgrade_for(timeout)
    }
}
