//! Abstract Julia types
//!
//! These types can be used with the `ConstructType` trait.

use std::marker::PhantomData;

use jlrs_macros::julia_version;

use super::construct_type::{ConstructType, TypeVarEnv};
use crate::{
    data::managed::{
        datatype::DataType,
        type_var::TypeVar,
        union_all::UnionAll,
        value::{Value, ValueData},
        Managed,
    },
    inline_static_ref,
    memory::target::Target,
};

/// Marker trait that the constructed type is an abstract type.
///
/// Safety: must only be implemented if the constructed type is an abstract type.
pub unsafe trait AbstractType: ConstructType {}

macro_rules! impl_construct_julia_type_abstract {
    ($ty:ty, $path:expr) => {
        unsafe impl ConstructType for $ty {
            type Static = $ty;

            const CACHEABLE: bool = false;

            #[inline]
            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> ValueData<'target, 'static, Tgt>
            where
                Tgt: Target<'target>,
            {
                Self::base_type(&target).unwrap().root(target)
            }

            #[inline]
            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                _: &TypeVarEnv,
            ) -> ValueData<'target, 'static, Tgt>
            where
                Tgt: Target<'target>,
            {
                Self::base_type(&target).unwrap().root(target)
            }

            #[inline]
            fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
            where
                Tgt: Target<'target>,
            {
                let value = inline_static_ref!(STATIC, Value, $path, target);
                Some(value)
            }
        }
    };
}

macro_rules! impl_construct_julia_type_abstract_using {
    ($ty:ty, $path:expr) => {
        unsafe impl ConstructType for $ty {
            type Static = $ty;

            const CACHEABLE: bool = false;

            #[inline]
            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> ValueData<'target, 'static, Tgt>
            where
                Tgt: Target<'target>,
            {
                Self::base_type(&target).unwrap().root(target)
            }

            #[inline]
            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                _: &TypeVarEnv,
            ) -> ValueData<'target, 'static, Tgt>
            where
                Tgt: Target<'target>,
            {
                Self::base_type(&target).unwrap().root(target)
            }

            #[inline]
            fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
            where
                Tgt: Target<'target>,
            {
                let value = $path(target).as_value();
                Some(value)
            }
        }
    };
}

/// Construct a new `Core.AbstractChar` type object.
pub struct AbstractChar;
unsafe impl AbstractType for AbstractChar {}
impl_construct_julia_type_abstract!(AbstractChar, "Core.AbstractChar");

/// Construct a new `Core.AbstractFloat` type object.
pub struct AbstractFloat;
unsafe impl AbstractType for AbstractFloat {}
impl_construct_julia_type_abstract_using!(AbstractFloat, DataType::floatingpoint_type);

/// Construct a new `Core.AbstractString` type object.
pub struct AbstractString;
unsafe impl AbstractType for AbstractString {}
impl_construct_julia_type_abstract!(AbstractString, "Core.AbstractString");

/// Construct a new `Core.Exception` type object.
pub struct Exception;
unsafe impl AbstractType for Exception {}
impl_construct_julia_type_abstract!(Exception, "Core.Exception");

/// Construct a new `Core.IO` type object.
pub struct IO;
unsafe impl AbstractType for IO {}
impl_construct_julia_type_abstract!(IO, "Core.IO");

/// Construct a new `Core.Integer` type object.
pub struct Integer;
unsafe impl AbstractType for Integer {}
impl_construct_julia_type_abstract!(Integer, "Core.Integer");

/// Construct a new `Core.Real` type object.
pub struct Real;
unsafe impl AbstractType for Real {}
impl_construct_julia_type_abstract!(Real, "Core.Real");

/// Construct a new `Core.Number` type object.
pub struct Number;
unsafe impl AbstractType for Number {}
impl_construct_julia_type_abstract_using!(Number, DataType::number_type);

/// Construct a new `Core.Signed` type object.
pub struct Signed;
unsafe impl AbstractType for Signed {}
impl_construct_julia_type_abstract_using!(Signed, DataType::signed_type);

