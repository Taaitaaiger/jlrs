//! Julia data caching
//!
//! There are a few places where jlrs supports caching Julia data, e.g. type construction and
//! static references. To prevent that this data is ever freed by the garbage collector, this
//! cached data must be rooted. A cache consists of three parts: the actual cache, the set of
//! Julia data which is referenced by the cached data, and a GC-safe RW-lock to orchestrate
//! interactions with the cache.
//!
//! It's the user's resposibility to add all Julia data referenced by the cache to the roots, the
//! `mark` method must be called from a root scanner. A single root scanner is currently installed
//! by `init_jlrs` which calls `mark` for all default caches.

use fnv::FnvHashSet;
use jl_sys::{jl_gc_mark_queue_obj, jl_value_t};

use super::managed::private::ManagedPriv;
use crate::{gc_safe::GcSafeRwLock, memory::PTls, prelude::Value, private::Private};

/// Cache for global data
pub(crate) struct Cache<C> {
    inner: GcSafeRwLock<CacheInner<C>>,
}

impl<C: Default> Default for Cache<C> {
    fn default() -> Self {
        Self::new(C::default())
    }
}

impl<C> Cache<C> {
    pub(crate) const fn new(cache: C) -> Self {
        let inner = GcSafeRwLock::new(CacheInner {
            cache,
            roots: Roots::new(),
        });

        Cache { inner }
    }

    #[inline(always)]
    pub(crate) fn get_unchecked(&self) -> &Self {
        self
    }

    /// Get read access to the cache. `func` must never trigger garbage collection.
    #[inline]
    pub(crate) unsafe fn read<T>(&self, func: impl FnOnce(&CacheInner<C>) -> T) -> T {
        let read_guard = self.inner.read();
        func(&read_guard)
    }

    /// Get write access to the cache. `func` must never trigger garbage collection.
    #[inline]
    pub(crate) unsafe fn write<T>(&self, func: impl FnOnce(&mut CacheInner<C>) -> T) -> T {
        let mut write_guard = self.inner.write();
        func(&mut write_guard)
    }

    #[inline]
    pub(crate) unsafe fn mark(&self, ptls: PTls, full: bool) {
        unsafe {
            // We need a write guard because `Roots::mark` needs a mutable reference
            // If a collection is triggered while this lock is held, a deadlock will occur.
            let mut write_guard = self
                .inner
                .try_write()
                .expect("Could not lock cache, this is a bug in jlrs");

            write_guard.roots.mark(ptls, full);
        }
    }
}

// The lock ensures the cache is thread-safe,
unsafe impl<C> Send for Cache<C> {}
unsafe impl<C> Sync for Cache<C> {}

pub(crate) struct CacheInner<C> {
    cache: C,
    roots: Roots,
}

impl<C> CacheInner<C> {
    #[inline(always)]
    pub(crate) fn cache(&self) -> &C {
        &self.cache
    }

    #[inline(always)]
    pub(crate) fn cache_mut(&mut self) -> &mut C {
        &mut self.cache
    }

    #[inline(always)]
    pub(crate) fn roots_mut(&mut self) -> &mut Roots {
        &mut self.roots
    }
}

pub(crate) struct Roots {
    data: FnvHashSet<*mut jl_value_t>,
    // The roots are dirty whenever young objects may be present
    dirty: bool,
}

impl Roots {
    #[inline(always)]
    pub(crate) fn insert(&mut self, data: Value) {
        // `data` might be young if it wasn't present. If it was, it must either be old or the
        // roots have already been marked as dirty.
        self.dirty |= self.data.insert(data.unwrap(Private));
    }

    const fn new() -> Self {
        let hasher = fnv::FnvBuildHasher::new();
        Roots {
            data: FnvHashSet::with_hasher(hasher),
            dirty: false,
        }
    }

    // Safety: must only be called during GC mark phase from a root scanner.
    unsafe fn mark(&mut self, ptls: PTls, full: bool) {
        // If this is an incremental collection and the roots aren't dirty, we can skip marking.
        if !full && !self.dirty {
            return;
        }

        for value in self.data.iter().copied() {
            unsafe { jl_gc_mark_queue_obj(ptls, value) };
        }

        // Every young object that survives a collection cycle becomes old
        self.dirty = false;
    }
}
