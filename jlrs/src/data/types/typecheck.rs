//! Trait for checking properties of Julia data.
//!
//! Several properties of Julia data can be checked by using [`Value::is`] and [`DataType::is`],
//! these methods must be used in combination with a type that implements the [`Typecheck`] trait.
//! Most types that implement this trait also implement [`Managed`] or [`Unbox`], for these types
//! the typecheck indicates whether or not it's valid to cast the value to or unbox it as that
//! type.
//!
//! [`Value::is`]: crate::data::managed::value::Value::is
//! [`Managed`]: crate::data::managed::Managed
//! [`Unbox`]: crate::convert::unbox::Unbox
use std::{ffi::c_void, marker::PhantomData};

// TODO: Unify with other ConstructType and abstract types?
use jl_sys::jl_string_type;

use super::abstract_type::AbstractType;
use crate::{
    convert::into_julia::IntoJulia,
    data::managed::{datatype::DataType, type_name::TypeName, union_all::UnionAll, Managed},
    memory::target::unrooted::Unrooted,
    prelude::LocalScope,
};

/// This trait is used in combination with [`Value::is`] and [`DataType::is`] to check if that
/// property holds true.
///
/// Safety: If this trait is implemented for some type which also implements `Unbox`, the trait
/// method `typecheck` must only return `true` if it's guaranteed that `Unbox::unbox` can safely
/// be called for values whose type is that method's argument.
///
/// [`Value::is`]: crate::data::managed::value::Value::is
/// [`Unbox`]: crate::convert::unbox::Unbox
/// [`Managed`]: crate::data::managed::Managed
#[cfg_attr(
    feature = "diagnostics",
    diagnostic::on_unimplemented(
        message = "the trait bound `{Self}: Typecheck` is not satisfied",
        label = "the trait `Typecheck` is not implemented for `{Self}`",
        note = "Custom types that implement `Typecheck` should be generated with JlrsCore.reflect",
        note = "Do not implement `ForeignType`, `OpaqueType`, or `ParametricVariant` unless this type is exported to Julia with `julia_module!`"
    )
)]
pub unsafe trait Typecheck {
    /// Returns whether the property implied by `Self` holds true.
    fn typecheck(t: DataType) -> bool;
}

/// Type that implements [`Typecheck`] for every [`AbstractType`] `A`.
///
/// The typecheck returns `true` if the type is a subtype of `A`.
pub struct AbstractTypecheck<A: AbstractType>(PhantomData<A>);