/// Construct a new `Core.Unsigned` type object.
pub struct Unsigned;
unsafe impl AbstractType for Unsigned {}
impl_construct_julia_type_abstract!(Unsigned, "Core.Unsigned");

/// Construct a new `Base.AbstractDisplay` type object.
pub struct AbstractDisplay;
unsafe impl AbstractType for AbstractDisplay {}
impl_construct_julia_type_abstract!(AbstractDisplay, "Base.AbstractDisplay");

/// Construct a new `Base.AbstractIrrational` type object.
pub struct AbstractIrrational;
unsafe impl AbstractType for AbstractIrrational {}
impl_construct_julia_type_abstract!(AbstractIrrational, "Base.AbstractIrrational");

/// Construct a new `Base.AbstractMatch` type object.
pub struct AbstractMatch;
unsafe impl AbstractType for AbstractMatch {}
impl_construct_julia_type_abstract!(AbstractMatch, "Base.AbstractMatch");

/// Construct a new `Base.AbstractPattern` type object.
pub struct AbstractPattern;
unsafe impl AbstractType for AbstractPattern {}
impl_construct_julia_type_abstract!(AbstractPattern, "Base.AbstractPattern");

/// Construct a new `Base.IndexStyle` type object.
pub struct IndexStyle;
unsafe impl AbstractType for IndexStyle {}
impl_construct_julia_type_abstract!(IndexStyle, "Base.IndexStyle");

/// Construct a new `Core.Signed` type object.
pub struct AnyType;
unsafe impl AbstractType for AnyType {}
impl_construct_julia_type_abstract_using!(AnyType, DataType::any_type);

/// Construct a new `AbstractArray` type object from the provided type parameters.
pub struct AbstractArray<T: ConstructType, N: ConstructType> {
    _type: PhantomData<T>,
    _rank: PhantomData<N>,
}

unsafe impl<T: ConstructType, N: ConstructType> AbstractType for AbstractArray<T, N> {}

