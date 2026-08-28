//! Construct constant Julia values from Rust types.

use crate::{
    data::{
        managed::value::{Value, ValueData},
        types::construct_type::{ConstructType, TypeVarEnv},
    },
    memory::target::Target,
};

macro_rules! impl_construct_julia_type_constant {
    ($ty:ty, $const_ty:ty) => {
        unsafe impl<const N: $const_ty> $crate::data::types::construct_type::ConstructType for $ty {
            type Static = $ty;

            const CACHEABLE: bool = false;

            #[inline]
            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                $crate::data::managed::value::Value::new(target, N)
            }

            #[inline]
            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                _: &$crate::data::types::construct_type::type_var::TypeVarEnv,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                $crate::data::managed::value::Value::new(target, N)
            }

            #[inline]
            fn base_type<'target, Tgt>(
                _target: &Tgt,
            ) -> Option<$crate::data::managed::value::Value<'target, 'static>>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                None
            }
        }
    };
}

macro_rules! impl_construct_julia_type_constant_cached {
    ($ty:ty, $const_ty:ty) => {
        unsafe impl<const N: $const_ty> $crate::data::types::construct_type::ConstructType for $ty {
            type Static = $ty;

            const CACHEABLE: bool = true;

            #[inline]
            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                $crate::data::managed::value::Value::new(target, N)
            }

            #[inline]
            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                _: &$crate::data::types::construct_type::type_var::TypeVarEnv,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                $crate::data::managed::value::Value::new(target, N)
            }

            #[inline]
            fn base_type<'target, Tgt>(
                _target: &Tgt,
            ) -> Option<$crate::data::managed::value::Value<'target, 'static>>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                None
            }
        }
    };
}

/// Constant `i8`.
pub struct ConstantI8<const N: i8>;
impl_construct_julia_type_constant_cached!(ConstantI8<N>, i8);

/// Constant `i16`.
pub struct ConstantI16<const N: i16>;
impl_construct_julia_type_constant!(ConstantI16<N>, i16);

/// Constant `i32`.
pub struct ConstantI32<const N: i32>;
impl_construct_julia_type_constant!(ConstantI32<N>, i32);

/// Constant `i64`.
pub struct ConstantI64<const N: i64>;
impl_construct_julia_type_constant!(ConstantI64<N>, i64);

/// Constant `isize`.
pub struct ConstantIsize<const N: isize>;
impl_construct_julia_type_constant!(ConstantIsize<N>, isize);

/// Constant `isize`.
pub struct ConstantSize<const N: usize>;
unsafe impl<const N: usize> ConstructType for ConstantSize<N> {
    type Static = ConstantSize<N>;

    const CACHEABLE: bool = false;

    #[inline]
    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        Value::new(target, N as isize)
    }

    #[inline]
    fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        None
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        Value::new(target, N as isize)
    }
}

/// Constant `u8`.
pub struct ConstantU8<const N: u8>;
impl_construct_julia_type_constant_cached!(ConstantU8<N>, u8);

/// Constant `u16`.
pub struct ConstantU16<const N: u16>;
impl_construct_julia_type_constant!(ConstantU16<N>, u16);

/// Constant `u32`.
pub struct ConstantU32<const N: u32>;
impl_construct_julia_type_constant!(ConstantU32<N>, u32);

/// Constant `u64`.
pub struct ConstantU64<const N: u64>;
impl_construct_julia_type_constant!(ConstantU64<N>, u64);

/// Constant `usize`.
pub struct ConstantUsize<const N: usize>;
impl_construct_julia_type_constant!(ConstantUsize<N>, usize);

/// Constant `bool`.
pub struct ConstantBool<const N: bool>;
impl_construct_julia_type_constant!(ConstantBool<N>, bool);

/// Constant `char`.
pub struct ConstantChar<const N: char>;
impl_construct_julia_type_constant!(ConstantChar<N>, char);
