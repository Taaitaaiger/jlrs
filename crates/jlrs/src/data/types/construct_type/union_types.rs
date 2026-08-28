//! Type constructors for union types.

use std::{any::TypeId, marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_bottom_type, jl_uniontype_type, jl_value_t};
use rustc_hash::FxHashSet;

use crate::{
    data::{
        layout::{is_bits::IsBits, typed_layout::HasLayout},
        managed::{
            union::Union,
            value::{Value, ValueData},
        },
        types::construct_type::{ConstructType, TypeVarEnv},
    },
    memory::{scope::LocalScopeExt, target::Target},
    private::Private,
};

/// Convert a list of two or more types into a `UnionTypeConstructor` type.
#[macro_export]
macro_rules! UnionOf {
    [$l:ty, $r:ty] => {
        $crate::data::types::construct_type::union_types::UnionTypeConstructor<$l, $r>
    };
    [$l:ty, $($rest:ty),+] => {
        $crate::UnionOf![$l, $crate::UnionOf![$($rest),+]]
    };
}

/// Constructor for a `Union` type.
///
/// Larger unions can be built by nesting `UnionTypeConstructor`. Instead of manually writing down
/// the union of types, you should use the [`UnionOf`] macro.
pub struct UnionTypeConstructor<L: ConstructType, R: ConstructType> {
    _l: PhantomData<L>,
    _r: PhantomData<R>,
}

unsafe impl<L: ConstructType, R: ConstructType> ConstructType for UnionTypeConstructor<L, R> {
    type Static = UnionTypeConstructor<L::Static, R::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, 2>(|target, mut frame| {
            let l = L::construct_type(&mut frame);
            let r = R::construct_type(&mut frame);

            unsafe { crate::data::managed::union::Union::new_unchecked(target, [l, r]) }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let ptr = NonNull::new_unchecked(jl_uniontype_type.cast::<jl_value_t>());
            Some(
                <Value as crate::data::managed::private::ManagedPriv>::wrap_non_null(
                    ptr,
                    crate::private::Private,
                ),
            )
        }
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, 2>(|target, mut frame| {
            let l = L::construct_type_with_env(&mut frame, env);
            let r = R::construct_type_with_env(&mut frame, env);

            unsafe { Union::new_unchecked(target, [l, r]) }
        })
    }
}

/// Trait implemented by [`UnionTypeConstructor`] if all variants implement [`BitsUnionCtorVariant`].
pub trait BitsUnionCtor: BitsUnionCtorVariant {
    /// Returns the number of unique variants.
    fn n_unique_variants() -> usize {
        Self::get_variants().len()
    }

    /// Returns the set of type ids of all unique variants, and the number of variants.
    fn get_variants() -> FxHashSet<TypeId>;
}

/// Trait implemented by type constructors that have an `IsBits` layout, and unions of such types.
pub trait BitsUnionCtorVariant: ConstructType {
    const N: usize;
    #[doc(hidden)]
    fn add_variants(ids: &mut FxHashSet<TypeId>);
}

impl<'scope, 'data, T> BitsUnionCtorVariant for T
where
    T: ConstructType + HasLayout<'scope, 'data>,
    T::Layout: IsBits,
{
    const N: usize = 1;
    fn add_variants(ids: &mut FxHashSet<TypeId>) {
        ids.insert(Self::TYPE_ID);
    }
}

impl<L: BitsUnionCtorVariant, R: BitsUnionCtorVariant> BitsUnionCtorVariant
    for UnionTypeConstructor<L, R>
{
    const N: usize = L::N + R::N;
    fn add_variants(ids: &mut FxHashSet<TypeId>) {
        L::add_variants(ids);
        R::add_variants(ids);
    }
}

impl<L: BitsUnionCtorVariant, R: BitsUnionCtorVariant> BitsUnionCtor
    for UnionTypeConstructor<L, R>
{
    fn get_variants() -> FxHashSet<TypeId> {
        let mut set = FxHashSet::<TypeId>::default();

        L::add_variants(&mut set);
        R::add_variants(&mut set);

        set
    }
}

/// The bottom type, `Union{}`.
pub struct BottomType;

unsafe impl ConstructType for BottomType {
    type Static = BottomType;

    #[inline]
    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let ptr = NonNull::new_unchecked(jl_bottom_type.cast::<jl_value_t>());
            target.data_from_ptr(ptr, Private)
        }
    }

    #[inline]
    fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let ptr = NonNull::new_unchecked(jl_bottom_type.cast::<jl_value_t>());
            Some(
                <Value as crate::data::managed::private::ManagedPriv>::wrap_non_null(
                    ptr,
                    crate::private::Private,
                ),
            )
        }
    }

    #[inline]
    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let ptr = NonNull::new_unchecked(jl_bottom_type.cast::<jl_value_t>());
            target.data_from_ptr(ptr, Private)
        }
    }
}