unsafe impl<T: ConstructType, N: ConstructType> ConstructType for AbstractArray<T, N> {
    type Static = AbstractArray<T::Static, N::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let rank_param = N::construct_type(&mut frame);
            let params = [ty_param, rank_param];
            unsafe {
                UnionAll::abstractarray_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(UnionAll::abstractarray_type(target).as_value())
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let rank_param = N::construct_type_with_env(&mut frame, env);
            let params = [ty_param, rank_param];
            unsafe {
                UnionAll::abstractarray_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }
}

/// Construct a new `DenseArray` type object from the provided type parameters.
pub struct DenseArray<T: ConstructType, N: ConstructType> {
    _type: PhantomData<T>,
    _rank: PhantomData<N>,
}

unsafe impl<T: ConstructType, N: ConstructType> AbstractType for DenseArray<T, N> {}

unsafe impl<T: ConstructType, N: ConstructType> ConstructType for DenseArray<T, N> {
    type Static = DenseArray<T::Static, N::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let rank_param = N::construct_type(&mut frame);
            let params = [ty_param, rank_param];
            unsafe {
                UnionAll::densearray_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(UnionAll::densearray_type(target).as_value())
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let rank_param = N::construct_type_with_env(&mut frame, env);
            let params = [ty_param, rank_param];
            unsafe {
                UnionAll::densearray_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }
}

/// Construct a new `Ref` type object from the provided type parameters.
pub struct RefTypeConstructor<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for RefTypeConstructor<T> {}

unsafe impl<T: ConstructType> ConstructType for RefTypeConstructor<T> {
    type Static = RefTypeConstructor<T::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let params = [ty_param];
            unsafe {
                UnionAll::ref_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(UnionAll::ref_type(target).as_value())
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let params = [ty_param];
            unsafe {
                UnionAll::ref_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }
}

/// Construct a new `Type` type object from the provided type parameters.
pub struct TypeTypeConstructor<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for TypeTypeConstructor<T> {}

unsafe impl<T: ConstructType> ConstructType for TypeTypeConstructor<T> {
    type Static = TypeTypeConstructor<T::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let params = [ty_param];
            unsafe {
                UnionAll::type_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(UnionAll::type_type(target).as_value())
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let params = [ty_param];
            unsafe {
                UnionAll::type_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }
}

/// Construct a new `AbstractChannel` type object from the provided type parameters.
pub struct AbstractChannel<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for AbstractChannel<T> {}

unsafe impl<T: ConstructType> ConstructType for AbstractChannel<T> {
    type Static = AbstractChannel<T::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.AbstractChannel", target);
        Some(value)
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }
}

/// Construct a new `AbstractDict` type object from the provided type parameters.
pub struct AbstractDict<K: ConstructType, V: ConstructType> {
    _key: PhantomData<K>,
    _value: PhantomData<V>,
}

unsafe impl<T: ConstructType, K: ConstructType> AbstractType for AbstractDict<T, K> {}

unsafe impl<K: ConstructType, V: ConstructType> ConstructType for AbstractDict<K, V> {
    type Static = AbstractDict<K::Static, V::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let key_param = K::construct_type(&mut frame);
            let value_param = V::construct_type(&mut frame);
            let params = [key_param, value_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.AbstractDict", target);
        Some(value)
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let key_param = K::construct_type_with_env(&mut frame, env);
            let value_param = V::construct_type_with_env(&mut frame, env);
            let params = [key_param, value_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }
}

/// Construct a new `AbstractMatrix` type object from the provided type parameters.
pub struct AbstractMatrix<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for AbstractMatrix<T> {}

unsafe impl<T: ConstructType> ConstructType for AbstractMatrix<T> {
    type Static = AbstractMatrix<T::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.AbstractMatrix", target);
        Some(value)
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }
}

/// Construct a new `AbstractRange` type object from the provided type parameters.
pub struct AbstractRange<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for AbstractRange<T> {}

unsafe impl<T: ConstructType> ConstructType for AbstractRange<T> {
    type Static = AbstractRange<T::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.AbstractRange", target);
        Some(value)
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }
}

/// Construct a new `AbstractSet` type object from the provided type parameters.
pub struct AbstractSet<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for AbstractSet<T> {}

unsafe impl<T: ConstructType> ConstructType for AbstractSet<T> {
    type Static = AbstractSet<T::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.AbstractSet", target);
        Some(value)
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }
}

#[julia_version(since = "1.9")]
/// Construct a new `AbstractSlices` type object from the provided type parameters.
pub struct AbstractSlices<T: ConstructType, N: ConstructType> {
    _type: PhantomData<T>,
    _n: PhantomData<N>,
}

#[julia_version(since = "1.9")]
unsafe impl<T: ConstructType, N: ConstructType> AbstractType for AbstractSlices<T, N> {}

#[julia_version(since = "1.9")]
unsafe impl<T: ConstructType, N: ConstructType> ConstructType for AbstractSlices<T, N> {
    type Static = AbstractSlices<T::Static, N::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let n_param = N::construct_type(&mut frame);
            let params = [ty_param, n_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let n_param = N::construct_type_with_env(&mut frame, env);
            let params = [ty_param, n_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.AbstractSlices", target);
        Some(value)
    }
}

/// Construct a new `AbstractUnitRange` type object from the provided type parameters.
pub struct AbstractUnitRange<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> ConstructType for AbstractUnitRange<T> {
    type Static = AbstractUnitRange<T::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.AbstractUnitRange", target);
        Some(value)
    }
}

unsafe impl<T: ConstructType> AbstractType for AbstractUnitRange<T> {}

/// Construct a new `AbstractVector` type object from the provided type parameters.
pub struct AbstractVector<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for AbstractVector<T> {}

unsafe impl<T: ConstructType> ConstructType for AbstractVector<T> {
    type Static = AbstractVector<T::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.AbstractVector", target);
        Some(value)
    }
}

/// Construct a new `DenseMatrix` type object from the provided type parameters.
pub struct DenseMatrix<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for DenseMatrix<T> {}

unsafe impl<T: ConstructType> ConstructType for DenseMatrix<T> {
    type Static = DenseMatrix<T::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.DenseMatrix", target);
        Some(value)
    }
}

