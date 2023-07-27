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
//! [`ConstructType`],
//!
//! [`Value`]: crate::data::managed::value::Value

use std::marker::PhantomData;

use super::{is_bits::IsBits, valid_layout::ValidLayout};
use crate::data::types::construct_type::ConstructType;

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
pub struct TypedLayout<L: ValidLayout, T: ConstructType> {
    data: L,
    _ty: PhantomData<T>,
}

impl<L, T> TypedLayout<L, T>
where
    L: ValidLayout,
    T: ConstructType,
{
    /// Convert `data` to a `TypedLayout` with an arbitrary type constructor `T`.
    #[inline]
    pub const fn new_relaxed(data: L) -> Self {
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

unsafe impl<L: IsBits + ValidLayout, T: 'static + HasLayout<'static, 'static, Layout = L>> IsBits
    for TypedLayout<L, T>
{
}

// TODO: other traits
