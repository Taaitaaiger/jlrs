//! Datatypes and properties.
//!
//! Julia has an optional typing system. The type information of a [`Value`] is available at
//! runtime. Additionally, a value can hold type information as its contents. For example,
//!
//! ```julia
//! truth = true
//! truthtype = typeof(truth)
//! @assert(truthtype == Bool)
//! @assert(truthtype isa DataType)
//! ```
//!
//! In this module you'll find the [`DataType`] struct which provides access to the properties
//! of its counterpart in Julia and lets you perform a large set of checks to find out its
//! properties. Many of these checks are handled through implementations of the trait
//! [`JuliaTypecheck`]. Some of these checks can be found in this module.
//!
//! [`Value`]: ../struct.Value.html
//! [`DataType`]: struct.DataType.html
//! [`JuliaTypecheck`]: ../../traits/trait.JuliaTypecheck.html

use crate::error::{JlrsError, JlrsResult};
use crate::traits::{Cast, JuliaTypecheck};
use crate::value::symbol::Symbol;
use crate::value::type_name::TypeName;
use crate::value::Value;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{
    jl_any_type, jl_code_info_type, jl_datatype_align, jl_datatype_isinlinealloc,
    jl_datatype_nbits, jl_datatype_nfields, jl_datatype_size, jl_datatype_t, jl_datatype_type,
    jl_field_isptr, jl_field_names, jl_field_offset, jl_field_size, jl_get_fieldtypes,
    jl_globalref_type, jl_gotonode_type, jl_intrinsic_type, jl_is_cpointer_type, jl_isbits,
    jl_linenumbernode_type, jl_namedtuple_typename, jl_newvarnode_type, jl_phicnode_type,
    jl_phinode_type, jl_pinode_type, jl_quotenode_type, jl_slotnumber_type, jl_string_type,
    jl_svec_data, jl_svec_len, jl_typedslot_type, jl_typename_str, jl_upsilonnode_type,
};
use std::ffi::CStr;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
/// Julia type information. You can acquire a [`Value`]'s datatype by by calling
/// [`Value::datatype`]. This struct implements [`JuliaTypecheck`] and [`Cast`]. It can be used in
/// combination with [`DataType::is`] and [`Value::is`]; if the check returns `true` the [`Value`]
///  can be cast to `DataType`:
///
/// ```
/// # use jlrs::prelude::*;
/// # use jlrs::util::JULIA;
/// # fn main() {
/// # JULIA.with(|j| {
/// # let mut julia = j.borrow_mut();
/// julia.frame(2, |global, frame| {
///     let val = Value::new(frame, 1u8)?;
///     let typeof_func = Module::core(global).function("typeof")?;
///     let ty_val = typeof_func.call1(frame, val)?.unwrap();
///     assert!(ty_val.is::<DataType>());
///     assert!(ty_val.cast::<DataType>().is_ok());
///     Ok(())
/// }).unwrap();
/// # });
/// # }
/// ```
///
/// [`JuliaTypecheck`]: ../../traits/trait.JuliaTypecheck.html
/// [`Cast`]: ../../traits/trait.Cast.html
/// [`DataType::is`]: ../datatype/struct.DataType.html#method.is
/// [`Value::is`]: ../struct.Value.html#method.is
/// [`Value`]: ../struct.Value.html
/// [`Value::datatype`]: ../struct.Value.html#method.datatype
/// [`Value::cast`]: ../struct.Value.html#method.cast
/// [`JuliaTypecheck`]: ../../traits/trait.JuliaTypecheck.html
/// [`DataType::is`]: struct.Datatype.html#method.is
/// [`Value::is`]: struct.Datatype.html#method.is
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct DataType<'frame>(*mut jl_datatype_t, PhantomData<&'frame ()>);

impl<'frame> DataType<'frame> {
    pub(crate) unsafe fn wrap(datatype: *mut jl_datatype_t) -> Self {
        DataType(datatype, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_datatype_t {
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

    /// Returns the size of a value of this type in bits.
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

    /// Returns the name of this type.
    pub fn name(self) -> &'frame str {
        unsafe {
            let name = jl_typename_str(self.ptr().cast());
            CStr::from_ptr(name).to_str().unwrap()
        }
    }

    /// Returns the `TypeName` of this type.
    pub fn type_name(self) -> TypeName<'frame> {
        unsafe { TypeName::wrap((&*self.ptr()).name) }
    }

    /// Returns the field names of this type as a slice of `Symbol`s. These symbols can be used
    /// to access their fields with [`Value::get_field`].
    ///
    /// [`Value::get_field`]: struct.Value.html#method.get_field
    pub fn field_names(self) -> &'frame [Symbol<'frame>] {
        unsafe {
            let field_names = jl_field_names(self.ptr().cast());
            let len = jl_svec_len(field_names);
            let items = jl_svec_data(field_names);
            std::slice::from_raw_parts(items.cast(), len)
        }
    }

