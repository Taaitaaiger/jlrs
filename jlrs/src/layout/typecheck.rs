//! Trait for checking Julia type properties.

use jl_sys::{
    jl_code_info_type, jl_datatype_type, jl_globalref_type, jl_gotonode_type, jl_intrinsic_type,
    jl_is_cpointer_type, jl_linenumbernode_type, jl_namedtuple_typename, jl_newvarnode_type,
    jl_nothing_type, jl_phicnode_type, jl_phinode_type, jl_pinode_type, jl_quotenode_type,
    jl_slotnumber_type, jl_string_type, jl_typedslot_type, jl_upsilonnode_type,
};

use crate::{
    private::Private,
    wrappers::ptr::{datatype::DataType, private::Wrapper},
};
use std::ffi::c_void;

/// This trait is used in combination with [`Value::is`] and [`DataType::is`]; types that
/// implement this trait can be used to check many properties of a Julia `DataType`.
///
/// This trait is implemented for a few types that implement [`ValidLayout`], eg `String`,
/// [`Array`], and `u8`. In these cases, if the check returns `true` the value can be successfully
/// cast to that type with [`Value::cast`] or unboxed with [`Value::unbox`].
///
/// [`Value::is`]: crate::wrappers::builtin::value::Value::is
/// [`Value::cast`]: crate::wrappers::builtin::value::Value::cast
/// [`Value::unbox`]: crate::wrappers::builtin::value::Value::unbox
/// [`Array`]: crate::wrappers::ptr::array::Array
/// [`ValidLayout`]: crate::layout::valid_layout::ValidLayout
pub unsafe trait Typecheck {
    #[doc(hidden)]
    unsafe fn typecheck(t: DataType) -> bool;
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_julia_typecheck {
    ($type:ty, $jl_type:expr, $($lt:lifetime),+) => {
        unsafe impl<$($lt),+> crate::layout::typecheck::Typecheck for $type {
            unsafe fn typecheck(t: $crate::wrappers::ptr::datatype::DataType) -> bool {
                <$crate::wrappers::ptr::datatype::DataType as $crate::wrappers::ptr::private::Wrapper>::unwrap(t, crate::private::Private) == $jl_type
            }
        }
    };
    ($type:ty, $jl_type:expr) => {
        unsafe impl crate::layout::typecheck::Typecheck for $type {
            unsafe fn typecheck(t: $crate::wrappers::ptr::datatype::DataType) -> bool {
                <$crate::wrappers::ptr::datatype::DataType as $crate::wrappers::ptr::private::Wrapper>::unwrap(t, crate::private::Private) == $jl_type
            }
        }
    };
    ($type:ty) => {
        unsafe impl crate::layout::typecheck::Typecheck for $type {
            unsafe fn typecheck(t: crate::wrappers::ptr::datatype::DataType) -> bool {
                let global = $crate::memory::global::Global::new();
                <$crate::wrappers::ptr::datatype::DataType as $crate::wrappers::ptr::private::Wrapper>::unwrap(t, crate::private::Private) == <$type as $crate::convert::into_julia::IntoJulia>::julia_type(global).ptr()
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

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a named tuple.
pub struct NamedTuple;

unsafe impl Typecheck for NamedTuple {
    unsafe fn typecheck(t: DataType) -> bool {
        t.unwrap_non_null(Private).as_ref().name == jl_namedtuple_typename
    }
}

impl_julia_typecheck!(DataType<'frame>, jl_datatype_type, 'frame);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the fields of a value of this type can be modified.
pub struct Mutable;

unsafe impl Typecheck for Mutable {
    unsafe fn typecheck(t: DataType) -> bool {
        t.unwrap_non_null(Private).as_ref().mutabl != 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the datatype is a mutable datatype.
pub struct MutableDatatype;

unsafe impl Typecheck for MutableDatatype {
    unsafe fn typecheck(t: DataType) -> bool {
        DataType::typecheck(t) && t.unwrap_non_null(Private).as_ref().mutabl != 0
    }
}
/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the datatype is `Nothing`.
pub struct Nothing;
impl_julia_typecheck!(Nothing, jl_nothing_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the fields of a value of this type cannot be modified.
pub struct Immutable;

unsafe impl Typecheck for Immutable {
    unsafe fn typecheck(t: DataType) -> bool {
        t.unwrap_non_null(Private).as_ref().mutabl == 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the datatype is an immutable datatype.
pub struct ImmutableDatatype;

unsafe impl Typecheck for ImmutableDatatype {
    unsafe fn typecheck(t: DataType) -> bool {
        DataType::typecheck(t) && t.unwrap_non_null(Private).as_ref().mutabl == 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a primitive type.
pub struct PrimitiveType;

unsafe impl Typecheck for PrimitiveType {
    unsafe fn typecheck(t: DataType) -> bool {
        t.is::<Immutable>()
            && !t.unwrap_non_null(Private).as_ref().layout.is_null()
            && t.n_fields() == 0
            && t.size() > 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a struct type.
pub struct StructType;

unsafe impl Typecheck for StructType {
    unsafe fn typecheck(t: DataType) -> bool {
        !t.is_abstract() && !t.is::<PrimitiveType>()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a struct type.
pub struct Singleton;

unsafe impl Typecheck for Singleton {
    unsafe fn typecheck(t: DataType) -> bool {
        !t.instance().is_undefined()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a slot.
pub struct Slot;

unsafe impl Typecheck for Slot {
    unsafe fn typecheck(t: DataType) -> bool {
        t.unwrap(Private) == jl_slotnumber_type || t.unwrap(Private) == jl_typedslot_type
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
    unsafe fn typecheck(t: DataType) -> bool {
        jl_is_cpointer_type(t.unwrap(Private).cast())
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
    unsafe fn typecheck(t: DataType) -> bool {
        t.unwrap_non_null(Private).as_ref().isconcretetype != 0
    }
}
