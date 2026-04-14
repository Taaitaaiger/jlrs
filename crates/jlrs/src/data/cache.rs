use std::{
    borrow::Borrow,
    hash::{BuildHasher, Hash, RandomState},
    sync::atomic::{AtomicBool, Ordering},
};

use dashmap::{DashMap, iter::Iter, mapref::one::Ref};
use fnv::FnvBuildHasher;
use rustc_hash::FxBuildHasher;

use crate::memory::gc::gc_safe;

pub(crate) struct Cache<K, V, S = RandomState> {
    map: dashmap::DashMap<K, V, S>,
    dirty: AtomicBool,
}

pub(crate) type FnvCache<K, V> = Cache<K, V, FnvBuildHasher>;
pub(crate) type FxCache<K, V> = Cache<K, V, FxBuildHasher>;

pub(crate) trait CacheMap<'a, K: 'a + Eq + Hash, V: 'a, S: BuildHasher> {
    fn new() -> Self;

    fn insert(&self, key: K, value: V) -> Option<V>;

    fn get<Q>(&'a self, key: &Q) -> Option<Ref<'a, K, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    fn clear_dirty(&self);

    fn is_dirty(&self) -> bool;

    unsafe fn iter(&'a self) -> Iter<'a, K, V, S, DashMap<K, V, S>>;
}

impl<'a, K: 'a + Eq + Hash, V: 'a> CacheMap<'a, K, V, FxBuildHasher> for FxCache<K, V> {
    fn new() -> Self {
        let hasher = FxBuildHasher;
        Cache {
            map: DashMap::with_hasher(hasher),
            dirty: AtomicBool::new(false),
        }
    }

    fn insert(&self, key: K, value: V) -> Option<V> {
        let res = unsafe { gc_safe(|| self.map.insert(key, value)) };
        self.dirty.store(true, Ordering::Relaxed);
        res
    }

    fn get<Q>(&'a self, key: &Q) -> Option<Ref<'a, K, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        unsafe { gc_safe(|| self.map.get(key)) }
    }

    unsafe fn iter(&'a self) -> Iter<'a, K, V, FxBuildHasher, DashMap<K, V, FxBuildHasher>> {
        self.map.iter()
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }
}

impl<'a, K: 'a + Eq + Hash, V: 'a> CacheMap<'a, K, V, FnvBuildHasher> for FnvCache<K, V> {
    fn new() -> Self {
        let hasher = FnvBuildHasher::new();
        Cache {
            map: DashMap::with_hasher(hasher),
            dirty: AtomicBool::new(false),
        }
    }

    fn insert(&self, key: K, value: V) -> Option<V> {
        let res = unsafe { gc_safe(|| self.map.insert(key, value)) };
        self.dirty.store(true, Ordering::Relaxed);
        res
    }

    fn get<Q>(&'a self, key: &Q) -> Option<Ref<'a, K, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        unsafe { gc_safe(|| self.map.get(key)) }
    }

    unsafe fn iter(&'a self) -> Iter<'a, K, V, FnvBuildHasher, DashMap<K, V, FnvBuildHasher>> {
        self.map.iter()
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }
}

unsafe impl<K, V, S> Send for Cache<K, V, S> {}
unsafe impl<K, V, S> Sync for Cache<K, V, S> {}
