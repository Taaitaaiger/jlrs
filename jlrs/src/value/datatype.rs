//! Datatypes and properties.
//!
//! Julia has an optional typing system. The type information of a [`Value`] is available at
//! runtime. Additionally, a value can hold type information as its contents. For example:
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

use crate::convert::cast::Cast;
use crate::layout::julia_typecheck::JuliaTypecheck;
use crate::memory::traits::frame::Frame;
use crate::value::symbol::Symbol;
use crate::value::type_name::TypeName;
use crate::value::Value;
use crate::{
    error::{JlrsError, JlrsResult},
    memory::traits::scope::Scope,
};
use crate::{impl_julia_typecheck, impl_valid_layout};
use crate::{memory::global::Global, private::Private};
use jl_sys::{
    jl_abstractslot_type, jl_abstractstring_type, jl_any_type, jl_anytuple_type,
    jl_argumenterror_type, jl_bool_type, jl_boundserror_type, jl_builtin_type, jl_char_type,
    jl_code_info_type, jl_code_instance_type, jl_datatype_align, jl_datatype_isinlinealloc,
    jl_datatype_nbits, jl_datatype_nfields, jl_datatype_size, jl_datatype_t, jl_datatype_type,
    jl_emptytuple_type, jl_errorexception_type, jl_expr_type, jl_field_isptr, jl_field_names,
    jl_field_offset, jl_field_size, jl_float16_type, jl_float32_type, jl_float64_type,
    jl_floatingpoint_type, jl_function_type, jl_get_fieldtypes, jl_globalref_type,
    jl_gotonode_type, jl_initerror_type, jl_int16_type, jl_int32_type, jl_int64_type, jl_int8_type,
    jl_intrinsic_type, jl_is_cpointer_type, jl_isbits, jl_lineinfonode_type,
    jl_linenumbernode_type, jl_loaderror_type, jl_method_instance_type, jl_method_type,
    jl_methoderror_type, jl_methtable_type, jl_module_type, jl_namedtuple_typename, jl_new_structv,
    jl_newvarnode_type, jl_nothing_type, jl_number_type, jl_phicnode_type, jl_phinode_type,
    jl_pinode_type, jl_quotenode_type, jl_signed_type, jl_simplevector_type, jl_slotnumber_type,
    jl_ssavalue_type, jl_string_type, jl_symbol_type, jl_task_type, jl_tvar_type,
    jl_typedslot_type, jl_typeerror_type, jl_typemap_entry_type, jl_typemap_level_type,
    jl_typename_str, jl_typename_type, jl_typeofbottom_type, jl_uint16_type, jl_uint32_type,
    jl_uint64_type, jl_uint8_type, jl_undefvarerror_type, jl_unionall_type, jl_uniontype_type,
    jl_upsilonnode_type, jl_voidpointer_type, jl_weakref_type,
};
use std::ffi::CStr;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::ptr::NonNull;

use super::{array::Array, simple_vector::SimpleVector, type_var::TypeVar};
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
/// julia.scope(|global, frame| {
///     let val = Value::new(&mut *frame, 1u8)?;
///     let typeof_func = Module::core(global).function("typeof")?;
///     let ty_val = typeof_func.call1(&mut *frame, val)?.unwrap();
///     assert!(ty_val.is::<DataType>());
///     assert!(ty_val.cast::<DataType>().is_ok());
///     Ok(())
/// }).unwrap();
/// # });
/// # }
/// ```
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct DataType<'frame>(NonNull<jl_datatype_t>, PhantomData<&'frame ()>);

impl<'frame> DataType<'frame> {
    pub(crate) unsafe fn wrap(datatype: *mut jl_datatype_t) -> Self {
        debug_assert!(!datatype.is_null());
        DataType(NonNull::new_unchecked(datatype), PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_datatype_t> {
        self.0
    }

    /*
    for (a, b) in zip(fieldnames(DataType), fieldtypes(DataType))
        println(a, ": ", b)
    end
    name: Core.TypeName
    super: DataType
    parameters: Core.SimpleVector
    types: Core.SimpleVector
    names: Core.SimpleVector
    instance: Any
    layout: Ptr{Nothing}
    size: Int32
    ninitialized: Int32
    hash: Int32
    abstract: Bool
    mutable: Bool
    hasfreetypevars: Bool
    isconcretetype: Bool
    isdispatchtuple: Bool
    isbitstype: Bool
    zeroinit: Bool
    isinlinealloc: Bool
    has_concrete_subtype: Bool
    cached_by_hash: Bool
    */

    /// Returns the `TypeName` of this type.
    pub fn type_name(self) -> TypeName<'frame> {
        unsafe { TypeName::wrap((&*self.inner().as_ptr()).name) }
    }

    /// Returns the supertype of this type.
    pub fn super_type(self) -> Option<Self> {
        unsafe {
            let sup = (&*self.inner().as_ptr()).super_;
            if sup.is_null() {
                None
            } else {
                Some(DataType::wrap(sup))
            }
        }
    }

    /// Returns the type parameters of this type.
    pub fn parameters(self) -> SimpleVector<'frame, TypeVar<'frame>> {
        unsafe { SimpleVector::wrap((&*self.inner().as_ptr()).parameters) }
    }

    /// Returns the field types of this type.
    pub fn field_types(self) -> SimpleVector<'frame, Value<'frame, 'static>> {
        unsafe { SimpleVector::wrap(jl_get_fieldtypes(self.inner().as_ptr())) }
    }

