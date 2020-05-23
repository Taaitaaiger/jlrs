//! The type of a Julia value.
//!
//! Julia has an optional typing system and the type information of a value is available at
//! runtime. Additionally, a value can hold type information as its contents. For example,
//!
//! ```julia
//! truth = true
//! truthtype = typeof(truth)
//! @assert(truthtype == Bool)
//! @assert(truthtype isa DataType)
//! ```
//!
//! In this module you'll find the `DataType` struct which provides access to the properties
//! of its counterpart in Julia and lets you perform a large set of checks. Many of these
//! checks are handled through implementations of the trait [`JuliaTypecheck`].

use crate::traits::JuliaType;
use crate::value::module::Module;
use crate::value::symbol::Symbol;
use jl_sys::{
    jl_code_info_type, jl_code_instance_type, jl_datatype_align, jl_datatype_isinlinealloc,
    jl_datatype_nbits, jl_datatype_nfields, jl_datatype_size, jl_datatype_t, jl_datatype_type,
    jl_expr_type, jl_globalref_type, jl_gotonode_type, jl_intrinsic_type, jl_is_cpointer_type,
    jl_linenumbernode_type, jl_method_instance_type, jl_method_type, jl_methtable_type,
    jl_module_type, jl_namedtuple_typename, jl_newvarnode_type, jl_phicnode_type, jl_phinode_type,
    jl_pinode_type, jl_quotenode_type, jl_simplevector_type, jl_slotnumber_type, jl_ssavalue_type,
    jl_string_type, jl_symbol_type, jl_task_type, jl_tuple_typename, jl_tvar_type,
    jl_typedslot_type, jl_typename_type, jl_unionall_type, jl_uniontype_type, jl_upsilonnode_type,
};
use std::marker::PhantomData;

/// This trait is used in combination with [`DataType::is`] and can be used to check many 
/// properties of a Julia `DataType`. You should not implement this trait for your own types.
/// 
/// This trait is implemented for a few types from the standard library, eg `String` and `u8`. In
/// these cases, [`DataType::is`] returns true if [`Value::is`] would return `true` for that type.
pub unsafe trait JuliaTypecheck {
    #[doc(hidden)]
    unsafe fn julia_typecheck(t: DataType) -> bool;
}

/// Julia type information. You can acquire a [`Value`]'s datatype by by calling 
/// [`Value::datatype`]. If a the value contains a datatype (`value.is::<DataType>()` returns 
/// `true`), you can cast the value to a `DataType` by calling [`Value::cast`].
/// 
/// `DataType` implements [`JuliaTypecheck`] and can be used in combination with [`DataType::is`].
/// This method returns `true` if a value of this type is itself a datatype. 
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct DataType<'frame>(*mut jl_datatype_t, PhantomData<&'frame ()>);

impl<'frame> DataType<'frame> {
    unsafe fn ptr(self) -> *mut jl_datatype_t {
        self.0
    }

    /// Performs the given typecheck.
    pub fn is<T: JuliaTypecheck>(self) -> bool {
        unsafe { T::julia_typecheck(self) }
    }

    /// Returns the size of a value of this type in bytes.
    pub fn size(self) -> i32 {
        unsafe { jl_datatype_size(self.0) }
    }

    /// Returns the alignment of a value of this type in bytes.
    pub fn align(self) -> u16 {
        unsafe { jl_datatype_align(self.0) }
    }

    /// Returns the alignment of a value of this type in bits.
    pub fn nbits(self) -> i32 {
        unsafe { jl_datatype_nbits(self.0) }
    }

    /// Returns the number of fields of a value of this type.
    pub fn nfields(self) -> u32 {
        unsafe { jl_datatype_nfields(self.0) }
    }

    /// Returns true if a value of this type stores its data inline.
    pub fn isinlinealloc(self) -> bool {
        unsafe { jl_datatype_isinlinealloc(self.0) != 0 } 
    }
}

