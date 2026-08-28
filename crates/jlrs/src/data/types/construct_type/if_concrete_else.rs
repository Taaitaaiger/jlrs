//! A type construction modifier that selects between two variants based on concreteness.
//!
//! A concrete type is a type that can have instances.

use std::marker::PhantomData;

use crate::{
    data::{
        managed::{
            datatype::DataType,
            value::{Value, ValueData},
        },
        types::construct_type::ConstructType,
    },
    memory::target::Target,
};

/// Returns the type constructed with the `ConstructType` implementation of `T1` if it is a
/// concrete type, otherwise that of `T2`.
pub struct IfConcreteElse<T1: ConstructType, T2: ConstructType> {
    _marker: PhantomData<(T1, T2)>,
}

unsafe impl<T1: ConstructType, T2: ConstructType> ConstructType for IfConcreteElse<T1, T2> {
    type Static = IfConcreteElse<T1::Static, T2::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let t1 = T1::construct_type(&target);
        unsafe {
            let v = t1.as_value();
            if v.is::<DataType>() {
                if v.cast_unchecked::<DataType>().is_concrete_type() {
                    return t1.root(target);
                }
            }

            T2::construct_type(target)
        }
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &crate::data::types::construct_type::TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let t1 = T1::construct_type_with_env_uncached(&target, env);
        unsafe {
            let v = t1.as_value();
            if v.is::<DataType>() {
                if v.cast_unchecked::<DataType>().is_concrete_type() {
                    return t1.root(target);
                }
            }

            T2::construct_type_with_env_uncached(target, env)
        }
    }

    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let t1 = T1::construct_type(&target);
        unsafe {
            let v = t1.as_value();
            if v.is::<DataType>() {
                if v.cast_unchecked::<DataType>().is_concrete_type() {
                    return T1::base_type(target);
                }
            }

            T2::base_type(target)
        }
    }
}
