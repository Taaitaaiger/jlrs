//! Construct Julia type objects from Rust types.

pub mod array;
pub mod bytes;
mod cache;
pub mod constants;
pub mod fast_key;
pub mod if_concrete_else;
pub mod primitives;
pub mod type_var;
pub mod union_types;

use std::any::TypeId;

pub(crate) use cache::mark_constructed_type_cache;
pub use type_var::TypeVarEnv;

use crate::{
    data::{
        managed::value::{Value, ValueData},
        types::construct_type::cache::CACHE,
    },
    memory::target::Target,
};

/// Associate a Julia type object with a Rust type.
///
/// Safety:
///
/// `ConstructType::construct_type` must either return a valid type object, or an instance of an
/// isbits type which is immediately used as a type parameter of another constructed type.
#[diagnostic::on_unimplemented(
    message = "the trait bound `{Self}: ConstructType` is not satisfied",
    label = "the trait `ConstructType` is not implemented for `{Self}`",
    note = "Custom types that implement `ConstructType` should be generated with JlrsCore.reflect",
    note = "Do not implement `ForeignType` or `OpaqueType` unless this type is exported to Julia with `julia_module!`"
)]

pub unsafe trait ConstructType: Sized {
    /// `Self`, but with all lifetimes set to `'static`. This ensures `Self::Static` has a type
    /// id.
    type Static: 'static + ConstructType;

    /// Indicates whether the type might be cacheable.
    ///
    /// If set to `false`, `construct_type` will never try to cache or look up the
    /// constructed type.
    const CACHEABLE: bool = true;

    /// The `TypeId` of `Self::Static`.
    const TYPE_ID: TypeId = TypeId::of::<Self::Static>();

    /// Construct the type object and try to cache the result. If a cached entry is available, it
    /// is returned.
    #[inline]
    fn construct_type<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        if Self::CACHEABLE {
            unsafe { CACHE.find_or_construct::<Self>().root(target) }
        } else {
            Self::construct_type_uncached(target)
        }
    }

    /// Constructs the type object associated with this type.
    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>;

    /// Construct the type object with an environment of `TypeVar`s and try to cache the result.
    /// If a cached entry is available, it is returned.
    ///
    /// No new type vars are constructed, if one is used and it don't already exist in `env`,
    /// this method panics. The result may have free `TypeVar`s, you can call
    /// [`DataType::wrap_with_env`] to create the appropriate `UnionAll`.
    /// 
    /// [`DataType::wrap_with_env`]: crate::data::managed::datatype::DataType::wrap_with_env
    #[inline]
    fn construct_type_with_env<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        if Self::CACHEABLE {
            unsafe { CACHE.find_or_construct_with_env::<Self>(env).root(target) }
        } else {
            Self::construct_type_with_env_uncached(target, env)
        }
    }

    /// Constructs the type object associated with this type.
    ///
    /// No new type vars are constructed, if one is used and it don't already exist in `env`,
    /// this method panics.
    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>;

    /// Returns the base type object associated with this type.
    ///
    /// The base type object is the type object without any types applied to it. If there is no
    /// such type object, e.g. when `Self` is a value type, `None` is returned. The base type must
    /// be globally rooted.
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>;
}