/// Construct a new `DenseVector` type object from the provided type parameters.
pub struct DenseVector<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for DenseVector<T> {}

unsafe impl<T: ConstructType> ConstructType for DenseVector<T> {
    type Static = DenseVector<T::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let params = [ty_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.DenseVector", target);
        Some(value)
    }
}

/// Construct a new `Enum` type object from the provided type parameters.
pub struct Enum<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for Enum<T> {}

unsafe impl<T: ConstructType> ConstructType for Enum<T> {
    type Static = Enum<T::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);

            // Validate bound
            if let Ok(tvar) = ty_param.cast::<TypeVar>() {
                unsafe {
                    let ub = tvar.upper_bound(&frame).as_value();
                    assert!(ub.subtype(Integer::construct_type(&mut frame)));
                }
            } else {
                assert!(ty_param.subtype(Integer::construct_type(&mut frame)));
            }

            let params = [ty_param];

            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);

            // Validate bound
            if let Ok(tvar) = ty_param.cast::<TypeVar>() {
                unsafe {
                    let ub = tvar.upper_bound(&frame).as_value();
                    assert!(ub.subtype(Integer::construct_type_with_env(&mut frame, env)));
                }
            } else {
                assert!(ty_param.subtype(Integer::construct_type_with_env(&mut frame, env)));
            }

            let params = [ty_param];

            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.Enum", target);
        Some(value)
    }
}

/// Construct a new `OrdinalRange` type object from the provided type parameters.
pub struct OrdinalRange<T: ConstructType, S: ConstructType> {
    _type: PhantomData<T>,
    _s: PhantomData<S>,
}

unsafe impl<T: ConstructType, S: ConstructType> AbstractType for OrdinalRange<T, S> {}

unsafe impl<T: ConstructType, S: ConstructType> ConstructType for OrdinalRange<T, S> {
    type Static = OrdinalRange<T::Static, S::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let ty_param = T::construct_type(&mut frame);
            let n_param = S::construct_type(&mut frame);
            let params = [ty_param, n_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            }
        })
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 3>(|target, mut frame| {
            let ty_param = T::construct_type_with_env(&mut frame, env);
            let n_param = S::construct_type_with_env(&mut frame, env);
            let params = [ty_param, n_param];
            unsafe {
                Self::base_type(&frame)
                    .unwrap()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let value = inline_static_ref!(STATIC, Value, "Base.OrdinalRange", target);
        Some(value)
    }
}

/*
function print_vars(ty)
    while ty isa UnionAll
        println("  ", ty.var)
        ty = ty.body
    end
end

function list_abstract_type()
    for name in names(Core)
        item = getglobal(Core, name)
        if isabstracttype(item)
            println("Core.", name)
            if item isa UnionAll
                print_vars(item)
            end
        end
    end

    for name in names(Base)
        item = getglobal(Base, name)
        if isabstracttype(item)
            println("Base.", name)
            if item isa UnionAll
                print_vars(item)
            end
        end
    end
end


Abstract DataTypes:

///Core.AbstractChar
///Core.AbstractFloat
///Core.AbstractString
///Core.Any
///Core.Exception
///Core.Function
///Core.IO
///Core.Integer
///Core.Number
///Core.Real
///Core.Signed
///Core.Unsigned

///Base.AbstractDisplay
///Base.AbstractIrrational
///Base.AbstractMatch
///Base.AbstractPattern
///Base.IndexStyle
///
///
///Abstract UnionAlls:
///
///Core.AbstractArray{T,N}
///Core.DenseArray{T,N}
///Core.Ref{T}
///Core.Type{T}
///
///Base.AbstractChannel{T}
///Base.AbstractDict{K, V}
///Base.AbstractMatrix{T}
///Base.AbstractRange{T}
///Base.AbstractSet{T}
///Base.AbstractSlices{T,N}
///Base.AbstractUnitRange{T}
///Base.AbstractVector{T}
///Base.DenseMatrix{T}
///Base.DenseVector{T}
///Base.Enum{T<:Integer}
///Base.OrdinalRange{T,S}
///
/// TODO:
/// Base.AbstractLock
/// Base.IteratorSize
*/
