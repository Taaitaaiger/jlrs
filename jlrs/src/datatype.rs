use crate::module::Module;
use crate::symbol::Symbol;
use crate::traits::{IntoJulia, TryUnbox};
use jl_sys::{
    jl_bool_type, jl_char_type, jl_code_info_type, jl_code_instance_type, jl_datatype_align,
    jl_datatype_isinlinealloc, jl_datatype_nbits, jl_datatype_nfields, jl_datatype_size,
    jl_datatype_t, jl_datatype_type, jl_expr_type, jl_float32_type, jl_float64_type,
    jl_globalref_type, jl_gotonode_type, jl_int16_type, jl_int32_type, jl_int64_type, jl_int8_type,
    jl_intrinsic_type, jl_is_cpointer_type, jl_linenumbernode_type, jl_method_instance_type,
    jl_method_type, jl_methtable_type, jl_module_type, jl_namedtuple_typename, jl_newvarnode_type,
    jl_phicnode_type, jl_phinode_type, jl_pinode_type, jl_quotenode_type, jl_simplevector_type,
    jl_slotnumber_type, jl_ssavalue_type, jl_string_type, jl_symbol_type, jl_task_type,
    jl_tuple_typename, jl_tvar_type, jl_typedslot_type, jl_typename_type, jl_uint16_type,
    jl_uint32_type, jl_uint64_type, jl_uint8_type, jl_unionall_type, jl_uniontype_type,
    jl_upsilonnode_type,
};
use std::marker::PhantomData;

/// Trait implemented by types that have an associated type in Julia. Do not implement this
/// yourself. This trait can be derived for structs that are marked as `[repr(C)]` and only
/// contain fields that implement this trait by deriving `JuliaTuple`.
pub unsafe trait JuliaType {
    unsafe fn julia_type() -> *mut jl_datatype_t;
}

pub unsafe trait JuliaTypecheck {
    unsafe fn julia_typecheck(t: Datatype) -> bool;
}

/// Implemented when using `#[derive(JuliaTuple)]`. Do not implement this yourself.
pub unsafe trait JuliaTuple: JuliaType + IntoJulia + TryUnbox + Copy + Clone {}

/// Implemented when using `#[derive(JuliaTuple)]`. Do not implement this yourself.
pub unsafe trait JuliaStruct: JuliaType + IntoJulia + TryUnbox + Copy + Clone {}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Datatype<'frame>(*mut jl_datatype_t, PhantomData<&'frame ()>);

impl<'frame> Datatype<'frame> {
    unsafe fn ptr(self) -> *mut jl_datatype_t {
        self.0
    }

    pub fn is<T: JuliaTypecheck>(self) -> bool {
        unsafe { T::julia_typecheck(self) }
    }

    pub fn size(self) -> i32 {
        unsafe { jl_datatype_size(self.0) }
    }

    pub fn align(self) -> u16 {
        unsafe { jl_datatype_align(self.0) }
    }

    pub fn nbits(self) -> i32 {
        unsafe { jl_datatype_nbits(self.0) }
    }

    pub fn nfields(self) -> u32 {
        unsafe { jl_datatype_nfields(self.0) }
    }

    pub fn isinlinealloc(self) -> u8 {
        unsafe { jl_datatype_isinlinealloc(self.0) }
    }
}

macro_rules! impl_julia_type {
    ($type:ty, $jl_type:expr) => {
        unsafe impl JuliaType for $type {
            unsafe fn julia_type() -> *mut jl_datatype_t {
                $jl_type
            }
        }
    };
}

macro_rules! impl_julia_typecheck {
    ($type:ty, $jl_type:expr, $($lt:lifetime),+) => {
        unsafe impl<$($lt),+> JuliaTypecheck for $type {
            unsafe fn julia_typecheck(t: Datatype) -> bool {
                t.ptr() == $jl_type
            }
        }
    };
    ($type:ty, $jl_type:expr) => {
        unsafe impl JuliaTypecheck for $type {
            unsafe fn julia_typecheck(t: Datatype) -> bool {
                t.ptr() == $jl_type
            }
        }
    };
    ($type:ty) => {
        unsafe impl JuliaTypecheck for $type {
            unsafe fn julia_typecheck(t: Datatype) -> bool {
                t.ptr() == <$type as crate::datatype::JuliaType>::julia_type()
            }
        }
    };
}

pub struct Tuple;

unsafe impl JuliaTypecheck for Tuple {
    unsafe fn julia_typecheck(t: Datatype) -> bool {
        (&*t.ptr()).name == jl_tuple_typename
    }
}

pub struct NamedTuple;

unsafe impl JuliaTypecheck for NamedTuple {
    unsafe fn julia_typecheck(t: Datatype) -> bool {
        (&*t.ptr()).name == jl_namedtuple_typename
    }
}