    /// Returns the field names of this type as a slice of `Symbol`s. These symbols can be used
    /// to access their fields with [`Value::get_field`].
    pub fn field_names(self) -> SimpleVector<'frame, Symbol<'frame>> {
        unsafe { SimpleVector::wrap(jl_field_names(self.inner().as_ptr())) }
    }

    /// Returns the instance if this type is a singleton.
    pub fn instance(self) -> Option<Value<'frame, 'static>> {
        unsafe { Value::wrap_maybe_null((&*self.inner().as_ptr()).instance) }
    }

    /// Returns the size of a value of this type in bytes.
    pub fn size(self) -> i32 {
        unsafe { jl_datatype_size(self.inner().as_ptr()) }
    }

    /// Returns the number of initialized fields.
    pub fn n_initialized(self) -> i32 {
        unsafe { (&*self.inner().as_ptr()).ninitialized }
    }

    /// Returns the hash of this type.
    pub fn hash(self) -> u32 {
        unsafe { (&*self.inner().as_ptr()).hash }
    }

    /// Returns true if this is an abstract type.
    pub fn is_abstract(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).abstract_ != 0 }
    }

    /// Returns true if this is a mutable type.
    pub fn mutable(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).mutabl != 0 }
    }

    /// Returns true if one or more of the type parameters has not been set.
    pub fn has_free_type_vars(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).hasfreetypevars != 0 }
    }

    /// Returns true if this type can have instances
    pub fn is_concrete_type(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).isconcretetype != 0 }
    }

    /// Returns true if this type is a dispatch, or leaf, tuple type.
    pub fn is_dispatch_tuple(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).isdispatchtuple != 0 }
    }

    /// Returns true if this type is a bits-type.
    pub fn isbits(self) -> bool {
        unsafe { jl_isbits(self.inner().as_ptr().cast()) }
    }

    /// Returns true if one or more fields require zero-initialization.
    pub fn zeroinit(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).zeroinit != 0 }
    }

    /// Returns true if a value of this type stores its data inline.
    pub fn isinlinealloc(self) -> bool {
        unsafe { jl_datatype_isinlinealloc(self.inner().as_ptr()) != 0 }
    }

    /// If false, no value will have this type.
    pub fn has_concrete_subtype(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).has_concrete_subtype != 0 }
    }

    /// If false, no value will have this type.
    pub fn cached_by_hash(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).cached_by_hash != 0 }
    }

    /// Performs the given typecheck.
    pub fn is<T: JuliaTypecheck>(self) -> bool {
        unsafe { T::julia_typecheck(self) }
    }

    /// Returns the alignment of a value of this type in bytes.
    pub fn align(self) -> u16 {
        unsafe { jl_datatype_align(self.inner().as_ptr()) }
    }

    /// Returns the size of a value of this type in bits.
    pub fn nbits(self) -> i32 {
        unsafe { jl_datatype_nbits(self.inner().as_ptr()) }
    }

    /// Returns the number of fields of a value of this type.
    pub fn nfields(self) -> u32 {
        unsafe { jl_datatype_nfields(self.inner().as_ptr()) }
    }

    /// Returns the name of this type.
    pub fn name(self) -> &'frame str {
        unsafe {
            let name = jl_typename_str(self.inner().as_ptr().cast());
            CStr::from_ptr(name).to_str().unwrap()
        }
    }

    /// Returns the size of the field at position `idx` in this type.
    pub fn field_size(self, idx: usize) -> u32 {
        unsafe { jl_field_size(self.inner().as_ptr(), idx as _) }
    }

    /// Returns the offset where the field at position `idx` is stored.
    pub fn field_offset(self, idx: usize) -> u32 {
        unsafe { jl_field_offset(self.inner().as_ptr(), idx as _) }
    }

    /// Returns true if the field at position `idx` is a pointer.
    pub fn is_pointer_field(self, idx: usize) -> bool {
        unsafe { jl_field_isptr(self.inner().as_ptr(), idx as _) }
    }

    /// Convert `self` to a `Value`.
    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }

    /// Intantiate this `DataType` with the given values. The type must be concrete. One free slot
    /// on the GC stack is required for this function to succeed, returns an error if no slot is
    /// available or if the type is not concrete.
    pub fn instantiate<'scope, 'fr, 'value, 'borrow, S, F, V>(
        self,
        scope: S,
        mut values: V,
    ) -> JlrsResult<S::Value>
    where
        S: Scope<'scope, 'fr, 'borrow, F>,
        F: Frame<'fr>,
        V: AsMut<[Value<'value, 'borrow>]>,
    {
        unsafe {
            if !self.is::<Concrete>() {
                Err(JlrsError::NotConcrete(self.name().into()))?;
            }

            if self.is::<Array>() {
                Err(JlrsError::ArrayNotSupported)?;
            }

            let values = values.as_mut();
            let value = jl_new_structv(
                self.inner().as_ptr(),
                values.as_mut_ptr().cast(),
                values.len() as _,
            );
            scope.value(value, Private)
        }
    }
}

