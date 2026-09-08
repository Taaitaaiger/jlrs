use std::any::{Any, TypeId};
#[cfg(feature = "static-cache")]
use std::sync::atomic::Ordering;

#[cfg(feature = "static-cache")]
use static_generics::define_namespace;

#[cfg(feature = "static-cache")]
use crate::data::static_data::StaticConstRef;
#[cfg(not(feature = "static-cache"))]
use crate::memory::{scope::LocalScopeExt as _, target::unrooted::Unrooted};
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
    memory::{PTls, gc::mark_queue_obj, scope::LocalScope as _},
    weak_handle_unchecked,
};

static INNER_CACHE: FnvCache<TypeId, ValueUnbound> = new_fnv_cache();

#[cfg(not(feature = "static-cache"))]
pub(crate) static CACHE: ConstructedTypes = ConstructedTypes::new(&INNER_CACHE);

#[cfg(feature = "static-cache")]
pub(crate) static CACHE: ConstructedTypes = ConstructedTypes::new();

#[allow(unused_variables)]
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

#[cfg(feature = "static-cache")]
define_namespace!(ConstructedTypesNamespace);

#[cfg(not(feature = "static-cache"))]
pub(crate) struct ConstructedTypes<'a> {
    data: &'a FnvCache<TypeId, ValueUnbound>,
}

#[cfg(feature = "static-cache")]
pub(crate) struct ConstructedTypes {
    _priv: (),
}

#[cfg(feature = "static-cache")]
impl ConstructedTypes {
    pub(super) const fn new() -> Self {
        ConstructedTypes { _priv: () }
    }

    #[inline]
    pub(crate) fn insert_foreign<T: Any>(&self, ty: DataType) -> WeakValue<'static, 'static> {
        if let Some(res) = self.find_or_none::<T>() {
            return res;
        }

        StaticConstRef::<T, WeakValue>::store::<ConstructedTypesNamespace>(
            ty.as_value().leak(),
            Ordering::Relaxed,
        );

        let tid = std::any::TypeId::of::<T>();
        unsafe { INNER_CACHE.insert(tid, ty.leak().as_value()) };
        ty.as_value().leak()
    }

    pub(crate) fn find_or_none<T: Any>(&self) -> Option<WeakValue<'static, 'static>> {
        StaticConstRef::<T, WeakValue>::load::<ConstructedTypesNamespace>(Ordering::Relaxed)
    }

    #[inline]
    pub(crate) fn find_or_construct<T: ConstructType>(&self) -> WeakValue<'static, 'static> {
        if let Some(res) = self.find_or_none::<T::Static>() {
            return res;
        }

        do_construct::<T>()
    }

    #[inline]
    pub(crate) fn find_or_construct_with_env<T: ConstructType>(
        &self,
        env: &TypeVarEnv,
    ) -> WeakValue<'static, 'static> {
        if let Some(res) = self.find_or_none::<T::Static>() {
            return res;
        }

        do_construct_with_context::<T>(env)
    }
}

#[cfg(not(feature = "static-cache"))]
impl<'a> ConstructedTypes<'a> {
    pub(super) const fn new(data: &'a FnvCache<TypeId, ValueUnbound>) -> Self {
        ConstructedTypes { data }
    }

    #[inline]
    pub(crate) fn insert_foreign<T: Any>(&self, ty: DataType) -> WeakValue<'static, 'static> {
        if let Some(res) = self.find_or_none::<T>() {
            return res;
        }

        let tid = std::any::TypeId::of::<T>();
        unsafe { self.data.insert(tid, ty.leak().as_value()) };
        ty.as_value().leak()
    }

    pub(crate) fn find_or_none<T: Any>(&self) -> Option<WeakValue<'static, 'static>> {
        let tid = std::any::TypeId::of::<T>();
        self.data.get(&tid).map(|s| s.as_weak())
    }

    #[inline]
    pub(crate) fn find_or_construct<T: ConstructType>(&self) -> WeakValue<'static, 'static> {
        if let Some(res) = self.find_or_none::<T::Static>() {
            return res;
        }

        do_construct::<T>(self, T::TYPE_ID)
    }

