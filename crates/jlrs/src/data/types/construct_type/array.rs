//! Specialized type constructors for array types.

use std::marker::PhantomData;

use crate::{
    data::{
        managed::{
            Managed,
            datatype::DataType,
            union_all::UnionAll,
            value::{Value, ValueData},
        },
        types::construct_type::{ConstructType, TypeVarEnv, constants::ConstantIsize},
    },
    memory::{scope::LocalScopeExt, target::Target},
};

/// Construct a new `Array` type from the provided type parameters.
///
/// `T` is the type constructor for the element type, `N` for the rank type. `N` must either be a
/// non-negative `ConstantIsize`, or a `TypeVar`.
pub struct ArrayTypeConstructor<T: ConstructType, N: ConstructType> {
    _type: PhantomData<T>,
    _rank: PhantomData<N>,
}

unsafe impl<T: ConstructType, N: ConstructType> ConstructType for ArrayTypeConstructor<T, N> {
    type Static = ArrayTypeConstructor<T::Static, N::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            target.with_local_scope::<_, 3>(|target, mut frame| {
                let ty_param = T::construct_type(&mut frame);
                let rank_param = N::construct_type(&mut frame);
                if rank_param.is::<isize>() {
                    if rank_param.unbox_unchecked::<isize>() < 0 {
                        panic!("ArrayTypeConstructor rank must be a TypeVar or non-negative ConstantIsize, got {rank_param:?}")
                    }
                }
                let params = [ty_param, rank_param];
                Self::base_type(&frame)
                    .unwrap_unchecked()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            })
        }
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let wrapper = UnionAll::array_type(target).as_value();
        Some(wrapper)
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            target.with_local_scope::<_, 3>(|target, mut frame| {
                let ty_param = T::construct_type_with_env(&mut frame, env);
                let rank_param = N::construct_type_with_env(&mut frame, env);
                if rank_param.is::<isize>() {
                    if rank_param.unbox_unchecked::<isize>() < 0 {
                        panic!("ArrayTypeConstructor rank must be a TypeVar or non-negative ConstantIsize, got {rank_param:?}")
                    }
                }
                let params = [ty_param, rank_param];
                Self::base_type(&frame)
                    .unwrap_unchecked()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            })
        }
    }
}

/// Alias for `ArrayTypeConstructor<T, ConstantIsize<N>>`.
pub type RankedArrayType<T, const N: isize> = ArrayTypeConstructor<T, ConstantIsize<N>>;