    /// Returns the field types of this type.
    pub fn field_types(self) -> &'frame [Value<'frame, 'static>] {
        unsafe {
            let field_types = jl_get_fieldtypes(self.ptr());
            let len = jl_svec_len(field_types);
            let items = jl_svec_data(field_types);
            std::slice::from_raw_parts(items.cast(), len)
        }
    }

    /// Returns the size of the field at position `idx` in this type.
    pub fn field_size(self, idx: usize) -> u32 {
        unsafe { jl_field_size(self.ptr(), idx as _) }
    }

    /// Returns the offset where the field at position `idx` is stored.
    pub fn field_offset(self, idx: usize) -> u32 {
        unsafe { jl_field_offset(self.ptr(), idx as _) }
    }

    /// Returns true if the field at position `idx` is a pointer.
    pub fn is_pointer_field(self, idx: usize) -> bool {
        unsafe { jl_field_isptr(self.ptr(), idx as _) }
    }

    /// Returns true if this type is a bits-type.
    pub fn isbits(self) -> bool {
        unsafe { jl_isbits(self.ptr().cast()) }
    }

    /// Returns the supertype of this type.
    pub fn super_type(self) -> Option<Self> {
        unsafe {
            let sup = (&*self.ptr()).super_;
            if sup.is_null() {
                None
            } else {
                Some(DataType::wrap(sup))
            }
        }
    }

    /// Returns the type parameters of this type.
    pub fn parameters(self) -> &'frame [Value<'frame, 'static>] {
        unsafe {
            let params = (&*self.ptr()).parameters;
            std::slice::from_raw_parts(jl_svec_data(params).cast(), jl_svec_len(params))
        }
    }

    /// Returns the instance if this type is a singleton.
    pub fn instance(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let instance = (&*self.ptr()).instance;
            if instance.is_null() {
                None
            } else {
                Some(Value::wrap(instance))
            }
        }
    }

    /// Returns the number of initialized fields.
    pub fn n_initialized(self) -> i32 {
        unsafe { (&*self.ptr()).ninitialized }
    }

    /// Returns the hash of this type.
    pub fn hash(self) -> u32 {
        unsafe { (&*self.ptr()).hash }
    }

    /// Returns true if this is an abstract type.
    pub fn is_abstract(self) -> bool {
        unsafe { (&*self.ptr()).abstract_ != 0 }
    }

    /// Returns true if this is a mutable type.
    pub fn mutable(self) -> bool {
        unsafe { (&*self.ptr()).mutabl != 0 }
    }

    /// Returns true if one or more of the type parameters has not been set.
    pub fn has_free_type_vars(self) -> bool {
        unsafe { (&*self.ptr()).hasfreetypevars != 0 }
    }

    /// Returns true if this type can have instances
    pub fn is_concrete_type(self) -> bool {
        unsafe { (&*self.ptr()).isconcretetype != 0 }
    }

    /// Returns true if this type is a dispatch, or leaf, tuple type.
    pub fn is_dispatch_tuple(self) -> bool {
        unsafe { (&*self.ptr()).isdispatchtuple != 0 }
    }

    /// Returns true if one or more fields require zero-initialization.
    pub fn zeroinit(self) -> bool {
        unsafe { (&*self.ptr()).zeroinit != 0 }
    }

    /// If false, no value will have this type.
    pub fn has_concrete_subtype(self) -> bool {
        unsafe { (&*self.ptr()).has_concrete_subtype != 0 }
    }
}

impl<'frame> Into<Value<'frame, 'static>> for DataType<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

impl<'frame, 'data> Debug for DataType<'frame> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("DataType").field(&self.name()).finish()
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for DataType<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotADataType)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        DataType::wrap(value.ptr().cast())
    }
}

impl_julia_type!(DataType<'frame>, jl_datatype_type, 'frame);
impl_valid_layout!(DataType<'frame>, 'frame);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a tuple.
pub struct Any;
impl_julia_typecheck!(Any, jl_any_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a named tuple.
pub struct NamedTuple;

unsafe impl JuliaTypecheck for NamedTuple {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        (&*t.ptr()).name == jl_namedtuple_typename
    }
}

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
/// a value of this type is a primitive type.
pub struct PrimitiveType;

unsafe impl JuliaTypecheck for PrimitiveType {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        t.is::<Immutable>() && !(&*t.ptr()).layout.is_null() && t.nfields() == 0 && t.size() > 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a struct type.
pub struct StructType;

unsafe impl JuliaTypecheck for StructType {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        !t.is_abstract() && !t.is::<PrimitiveType>()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a struct type.
pub struct Singleton;

unsafe impl JuliaTypecheck for Singleton {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        t.instance().is_some()
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a slot.
pub struct Slot;

unsafe impl JuliaTypecheck for Slot {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        t.ptr() == jl_slotnumber_type || t.ptr() == jl_typedslot_type
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

pub struct Concrete;
unsafe impl JuliaTypecheck for Concrete {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        (&*t.ptr()).isconcretetype != 0
    }
}
