//! Fully-typed layout.
//!
//! Layouts don't necessarily use all type parameters of a Julia type, for example when a
//! parameter is a value type or is only used by fields which have a mutable type. This means the
//! layout can't implement [`ConstructType`], which is necessary to convert that data into a
//! [`Value`].
//!
//! For this purpose [`TypedLayout`] and [`HasLayout`] exist. A `TypedLayout` wraps a layout and
//! a type constructor, `HasLayout` declares what layouts are associated with a type constructor.
//! This trait is automatically implemented when a type implements both [`ValidLayout`] and
//! [`ConstructType`].
//!
//! [`Value`]: crate::data::managed::value::Value

use std::{fmt, marker::PhantomData};

use super::{
    is_bits::IsBits,
    valid_layout::{ValidField, ValidLayout},
};
use crate::{
    convert::{
        ccall_types::{CCallArg, CCallReturn},
        unbox::Unbox,
    },
    data::types::{construct_type::ConstructType, typecheck::Typecheck},
    prelude::{DataType, LocalScope, Managed, Target, Value},
};

/// Associate a layout with a type constructor.
///
/// Safety:
///
/// `Layout` must be a valid layout for the type constructor.
pub unsafe trait HasLayout<'scope, 'data>: ConstructType {
    /// The layout associated with this type constructor.
    type Layout: ValidLayout;
}

unsafe impl<'scope, 'data, T: ConstructType + ValidLayout> HasLayout<'scope, 'data> for T {
    type Layout = T;
}

/// A layout annotated with its type constructor.
#[repr(transparent)]
pub struct TypedLayout<L, T> {
    data: L,
    _ty: PhantomData<T>,
}

impl<L, T> TypedLayout<L, T>
where
    L: ValidField,
    T: ConstructType,
{
    /// Convert `data` to a `TypedLayout` with an arbitrary type constructor `T`.
    ///
    /// Safety: `L` must be a valid layout for `T`.
    #[inline]
    pub const unsafe fn new_relaxed(data: L) -> Self {
        TypedLayout {
            data,
            _ty: PhantomData,
        }
    }

    /// Convert a typed layout to its layout.
    #[inline]
    pub fn into_layout(self) -> L {
        self.data
    }
}

impl<'scope, 'data, T> TypedLayout<T::Layout, T>
where
    T: HasLayout<'scope, 'data>,
{
    /// Convert `data` to a `TypedLayout`.
    #[inline]
    pub const fn new(data: T::Layout) -> Self {
        TypedLayout {
            data,
            _ty: PhantomData,
        }
    }
}

unsafe impl<L, T> IsBits for TypedLayout<L, T>
where
    L: IsBits + ValidLayout,
    T: 'static + HasLayout<'static, 'static, Layout = L>,
{
}

impl<L, T> Clone for TypedLayout<L, T>
where
    L: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            _ty: PhantomData,
        }
    }
}

impl<L, T> fmt::Debug for TypedLayout<L, T>
where
    L: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedLayout")
            .field("data", &self.data)
            .finish()
    }
}

unsafe impl<L, T> Typecheck for TypedLayout<L, T>
where
    L: Typecheck,
    T: ConstructType,
{
    fn typecheck(t: DataType) -> bool {
        t.unrooted_target().local_scope::<_, 1>(|mut frame| {
            let ty = T::construct_type(&mut frame);
            if ty != t {
                return false;
            }

            L::typecheck(t)
        })
    }
}

unsafe impl<L, T> ValidLayout for TypedLayout<L, T>
where
    L: ValidLayout,
    T: ConstructType,
{
    fn valid_layout(t: Value) -> bool {
        t.unrooted_target().local_scope::<_, 1>(|mut frame| {
            let ty = T::construct_type(&mut frame);
            if ty != t {
                return false;
            }

            L::valid_layout(t)
        })
    }

    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        L::type_object(target)
    }
}

unsafe impl<L, T> ValidField for TypedLayout<L, T>
where
    L: ValidField,
    T: ConstructType,
{
    fn valid_field(t: Value) -> bool {
        t.unrooted_target().local_scope::<_, 1>(|mut frame| {
            let ty = T::construct_type(&mut frame);
            if ty != t {
                return false;
            }

            L::valid_field(t)
        })
    }
}

unsafe impl<L, T> Unbox for TypedLayout<L, T>
where
    L: Clone,
    T: ConstructType,
{
    type Output = Self;
}

unsafe impl<L, T> ConstructType for TypedLayout<L, T>
where
    T: ConstructType,
{
    type Static = T::Static;

    #[inline]
    fn construct_type_uncached<'target, Tgt>(
        target: Tgt,
    ) -> crate::prelude::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        T::construct_type_uncached(target)
    }

    #[inline]
    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &crate::data::types::construct_type::TypeVarEnv,
    ) -> crate::prelude::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        T::construct_type_with_env_uncached(target, env)
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        T::base_type(target)
    }
}

unsafe impl<L, T> CCallArg for TypedLayout<L, T>
where
    L: IsBits,
    T: ConstructType,
{
    type CCallArgType = Self;
    type FunctionArgType = Self;
}

unsafe impl<L, T> CCallReturn for TypedLayout<L, T>
where
    L: IsBits,
    T: ConstructType,
{
    type FunctionReturnType = Self;
    type CCallReturnType = Self;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}
