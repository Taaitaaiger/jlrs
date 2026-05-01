use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::{BuildHasher, Hash, RandomState},
    sync::atomic::{AtomicBool, Ordering},
};

use fnv::FnvBuildHasher;
use rustc_hash::FxBuildHasher;

use crate::{gc_safe::GcSafeRwLock, memory::gc::gc_safe};

pub(crate) struct Cache<K, V, S = RandomState> {
    map: GcSafeRwLock<HashMap<K, V, S>>,
    dirty: AtomicBool,
}

pub(crate) type FnvCache<K, V> = Cache<K, V, FnvBuildHasher>;
pub(crate) type FxCache<K, V> = Cache<K, V, FxBuildHasher>;

pub(crate) trait CacheMap<'a, K: 'a + Eq + Hash, V: 'a + Clone, S: BuildHasher> {
    fn new() -> Self;

    fn insert(&self, key: K, value: V) -> Option<V>;

    fn get<Q>(&'a self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    fn clear_dirty(&self);

    fn is_dirty(&self) -> bool;

    unsafe fn iter(&'a self) -> impl Iterator<Item = V>;
}

impl<'a, K: 'a + Eq + Hash, V: 'a + Clone> CacheMap<'a, K, V, FxBuildHasher> for FxCache<K, V> {
    fn new() -> Self {
        let hasher = FxBuildHasher;
        Cache {
            map: GcSafeRwLock::new(HashMap::with_hasher(hasher)),
            dirty: AtomicBool::new(false),
        }
    }

    fn insert(&self, key: K, value: V) -> Option<V> {
        let res = unsafe { gc_safe(|| self.map.write().insert(key, value)) };
        self.dirty.store(true, Ordering::Relaxed);
        res
    }

    fn get<Q>(&'a self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.read().get(key).cloned()
    }

    unsafe fn iter(&'a self) -> impl Iterator<Item = V> {
        self.map
            .read()
            .iter()
            .map(|(_k, v)| v.clone())
            .collect::<Vec<V>>()
            .into_iter()
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }
}

impl<'a, K: 'a + Eq + Hash, V: 'a + Clone> CacheMap<'a, K, V, FnvBuildHasher> for FnvCache<K, V> {
    fn new() -> Self {
        let hasher = FnvBuildHasher::new();
        Cache {
            map: GcSafeRwLock::new(HashMap::with_hasher(hasher)),
            dirty: AtomicBool::new(false),
        }
    }

    fn insert(&self, key: K, value: V) -> Option<V> {
        let res = self.map.write().insert(key, value);
        self.dirty.store(true, Ordering::Relaxed);
        res
    }

    fn get<Q>(&'a self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.read().get(key).cloned()
    }

    unsafe fn iter(&'a self) -> impl Iterator<Item = V> {
        self.map
            .read()
            .iter()
            .map(|(_k, v)| v.clone())
            .collect::<Vec<V>>()
            .into_iter()
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
