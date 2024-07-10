//! Convert a Rust enum to a Julia enum.

use crate::{
    data::types::primitive_type::IntegerType,
    prelude::{Target, Value},
};

/// Trait to convert a Rust enum to a Julia enum.
///
/// This trait is automatically derived when bindings for enums are generated with
/// `JlrsCore.Reflect.reflect`.
///
/// Safety: the implementation must correctly map the Rust to the Julia enum.
pub unsafe trait Enum {
    /// The type that represents this enum.
    type Super: IntegerType;

    /// Convert `self` to a `Value`
    fn as_value<'target, Tgt: Target<'target>>(&self, _: &Tgt) -> Value<'target, 'static>;

    /// Convert `self` to its representation
    fn as_super(&self) -> Self::Super;
}