impl<'base> DataType<'base> {
    /// The type of the bottom type, `Union{}`.
    pub fn typeofbottom_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typeofbottom_type) }
    }

    /// The type `DataType`.
    pub fn datatype_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_datatype_type) }
    }

    /// The type `Union`.
    pub fn uniontype_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_uniontype_type) }
    }

    /// The type `UnionAll`.
    pub fn unionall_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_unionall_type) }
    }

    /// The type `TypeVar`.
    pub fn tvar_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_tvar_type) }
    }

    /// The type `Any`.
    pub fn any_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_any_type) }
    }

    /// The type `TypeName`.
    pub fn typename_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typename_type) }
    }

    /// The type `Symbol`.
    pub fn symbol_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_symbol_type) }
    }

    /// The type `Core.SSAValue`.
    pub fn ssavalue_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_ssavalue_type) }
    }

    /// The type `Slot`.
    pub fn abstractslot_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_abstractslot_type) }
    }

    /// The type `SlotNumber`.
    pub fn slotnumber_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_slotnumber_type) }
    }

    /// The type `TypedSlot`.
    pub fn typedslot_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typedslot_type) }
    }

    /// The type `SimpleVector`, or `SVec`.
    pub fn simplevector_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_simplevector_type) }
    }

    /// The type `Tuple`.
    pub fn anytuple_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_anytuple_type) }
    }

    /// The type `Tuple`.
    pub fn tuple_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_anytuple_type) }
    }

    /// The type of an empty tuple.
    pub fn emptytuple_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_emptytuple_type) }
    }

    /// The type `Function`.
    pub fn function_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_function_type) }
    }

    /// The type `Builtin`.
    pub fn builtin_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_builtin_type) }
    }

    /// The type `MethodInstance`.
    pub fn method_instance_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_method_instance_type) }
    }

    /// The type `CodeInstance`.
    pub fn code_instance_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_code_instance_type) }
    }

    /// The type `CodeInfo`.
    pub fn code_info_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_code_info_type) }
    }

    /// The type `Method`.
    pub fn method_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_method_type) }
    }

    /// The type `Module`.
    pub fn module_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_module_type) }
    }

    /// The type `WeakRef`.
    pub fn weakref_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_weakref_type) }
    }

    /// The type `AbstractString`.
    pub fn abstractstring_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_abstractstring_type) }
    }

    /// The type `String`.
    pub fn string_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_string_type) }
    }

    /// The type `ErrorException`.
    pub fn errorexception_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_errorexception_type) }
    }

    /// The type `ArgumentError`.
    pub fn argumenterror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_argumenterror_type) }
    }

    /// The type `LoadError`.
    pub fn loaderror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_loaderror_type) }
    }

    /// The type `InitError`.
    pub fn initerror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_initerror_type) }
    }

    /// The type `TypeError`.
    pub fn typeerror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typeerror_type) }
    }

    /// The type `MethodError`.
    pub fn methoderror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_methoderror_type) }
    }

    /// The type `UndefVarError`.
    pub fn undefvarerror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_undefvarerror_type) }
    }

    /// The type `LineInfoNode`.
    pub fn lineinfonode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_lineinfonode_type) }
    }

    /// The type `BoundsError`.
    pub fn boundserror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_boundserror_type) }
    }

    /// The type `Bool`.
    pub fn bool_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_bool_type) }
    }

    /// The type `Char`.
    pub fn char_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_char_type) }
    }

    /// The type `Int8`.
    pub fn int8_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_int8_type) }
    }

    /// The type `UInt8`.
    pub fn uint8_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_uint8_type) }
    }

    /// The type `Int16`.
    pub fn int16_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_int16_type) }
    }

    /// The type `UInt16`.
    pub fn uint16_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_uint16_type) }
    }

    /// The type `Int32`.
    pub fn int32_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_int32_type) }
    }

    /// The type `UInt32`.
    pub fn uint32_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_uint32_type) }
    }

    /// The type `Int64`.
    pub fn int64_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_int64_type) }
    }

    /// The type `UInt64`.
    pub fn uint64_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_uint64_type) }
    }

    /// The type `Float16`.
    pub fn float16_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_float16_type) }
    }

    /// The type `Float32`.
    pub fn float32_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_float32_type) }
    }

    /// The type `Float64`.
    pub fn float64_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_float64_type) }
    }

    /// The type `AbstractFloat`.
    pub fn floatingpoint_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_floatingpoint_type) }
    }

    /// The type `Number`.
    pub fn number_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_number_type) }
    }

    /// The type `Nothing`.
    pub fn nothing_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_nothing_type) }
    }

    /// The type `Signed`.
    pub fn signed_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_signed_type) }
    }

    /// The type `Ptr{Nothing}`.
    pub fn voidpointer_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_voidpointer_type) }
    }

    /// The type `Task`.
    pub fn task_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_task_type) }
    }

    /// The type `Expr`.
    pub fn expr_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_expr_type) }
    }

    /// The type `GlobalRef`.
    pub fn globalref_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_globalref_type) }
    }

    /// The type `LineNumberNode`.
    pub fn linenumbernode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_linenumbernode_type) }
    }

    /// The type `GotoNode`.
    pub fn gotonode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_gotonode_type) }
    }

    /// The type `PhiNode`.
    pub fn phinode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_phinode_type) }
    }

    /// The type `PiNode`.
    pub fn pinode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_pinode_type) }
    }

    /// The type `PhiCNode`.
    pub fn phicnode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_phicnode_type) }
    }

    /// The type `UpsilonNode`.
    pub fn upsilonnode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_upsilonnode_type) }
    }

    /// The type `QuoteNode`.
    pub fn quotenode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_quotenode_type) }
    }

    /// The type `NewVarNode`.
    pub fn newvarnode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_newvarnode_type) }
    }

    /// The type `Intrinsic`.
    pub fn intrinsic_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_intrinsic_type) }
    }

    /// The type `MethodTable`.
    pub fn methtable_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_methtable_type) }
    }

    /// The type `TypeMapLevel`.
    pub fn typemap_level_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typemap_level_type) }
    }

    /// The type `TypeMapEntry`.
    pub fn typemap_entry_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typemap_entry_type) }
    }
}