unsafe impl<A: AbstractType> Typecheck for AbstractTypecheck<A> {
    fn typecheck(t: DataType) -> bool {
        t.unrooted_target().local_scope::<_, 1>(|mut frame| {
            let ty = A::construct_type(&mut frame);
            t.as_value().subtype(ty)
        })
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_julia_typecheck {
    ($type:ty, $jl_type:expr, $($lt:lifetime),+) => {
        unsafe impl<$($lt),+> crate::data::types::typecheck::Typecheck for $type {
            #[inline]
            fn typecheck(t: $crate::data::managed::datatype::DataType) -> bool {
                unsafe {
                    <$crate::data::managed::datatype::DataType as $crate::data::managed::private::ManagedPriv>::unwrap(t, crate::private::Private) == $jl_type
                }
            }
        }
    };
    ($type:ty, $jl_type:expr) => {
        unsafe impl crate::data::types::typecheck::Typecheck for $type {
            #[inline]
            fn typecheck(t: $crate::data::managed::datatype::DataType) -> bool {
                unsafe {
                    <$crate::data::managed::datatype::DataType as $crate::data::managed::private::ManagedPriv>::unwrap(t, crate::private::Private) == $jl_type
                }
            }
        }
    };
    ($type:ty) => {
        unsafe impl crate::data::types::typecheck::Typecheck for $type {
            #[inline]
            fn typecheck(t: crate::data::managed::datatype::DataType) -> bool {
                unsafe {
                    let global = $crate::memory::target::unrooted::Unrooted::new();
                    <$crate::data::managed::datatype::DataType as $crate::data::managed::private::ManagedPriv>::unwrap(t, crate::private::Private) == <$type as $crate::convert::into_julia::IntoJulia>::julia_type(global).ptr().as_ptr()
                }
            }
        }
    };
}

impl_julia_typecheck!(i8);
impl_julia_typecheck!(i16);
impl_julia_typecheck!(i32);
impl_julia_typecheck!(i64);
impl_julia_typecheck!(isize);
impl_julia_typecheck!(u8);
impl_julia_typecheck!(u16);
impl_julia_typecheck!(u32);
impl_julia_typecheck!(u64);
impl_julia_typecheck!(usize);
impl_julia_typecheck!(f32);
impl_julia_typecheck!(f64);
impl_julia_typecheck!(bool);
impl_julia_typecheck!(char);
impl_julia_typecheck!(*mut c_void);

unsafe impl<T: IntoJulia> Typecheck for *mut T {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        unsafe {
            let global = Unrooted::new();
            let ptr_tname = TypeName::of_pointer(&global);

            if t.type_name() != ptr_tname {
                return false;
            }

            let params = t.parameters();
            let param = params.data().get(global, 0);
            let inner_ty = T::julia_type(global);
            if param.unwrap_unchecked().as_value() != inner_ty.as_value() {
                return false;
            }

            true
        }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the `DataType` (or the `DataType` of the `Value`) is a kind, i.e. its the type of a
/// `DataType`, a `UnionAll`, a `Union` or a `Union{}`.
pub struct Type;
unsafe impl Typecheck for Type {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        t.as_value().is_kind()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the `DataType` (or the `DataType` of the `Value`) is a bits type.
pub struct Bits;
unsafe impl Typecheck for Bits {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        t.is_bits()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the `DataType` is abstract. If it's invoked through `Value::is` it will always return false.
pub struct Abstract;
unsafe impl Typecheck for Abstract {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        t.is_abstract()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the value is a `Ref`.
pub struct AbstractRef;
unsafe impl Typecheck for AbstractRef {
    fn typecheck(t: DataType) -> bool {
        unsafe {
            t.type_name()
                == UnionAll::ref_type(&Unrooted::new())
                    .body()
                    .cast_unchecked::<DataType>()
                    .type_name()
        }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the value is a `VecElement`.
pub struct VecElement;
unsafe impl Typecheck for VecElement {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name() == TypeName::of_vecelement(&Unrooted::new()) }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the value is a `Type{T}`.
pub struct TypeType;
unsafe impl Typecheck for TypeType {
    fn typecheck(t: DataType) -> bool {
        unsafe {
            t.type_name()
                == UnionAll::type_type(&Unrooted::new())
                    .body()
                    .cast_unchecked::<DataType>()
                    .type_name()
        }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a named tuple.
pub struct NamedTuple;
unsafe impl Typecheck for NamedTuple {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name() == TypeName::of_namedtuple(&Unrooted::new()) }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the fields of a value of this type can be modified.
pub struct Mutable;
unsafe impl Typecheck for Mutable {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        t.mutable()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the fields of a value of this type cannot be modified.
pub struct Immutable;
unsafe impl Typecheck for Immutable {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        !t.mutable()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a primitive type.
pub struct PrimitiveType;
unsafe impl Typecheck for PrimitiveType {
    fn typecheck(t: DataType) -> bool {
        unsafe {
            t.is::<Immutable>()
                && t.has_layout()
                && t.n_fields().unwrap_unchecked() == 0
                && t.size().unwrap_unchecked() > 0
        }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a struct type.
pub struct StructType;
unsafe impl Typecheck for StructType {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        !t.is_abstract() && !t.is::<PrimitiveType>()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a struct type.
pub struct Singleton;
unsafe impl Typecheck for Singleton {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        t.instance().is_some()
    }
}

impl_julia_typecheck!(String, jl_string_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a pointer to data not owned by Julia.
pub struct Pointer;
unsafe impl Typecheck for Pointer {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name() == TypeName::of_pointer(&Unrooted::new()) }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is an LLVM pointer.
pub struct LLVMPointer;
unsafe impl Typecheck for LLVMPointer {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name() == TypeName::of_llvmpointer(&Unrooted::new()) }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// instances of the type can be created.
pub struct Concrete;
unsafe impl Typecheck for Concrete {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        t.is_concrete_type()
    }
}
