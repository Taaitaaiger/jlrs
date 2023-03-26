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
use std::ffi::c_void;

#[julia_version(until = "1.9")]
use jl_sys::jl_typedslot_type;
use jl_sys::{
    jl_code_info_type, jl_globalref_type, jl_gotonode_type, jl_intrinsic_type,
    jl_linenumbernode_type, jl_namedtuple_typename, jl_newvarnode_type, jl_phicnode_type,
    jl_phinode_type, jl_pinode_type, jl_quotenode_type, jl_slotnumber_type, jl_string_type,
    jl_upsilonnode_type,
};
use jlrs_macros::julia_version;

use crate::{
    convert::into_julia::IntoJulia,
    data::managed::{
        datatype::DataType, private::ManagedPriv, type_name::TypeName, union_all::UnionAll, Managed,
    },
    memory::target::unrooted::Unrooted,
    private::Private,
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
pub unsafe trait Typecheck {
    /// Returns whether the property implied by `Self` holds true.
    fn typecheck(t: DataType) -> bool;
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_julia_typecheck {
    ($type:ty, $jl_type:expr, $($lt:lifetime),+) => {
        unsafe impl<$($lt),+> crate::data::types::typecheck::Typecheck for $type {
            #[inline(always)]
            fn typecheck(t: $crate::data::managed::datatype::DataType) -> bool {
                unsafe {
                    <$crate::data::managed::datatype::DataType as $crate::data::managed::private::ManagedPriv>::unwrap(t, crate::private::Private) == $jl_type
                }
            }
        }
    };
    ($type:ty, $jl_type:expr) => {
        unsafe impl crate::data::types::typecheck::Typecheck for $type {
            #[inline(always)]
            fn typecheck(t: $crate::data::managed::datatype::DataType) -> bool {
                unsafe {
                    <$crate::data::managed::datatype::DataType as $crate::data::managed::private::ManagedPriv>::unwrap(t, crate::private::Private) == $jl_type
                }
            }
        }
    };
    ($type:ty) => {
        unsafe impl crate::data::types::typecheck::Typecheck for $type {
            #[inline(always)]
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
    fn typecheck(t: DataType) -> bool {
        unsafe {
            let global = Unrooted::new();
            let ptr_tname = TypeName::of_pointer(&global);

            if t.type_name() != ptr_tname {
                return false;
            }

            let params = t.parameters();
            let params = params.data().as_slice();
            let inner_ty = T::julia_type(global);
            if params[0].unwrap().as_value() != inner_ty.as_value() {
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
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        t.as_value().is_kind()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the `DataType` (or the `DataType` of the `Value`) is a bits type.
pub struct Bits;
unsafe impl Typecheck for Bits {
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        t.is_bits()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the `DataType` is abstract. If it's invoked through `Value::is` it will always return false.
pub struct Abstract;
unsafe impl Typecheck for Abstract {
    #[inline(always)]
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
    #[inline(always)]
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
/// the value is a dispatch tuple.
pub struct DispatchTuple;
unsafe impl Typecheck for DispatchTuple {
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        t.is_dispatch_tuple()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a named tuple.
pub struct NamedTuple;
unsafe impl Typecheck for NamedTuple {
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        unsafe { t.unwrap_non_null(Private).as_ref().name == jl_namedtuple_typename }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the fields of a value of this type can be modified.
pub struct Mutable;
unsafe impl Typecheck for Mutable {
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        t.mutable()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the fields of a value of this type cannot be modified.
pub struct Immutable;
unsafe impl Typecheck for Immutable {
    #[inline(always)]
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
                && !t.layout().is_none()
                && t.n_fields().unwrap_unchecked() == 0
                && t.size().unwrap_unchecked() > 0
        }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a struct type.
pub struct StructType;
unsafe impl Typecheck for StructType {
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        !t.is_abstract() && !t.is::<PrimitiveType>()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a struct type.
pub struct Singleton;
unsafe impl Typecheck for Singleton {
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        t.instance().is_some()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a slot.
pub struct Slot;

#[julia_version(until = "1.9")]
unsafe impl Typecheck for Slot {
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        unsafe { t.unwrap(Private) == jl_slotnumber_type || t.unwrap(Private) == jl_typedslot_type }
    }
}

#[julia_version(since = "1.10")]
unsafe impl Typecheck for Slot {
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        unsafe { t.unwrap(Private) == jl_slotnumber_type }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a global reference.
pub struct GlobalRef;
impl_julia_typecheck!(GlobalRef, jl_globalref_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a Goto node.
pub struct GotoNode;
impl_julia_typecheck!(GotoNode, jl_gotonode_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a Pi node.
pub struct PiNode;
impl_julia_typecheck!(PiNode, jl_pinode_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a Phi node.
pub struct PhiNode;
impl_julia_typecheck!(PhiNode, jl_phinode_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a PhiC node.
pub struct PhiCNode;
impl_julia_typecheck!(PhiCNode, jl_phicnode_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is an Upsilon node.
pub struct UpsilonNode;
impl_julia_typecheck!(UpsilonNode, jl_upsilonnode_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a Quote node.
pub struct QuoteNode;
impl_julia_typecheck!(QuoteNode, jl_quotenode_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is an NewVar node.
pub struct NewVarNode;
impl_julia_typecheck!(NewVarNode, jl_newvarnode_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a Line node.
pub struct LineNode;
impl_julia_typecheck!(LineNode, jl_linenumbernode_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is code info.
pub struct CodeInfo;
impl_julia_typecheck!(CodeInfo, jl_code_info_type);

impl_julia_typecheck!(String, jl_string_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a pointer to data not owned by Julia.
pub struct Pointer;
unsafe impl Typecheck for Pointer {
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name() == TypeName::of_pointer(&Unrooted::new()) }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is an LLVM pointer.
pub struct LLVMPointer;
unsafe impl Typecheck for LLVMPointer {
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name() == TypeName::of_llvmpointer(&Unrooted::new()) }
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is an intrinsic.
pub struct Intrinsic;
impl_julia_typecheck!(Intrinsic, jl_intrinsic_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// instances of the type can be created.
pub struct Concrete;
unsafe impl Typecheck for Concrete {
    #[inline(always)]
    fn typecheck(t: DataType) -> bool {
        t.is_concrete_type()
    }
}