macro_rules! impl_julia_typecheck {
    ($type:ty, $jl_type:expr, $($lt:lifetime),+) => {
        unsafe impl<$($lt),+> JuliaTypecheck for $type {
            unsafe fn julia_typecheck(t: DataType) -> bool {
                t.ptr() == $jl_type
            }
        }
    };
    ($type:ty, $jl_type:expr) => {
        unsafe impl JuliaTypecheck for $type {
            unsafe fn julia_typecheck(t: DataType) -> bool {
                t.ptr() == $jl_type
            }
        }
    };
    ($type:ty) => {
        unsafe impl JuliaTypecheck for $type {
            unsafe fn julia_typecheck(t: DataType) -> bool {
                t.ptr() == <$type as crate::value::datatype::JuliaType>::julia_type()
            }
        }
    };
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a tuple.
pub struct Tuple;

unsafe impl JuliaTypecheck for Tuple {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        (&*t.ptr()).name == jl_tuple_typename
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a named tuple.
pub struct NamedTuple;

unsafe impl JuliaTypecheck for NamedTuple {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        (&*t.ptr()).name == jl_namedtuple_typename
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is an svec (simple vector).
pub struct SVec;
impl_julia_typecheck!(SVec, jl_simplevector_type);
impl_julia_typecheck!(DataType<'frame>, jl_datatype_type, 'frame);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the fields of a value of this type can be modified.
pub struct Mutable;

unsafe impl JuliaTypecheck for Mutable {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        (&*t.ptr()).mutabl != 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the datatype is a mutable datatype.
pub struct MutableDatatype;

unsafe impl JuliaTypecheck for MutableDatatype {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        DataType::julia_typecheck(t) && (&*t.ptr()).mutabl != 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the fields of a value of this type cannot be modified.
pub struct Immutable;

unsafe impl JuliaTypecheck for Immutable {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        (&*t.ptr()).mutabl == 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the datatype is an immutable datatype.
pub struct ImmutableDatatype;

unsafe impl JuliaTypecheck for ImmutableDatatype {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        DataType::julia_typecheck(t) && (&*t.ptr()).mutabl == 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a union.
pub struct UnionType;
impl_julia_typecheck!(UnionType, jl_uniontype_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a type var.
pub struct TypeVar;
impl_julia_typecheck!(TypeVar, jl_tvar_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a `UnionAll`.
pub struct UnionAll;
impl_julia_typecheck!(UnionAll, jl_unionall_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a type name.
pub struct TypeName;
impl_julia_typecheck!(TypeName, jl_typename_type);

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
impl_julia_typecheck!(Symbol<'frame>, jl_symbol_type, 'frame);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is an SSA value.
pub struct SSAValue;
impl_julia_typecheck!(SSAValue, jl_ssavalue_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a slot.
pub struct Slot;

unsafe impl JuliaTypecheck for Slot {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        t.ptr() == jl_slotnumber_type || t.ptr() == jl_typedslot_type
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is an expr, a type representing compound expressions in parsed julia code
/// (ASTs).
pub struct Expr;
impl_julia_typecheck!(Expr, jl_expr_type);

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
/// a value of this type is a method instance.
pub struct MethodInstance;
impl_julia_typecheck!(MethodInstance, jl_method_instance_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a code instance.
pub struct CodeInstance;
impl_julia_typecheck!(CodeInstance, jl_code_instance_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is code info.
pub struct CodeInfo;
impl_julia_typecheck!(CodeInfo, jl_code_info_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a method.
pub struct Method;
impl_julia_typecheck!(Method, jl_method_type);

impl_julia_typecheck!(Module<'frame>, jl_module_type, 'frame);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a method table.
pub struct MethodTable;
impl_julia_typecheck!(MethodTable, jl_methtable_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a task.
pub struct Task;
impl_julia_typecheck!(Task, jl_task_type);

impl_julia_typecheck!(String, jl_string_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a pointer.
pub struct Pointer;
unsafe impl JuliaTypecheck for Pointer {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        jl_is_cpointer_type(t.ptr().cast())
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is an intrinsic.
pub struct Intrinsic;
impl_julia_typecheck!(Intrinsic, jl_intrinsic_type);