pub struct SVec;
impl_julia_typecheck!(SVec, jl_simplevector_type);
impl_julia_typecheck!(Datatype<'frame>, jl_datatype_type, 'frame);

pub struct Mutable;

unsafe impl JuliaTypecheck for Mutable {
    unsafe fn julia_typecheck(t: Datatype) -> bool {
        (&*t.ptr()).mutabl != 0
    }
}

pub struct MutableDatatype;

unsafe impl JuliaTypecheck for MutableDatatype {
    unsafe fn julia_typecheck(t: Datatype) -> bool {
        Datatype::julia_typecheck(t) && (&*t.ptr()).mutabl != 0
    }
}

pub struct Immutable;

unsafe impl JuliaTypecheck for Immutable {
    unsafe fn julia_typecheck(t: Datatype) -> bool {
        (&*t.ptr()).mutabl == 0
    }
}

pub struct ImmutableDatatype;

unsafe impl JuliaTypecheck for ImmutableDatatype {
    unsafe fn julia_typecheck(t: Datatype) -> bool {
        Datatype::julia_typecheck(t) && (&*t.ptr()).mutabl == 0
    }
}

pub struct UnionType;
impl_julia_typecheck!(UnionType, jl_uniontype_type);

pub struct TypeVar;
impl_julia_typecheck!(TypeVar, jl_tvar_type);

pub struct UnionAll;
impl_julia_typecheck!(UnionAll, jl_unionall_type);

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

pub struct SSAValue;
impl_julia_typecheck!(SSAValue, jl_ssavalue_type);

pub struct Slot;

unsafe impl JuliaTypecheck for Slot {
    unsafe fn julia_typecheck(t: Datatype) -> bool {
        t.ptr() == jl_slotnumber_type || t.ptr() == jl_typedslot_type
    }
}

pub struct Expr;
impl_julia_typecheck!(Expr, jl_expr_type);

pub struct GlobalRef;
impl_julia_typecheck!(GlobalRef, jl_globalref_type);

pub struct GotoNode;
impl_julia_typecheck!(GotoNode, jl_gotonode_type);

pub struct PiNode;
impl_julia_typecheck!(PiNode, jl_pinode_type);

pub struct PhiNode;
impl_julia_typecheck!(PhiNode, jl_phinode_type);

pub struct PhiCNode;
impl_julia_typecheck!(PhiCNode, jl_phicnode_type);

pub struct UpsilonNode;
impl_julia_typecheck!(UpsilonNode, jl_upsilonnode_type);

pub struct QuoteNode;
impl_julia_typecheck!(QuoteNode, jl_quotenode_type);

pub struct NewVarNode;
impl_julia_typecheck!(NewVarNode, jl_newvarnode_type);

pub struct LineNode;
impl_julia_typecheck!(LineNode, jl_linenumbernode_type);

pub struct MethodInstance;
impl_julia_typecheck!(MethodInstance, jl_method_instance_type);

pub struct CodeInstance;
impl_julia_typecheck!(CodeInstance, jl_code_instance_type);

pub struct CodeInfo;
impl_julia_typecheck!(CodeInfo, jl_code_info_type);

pub struct Method;
impl_julia_typecheck!(Method, jl_method_type);

impl_julia_typecheck!(Module<'frame>, jl_module_type, 'frame);

pub struct MTable;
impl_julia_typecheck!(MTable, jl_methtable_type);

pub struct Task;
impl_julia_typecheck!(Task, jl_task_type);

impl_julia_typecheck!(String, jl_string_type);

pub struct Pointer;
unsafe impl JuliaTypecheck for Pointer {
    unsafe fn julia_typecheck(t: Datatype) -> bool {
        jl_is_cpointer_type(t.ptr().cast())
    }
}

pub struct Intrinsic;
impl_julia_typecheck!(Intrinsic, jl_intrinsic_type);

impl_julia_type!(u8, jl_uint8_type);
impl_julia_type!(u16, jl_uint16_type);
impl_julia_type!(u32, jl_uint32_type);
impl_julia_type!(u64, jl_uint64_type);
impl_julia_type!(i8, jl_int8_type);
impl_julia_type!(i16, jl_int16_type);
impl_julia_type!(i32, jl_int32_type);
impl_julia_type!(i64, jl_int64_type);
impl_julia_type!(f32, jl_float32_type);
impl_julia_type!(f64, jl_float64_type);
impl_julia_type!(bool, jl_bool_type);
impl_julia_type!(char, jl_char_type);

#[cfg(not(target_pointer_width = "64"))]
unsafe impl JuliaType for usize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_uint32_type
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl JuliaType for usize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_uint64_type
    }
}

#[cfg(not(target_pointer_width = "64"))]
unsafe impl JuliaType for isize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_int32_type
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl JuliaType for isize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_int64_type
    }
}