impl<'frame> Into<Value<'frame, 'static>> for DataType<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap_non_null(self.inner().cast()) }
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
        DataType::wrap(value.inner().as_ptr().cast())
    }
}

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
        (&*t.inner().as_ptr()).name == jl_namedtuple_typename
    }
}

impl_julia_typecheck!(DataType<'frame>, jl_datatype_type, 'frame);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the fields of a value of this type can be modified.
pub struct Mutable;

unsafe impl JuliaTypecheck for Mutable {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        (&*t.inner().as_ptr()).mutabl != 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the datatype is a mutable datatype.
pub struct MutableDatatype;

unsafe impl JuliaTypecheck for MutableDatatype {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        DataType::julia_typecheck(t) && (&*t.inner().as_ptr()).mutabl != 0
    }
}
/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the datatype is `Nothing`.
pub struct Nothing;
impl_julia_typecheck!(Nothing, jl_nothing_type);

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the fields of a value of this type cannot be modified.
pub struct Immutable;

unsafe impl JuliaTypecheck for Immutable {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        (&*t.inner().as_ptr()).mutabl == 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// the datatype is an immutable datatype.
pub struct ImmutableDatatype;

unsafe impl JuliaTypecheck for ImmutableDatatype {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        DataType::julia_typecheck(t) && (&*t.inner().as_ptr()).mutabl == 0
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a primitive type.
pub struct PrimitiveType;

unsafe impl JuliaTypecheck for PrimitiveType {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        t.is::<Immutable>()
            && !(&*t.inner().as_ptr()).layout.is_null()
            && t.nfields() == 0
            && t.size() > 0
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
        t.inner().as_ptr() == jl_slotnumber_type || t.inner().as_ptr() == jl_typedslot_type
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
        jl_is_cpointer_type(t.inner().as_ptr().cast())
    }
}

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is an intrinsic.
pub struct Intrinsic;
impl_julia_typecheck!(Intrinsic, jl_intrinsic_type);

pub struct Concrete;
unsafe impl JuliaTypecheck for Concrete {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        (&*t.inner().as_ptr()).isconcretetype != 0
    }
}