    #[inline]
    pub(crate) fn find_or_construct_with_env<T: ConstructType>(
        &self,
        env: &TypeVarEnv,
    ) -> WeakValue<'static, 'static> {
        if let Some(res) = self.find_or_none::<T::Static>() {
            return res;
        }

        do_construct_with_context::<T>(self, T::TYPE_ID, env)
    }
}

#[inline(never)]
#[cfg(feature = "static-cache")]
#[cold]
fn do_construct<T: ConstructType>() -> WeakValue<'static, 'static> {
    unsafe {
        let handle = weak_handle_unchecked!();

        handle.local_scope::<_, 1>(|mut frame| {
            let ty = T::construct_type_uncached(&mut frame);
            let weak_ty = ty.leak();
            if T::CACHEABLE && ty.is::<DataType>() {
                let dt = ty.cast_unchecked::<DataType>();
                if !dt.has_free_type_vars() && (!dt.is::<Tuple>() || dt.is_concrete_type()) {
                    StaticConstRef::<T::Static, WeakValue>::store::<ConstructedTypesNamespace>(
                        weak_ty,
                        Ordering::Relaxed,
                    );
                }
            } else if ty.is::<u8>() || ty.is::<i8>() {
                StaticConstRef::<T::Static, WeakValue>::store::<ConstructedTypesNamespace>(
                    weak_ty,
                    Ordering::Relaxed,
                );
            }

            weak_ty
        })
    }
}

#[inline(never)]
#[cfg(feature = "static-cache")]
#[cold]
fn do_construct_with_context<T: ConstructType>(env: &TypeVarEnv) -> WeakValue<'static, 'static> {
    unsafe {
        let handle = weak_handle_unchecked!();

        handle.local_scope::<_, 1>(|mut frame| {
            let ty = T::construct_type_with_env_uncached(&mut frame, env);
            let weak_ty = ty.leak();
            if T::CACHEABLE && ty.is::<DataType>() {
                let dt = ty.cast_unchecked::<DataType>();
                if !dt.has_free_type_vars() && (!dt.is::<Tuple>() || dt.is_concrete_type()) {
                    INNER_CACHE.insert(T::TYPE_ID, weak_ty.as_managed());
                    StaticConstRef::<T::Static, WeakValue>::store::<ConstructedTypesNamespace>(
                        weak_ty,
                        Ordering::Relaxed,
                    );
                }
            } else if ty.is::<u8>() || ty.is::<i8>() {
                StaticConstRef::<T::Static, WeakValue>::store::<ConstructedTypesNamespace>(
                    weak_ty,
                    Ordering::Relaxed,
                );
            }

            weak_ty
        })
    }
}

#[cfg(not(feature = "static-cache"))]
#[inline(never)]
#[cold]
fn do_construct_with_context<T: ConstructType>(
    ct: &ConstructedTypes,
    tid: TypeId,
    env: &TypeVarEnv,
) -> WeakValue<'static, 'static> {
    unsafe {
        let handle = weak_handle_unchecked!();
        handle.local_scope::<_, 1>(|mut frame| {
            let ty = T::construct_type_with_env_uncached(&mut frame, env);

            if T::CACHEABLE && ty.is::<DataType>() {
                let dt = ty.cast_unchecked::<DataType>();
                if !dt.has_free_type_vars() && (!dt.is::<Tuple>() || dt.is_concrete_type()) {
                    ct.data.insert(tid, ty.leak().as_value());
                }
            } else if ty.is::<u8>() || ty.is::<i8>() {
                ct.data.insert(tid, ty.leak().as_value());
            }

            ty.leak()
        })
    }
}

#[cfg(not(feature = "static-cache"))]
#[inline(never)]
#[cold]
fn do_construct<T: ConstructType>(
    ct: &ConstructedTypes,
    tid: TypeId,
) -> WeakValue<'static, 'static> {
    unsafe {
        let unrooted = Unrooted::new();
        unrooted.with_local_scope::<_, 1>(|target, mut frame| {
            let ty = T::construct_type_uncached(&mut frame);

            if T::CACHEABLE && ty.is::<DataType>() {
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
