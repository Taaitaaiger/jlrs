use std::any::TypeId;

use crate::{
    data::{
        cache::{CacheMap, FnvCache, new_fnv_cache},
        layout::tuple::Tuple,
        managed::{
            Managed,
            datatype::DataType,
            value::{ValueUnbound, WeakValue},
        },
        types::construct_type::{ConstructType, type_var::TypeVarEnv},
    },
    memory::{PTls, gc::mark_queue_obj, scope::LocalScopeExt, target::unrooted::Unrooted},
};

static INNER_CACHE: FnvCache<TypeId, ValueUnbound> = new_fnv_cache();
pub(super) static CACHE: ConstructedTypes = ConstructedTypes::new(&INNER_CACHE);

pub(crate) unsafe fn mark_constructed_type_cache(ptls: PTls, full: bool) {
    unsafe {
        if full || INNER_CACHE.is_dirty() {
            INNER_CACHE.map(|value| {
                mark_queue_obj(ptls, value.as_weak());
            });
            INNER_CACHE.clear_dirty();
        }
    }
}

pub(super) struct ConstructedTypes<'a> {
    data: &'a FnvCache<TypeId, ValueUnbound>,
}

impl<'a> ConstructedTypes<'a> {
    pub(super) const fn new(data: &'a FnvCache<TypeId, ValueUnbound>) -> Self {
        ConstructedTypes { data }
    }

    #[inline]
    pub(super) fn find_or_construct<T: ConstructType>(&self) -> WeakValue<'static, 'static> {
        let tid = T::TYPE_ID;
        let res = self.data.get(&tid).map(|s| s.as_weak());

        if let Some(res) = res {
            return res;
        }

        do_construct::<T>(self, tid)
    }

    #[inline]
    pub(super) fn find_or_construct_with_env<T: ConstructType>(
        &self,
        env: &TypeVarEnv,
    ) -> WeakValue<'static, 'static> {
        let tid = T::TYPE_ID;
        let res = self.data.get(&tid).map(|s| s.as_weak());

        if let Some(res) = res {
            return res;
        }

        do_construct_with_context::<T>(self, tid, env)
    }
}

// #[inline(never)]
#[cold]
fn do_construct<T: ConstructType>(
    ct: &ConstructedTypes,
    tid: TypeId,
) -> WeakValue<'static, 'static> {
    unsafe {
        let unrooted = Unrooted::new();
        unrooted.with_local_scope::<_, 1>(|target, mut frame| {
            let ty = T::construct_type_uncached(&mut frame);

            if ty.is::<DataType>() {
                let dt = ty.cast_unchecked::<DataType>();
                if !dt.has_free_type_vars() && (!dt.is::<Tuple>() || dt.is_concrete_type()) {
                    ct.data.insert(tid, ty.leak().as_value());
                }
            } else if ty.is::<u8>() || ty.is::<i8>() {
                ct.data.insert(tid, ty.leak().as_value());
            }

            ty.root(target)
        })
    }
}

// #[inline(never)]
#[cold]
fn do_construct_with_context<T: ConstructType>(
    ct: &ConstructedTypes,
    tid: TypeId,
    env: &TypeVarEnv,
) -> WeakValue<'static, 'static> {
    unsafe {
        let unrooted = Unrooted::new();
        unrooted.with_local_scope::<_, 1>(|target, mut frame| {
            let ty = T::construct_type_with_env_uncached(&mut frame, env);

            if ty.is::<DataType>() {
                let dt = ty.cast_unchecked::<DataType>();
                if !dt.has_free_type_vars() && (!dt.is::<Tuple>() || dt.is_concrete_type()) {
                    ct.data.insert(tid, ty.leak().as_value());
                }
            } else if ty.is::<u8>() || ty.is::<i8>() {
                ct.data.insert(tid, ty.leak().as_value());
            }

            ty.root(target)
        })
    }
}
