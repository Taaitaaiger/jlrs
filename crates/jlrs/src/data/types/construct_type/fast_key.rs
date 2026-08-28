//! Fast type construction keys for types with parameters.

use std::marker::PhantomData;

use crate::{
    data::{
        managed::{
            Managed,
            array::{ConstructTypedArray, dimensions::DimsExt},
            datatype::DataType,
            value::{Value, ValueData},
        },
        types::{
            abstract_type::AbstractType,
            construct_type::{ConstructType, TypeVarEnv},
        },
    },
    memory::target::Target,
};

/// Define a fast key type for a constructible type.
///
/// See [`FastKey`] and [`FastArrayKey`] for examples.
#[macro_export]
macro_rules! define_fast_key {
    ($(#[$meta:meta])* $vis:vis $ty:ident, $for_ty:ty) => {
        $(#[$meta])*
        $vis struct $ty;

        unsafe impl $crate::data::types::construct_type::fast_key::FastKey for $ty {
            type For = $for_ty;

            #[inline]
            fn construct_type_fast<'target, Tgt>(
                target: &Tgt,
            ) -> $crate::data::managed::value::Value<'target, 'static>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                static REF: $crate::data::static_data::StaticConstructibleType<$for_ty> =
                    $crate::data::static_data::StaticConstructibleType::<$for_ty>::new();
                REF.get_or_init(&target)
            }
        }
    };
    ($(#[$meta:meta])* $vis:vis $ty:ident, $elem_ty:ty, $rank:literal) => {
        $(#[$meta])*
        $vis struct $ty;

        unsafe impl $crate::data::types::construct_type::fast_key::FastKey for $ty {
            type For = $crate::data::managed::array::TypedRankedArray<
                'static,
                'static,
                <$elem_ty as $crate::data::types::construct_type::ConstructType>::Static,
                $rank
            >;

            #[inline]
            fn construct_type_fast<'target, Tgt>(
                target: &Tgt,
            ) -> $crate::data::managed::value::Value<'target, 'static>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                type Ty = $crate::data::types::construct_type::array::RankedArrayType<$elem_ty, $rank>;
                static REF: $crate::data::static_data::StaticConstructibleType<Ty> =
                    $crate::data::static_data::StaticConstructibleType::<Ty>::new();
                REF.get_or_init(&target)
            }
        }

        unsafe impl $crate::data::types::construct_type::fast_key::FastArrayKey<$rank> for $ty {
            type ElemType = $elem_ty;
        }
    };
}

/// Cache a single constructible type.
///
/// By default, constructible types with type parameters are significantly slower to access than
/// types without type parameters. The reason is that types without type parameters can be cached
/// in a local static variable, while types with type parameters are stored in a global hash map.
///
/// It's not possible to generally cache types with type parameters in a local static variable,
/// but specific types can be cached by implementing this trait for a new type.
///
/// Safety: the constructed type must not have any free type parameters, [`define_fast_key`]
/// should be used to create the type and implementation.
///
/// Example:
///
/// ```
/// # use jlrs::prelude::*;
/// use jlrs::data::types::{abstract_type::AbstractSet, construct_type::{ConstructType, fast_key::Key}};
///
/// define_fast_key!(pub AbstractSetF64, AbstractSet<f64>);
///
/// # fn main() {
/// # let mut julia = Builder::new().start_local().unwrap();
///
/// julia.local_scope::<_, 2>(|mut frame| {
///     let ty = Key::<AbstractSetF64>::construct_type(&mut frame);
///     let ty2 = AbstractSet::<f64>::construct_type(&mut frame);
///     assert_eq!(ty, ty2);
/// });
/// # }
/// ```
pub unsafe trait FastKey: 'static {
    type For: ConstructType;
    fn construct_type_fast<'target, Tgt>(target: &Tgt) -> Value<'target, 'static>
    where
        Tgt: Target<'target>;
}

/// Cache a single constructible array type.
///
/// Similar to [`FastKey`] but specifically intended to be used with array types. The implementation
/// must not be generic over `N`, but must be set to a specific, non-negative value.
///
/// All implementations of `FastArrayKey` implement [`ConstructTypedArray`].
///
/// Safety: the constructed type must not have any free type parameters and must be an array type,
/// [`define_fast_key!`] should be used to create the type and implementation.
///
/// Example:
///
/// ```
/// # use jlrs::prelude::*;
/// use jlrs::data::types::construct_type::{ConstructType, fast_key::Key};
///
/// define_fast_key!(pub VecF32, f32, 1);
///
/// # fn main() {
/// # let mut julia = Builder::new().start_local().unwrap();
///
/// julia.local_scope::<_, 3>(|mut frame| {
///     let ty = Key::<VecF32>::construct_type(&mut frame);
///     let ty2 = TypedRankedArray::<f32, 1>::construct_type(&mut frame);
///     assert_eq!(ty, ty2);
///
///     let v = VecF32::new(&mut frame, 4);
///     assert!(v.is_ok());
/// });
/// # }
/// ```
pub unsafe trait FastArrayKey<const N: isize>: 'static + FastKey {
    /// The element type of this array type.
    type ElemType: ConstructType;

    /// Assert that `Self::RANK` is non-negative.
    const ASSERT_VALID_RANK: () = assert!(N >= 0, "Array rank must be known at compile time");
}

impl<T: FastArrayKey<N>, const N: isize> ConstructTypedArray<T::ElemType, N> for T {
    #[inline]
    fn array_type<'target, D, Tgt>(target: Tgt, _dims: &D) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
        D: DimsExt,
    {
        let _ = Self::ASSERT_VALID_RANK;
        Key::<T>::construct_type(target)
    }
}

/// Type used to expose [`ConstructType`] for types that implement [`FastKey`].
///
/// See [`FastKey`] and [`FastArrayKey`] for examples.
pub struct Key<K: FastKey>(PhantomData<K>);

unsafe impl<K: FastKey> ConstructType for Key<K> {
    type Static = <K::For as ConstructType>::Static;
    const CACHEABLE: bool = false;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        K::construct_type_fast(&target).root(target)
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        K::construct_type_fast(&target).root(target)
    }

    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let v = K::construct_type_fast(target);
            if v.is::<DataType>() {
                let dt = v.cast_unchecked::<DataType>();
                Some(dt.type_name().wrapper())
            } else {
                Some(v)
            }
        }
    }
}

unsafe impl<K> AbstractType for Key<K>
where
    K: FastKey,
    K::For: AbstractType,
{
}
