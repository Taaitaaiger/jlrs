//! Wrapper for `Core.DataType`, which provides access to type properties.
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

use super::{
    array::Array, private::Wrapper as WrapperPriv, type_var::TypeVar, DataTypeRef, SimpleVectorRef,
    TypeNameRef, ValueRef, Wrapper,
};
use crate::impl_valid_layout;
use crate::layout::typecheck::{Concrete, Typecheck};
use crate::memory::traits::frame::Frame;
use crate::wrappers::ptr::symbol::Symbol;
use crate::wrappers::ptr::value::Value;
use crate::{
    error::{JlrsError, JlrsResult},
    memory::traits::scope::Scope,
};
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
    jl_intrinsic_type, jl_isbits, jl_lineinfonode_type, jl_linenumbernode_type, jl_loaderror_type,
    jl_method_instance_type, jl_method_type, jl_methoderror_type, jl_methtable_type,
    jl_module_type, jl_new_structv, jl_newvarnode_type, jl_nothing_type, jl_number_type,
    jl_phicnode_type, jl_phinode_type, jl_pinode_type, jl_quotenode_type, jl_signed_type,
    jl_simplevector_type, jl_slotnumber_type, jl_ssavalue_type, jl_string_type, jl_symbol_type,
    jl_task_type, jl_tvar_type, jl_typedslot_type, jl_typeerror_type, jl_typemap_entry_type,
    jl_typemap_level_type, jl_typename_str, jl_typename_type, jl_typeofbottom_type, jl_uint16_type,
    jl_uint32_type, jl_uint64_type, jl_uint8_type, jl_undefvarerror_type, jl_unionall_type,
    jl_uniontype_type, jl_upsilonnode_type, jl_voidpointer_type, jl_weakref_type,
};
use std::ffi::CStr;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::ptr::NonNull;
/// Julia type information. You can acquire a [`Value`]'s datatype by by calling
/// [`Value::datatype`].It can be used in combination with [`DataType::is`] and [`Value::is`], if
/// the check returns `true` the [`Value`] can be cast to `DataType`:
///
/// ```
/// # use jlrs::prelude::*;
/// # use jlrs::util::JULIA;
/// # fn main() {
/// # JULIA.with(|j| {
/// # let mut julia = j.borrow_mut();
/// julia.scope(|global, frame| {
///     let val = Value::new(&mut *frame, 1u8)?;
///     let typeof_func = Module::core(global).function(&mut *frame, "typeof")?;
///     let ty_val = typeof_func.call1(&mut *frame, val)?.unwrap();
///     assert!(ty_val.is::<DataType>());
///     assert!(ty_val.cast::<DataType>().is_ok());
///     Ok(())
/// }).unwrap();
/// # });
/// # }
/// ```
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct DataType<'frame>(NonNull<jl_datatype_t>, PhantomData<&'frame ()>);

impl<'frame> DataType<'frame> {
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
    pub fn type_name(self) -> TypeNameRef<'frame> {
        unsafe { TypeNameRef::wrap(self.unwrap_non_null(Private).as_ref().name) }
    }

    /// Returns the super type of this type.
    pub fn super_type(self) -> DataTypeRef<'frame> {
        unsafe {
            let sup = self.unwrap_non_null(Private).as_ref().super_;
            DataTypeRef::wrap(sup)
        }
    }

    /// Returns the type parameters of this type.
    pub fn parameters(self) -> SimpleVectorRef<'frame, TypeVar<'frame>> {
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().parameters) }
    }

    /// Returns the field types of this type.
    pub fn field_types(self) -> SimpleVectorRef<'frame> {
        unsafe { SimpleVectorRef::wrap(jl_get_fieldtypes(self.unwrap(Private))) }
    }

    /// Returns the field names of this type.
    pub fn field_names(self) -> SimpleVectorRef<'frame, Symbol<'frame>> {
        unsafe { SimpleVectorRef::wrap(jl_field_names(self.unwrap(Private))) }
    }

    /// Returns the instance if this type is a singleton.
    pub fn instance(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().instance) }
    }

    /// Returns the size of a value of this type in bytes.
    pub fn size(self) -> i32 {
        unsafe { jl_datatype_size(self.unwrap(Private)) }
    }

    /// Returns the number of initialized fields.
    pub fn n_initialized(self) -> i32 {
        unsafe { self.unwrap_non_null(Private).as_ref().ninitialized }
    }

    /// Returns the hash of this type.
    pub fn hash(self) -> u32 {
        unsafe { self.unwrap_non_null(Private).as_ref().hash }
    }

    /// Returns true if this is an abstract type.
    pub fn is_abstract(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().abstract_ != 0 }
    }

    /// Returns true if this is a mutable type.
    pub fn mutable(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().mutabl != 0 }
    }

    /// Returns true if one or more of the type parameters has not been set.
    pub fn has_free_type_vars(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().hasfreetypevars != 0 }
    }

    /// Returns true if this type can have instances
    pub fn is_concrete_type(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().isconcretetype != 0 }
    }

    /// Returns true if this type is a dispatch, or leaf, tuple type.
    pub fn is_dispatch_tuple(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().isdispatchtuple != 0 }
    }

    /// Returns true if this type is a bits-type.
    pub fn is_bits(self) -> bool {
        unsafe { jl_isbits(self.unwrap(Private).cast()) }
    }

    /// Returns true if one or more fields require zero-initialization.
    pub fn zero_init(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().zeroinit != 0 }
    }

    /// Returns true if a value of this type stores its data inline.
    pub fn is_inline_alloc(self) -> bool {
        unsafe { jl_datatype_isinlinealloc(self.unwrap(Private)) != 0 }
    }

    /// If false, no value will have this type.
    pub fn has_concrete_subtype(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().has_concrete_subtype != 0 }
    }

    /// stored in hash-based set cache (instead of linear cache)
    pub fn cached_by_hash(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().cached_by_hash != 0 }
    }
}

impl<'frame> DataType<'frame> {
    /// Performs the given typecheck.
    pub fn is<T: Typecheck>(self) -> bool {
        unsafe { T::typecheck(self) }
    }

    /// Returns the alignment of a value of this type in bytes.
    pub fn align(self) -> u16 {
        unsafe { jl_datatype_align(self.unwrap(Private)) }
    }

    /// Returns the size of a value of this type in bits.
    pub fn n_bits(self) -> i32 {
        unsafe { jl_datatype_nbits(self.unwrap(Private)) }
    }

    /// Returns the number of fields of a value of this type.
    pub fn n_fields(self) -> u32 {
        unsafe { jl_datatype_nfields(self.unwrap(Private)) }
    }

    /// Returns the name of this type.
    pub fn name(self) -> &'frame str {
        unsafe {
            let name = jl_typename_str(self.unwrap(Private).cast());
            CStr::from_ptr(name).to_str().unwrap()
        }
    }

    /// Returns the size of the field at position `idx` in this type.
    pub fn field_size(self, idx: usize) -> u32 {
        unsafe { jl_field_size(self.unwrap(Private), idx as _) }
    }

    /// Returns the offset where the field at position `idx` is stored.
    pub fn field_offset(self, idx: usize) -> u32 {
        unsafe { jl_field_offset(self.unwrap(Private), idx as _) }
    }

    /// Returns true if the field at position `idx` is a pointer.
    pub fn is_pointer_field(self, idx: usize) -> bool {
        unsafe { jl_field_isptr(self.unwrap(Private), idx as _) }
    }

    /// Intantiate a value of this `DataType` with the given values. Returns an error if the type
    /// is not concrete.
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
                self.unwrap(Private),
                values.as_mut_ptr().cast(),
                values.len() as _,
            );
            scope.value(NonNull::new_unchecked(value), Private)
        }
    }
}

impl<'base> DataType<'base> {
    /// The type of the bottom type, `Union{}`.
    pub fn typeofbottom_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typeofbottom_type), Private) }
    }

    /// The type `DataType`.
    pub fn datatype_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_datatype_type), Private) }
    }

    /// The type `Union`.
    pub fn uniontype_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uniontype_type), Private) }
    }

    /// The type `UnionAll`.
    pub fn unionall_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_unionall_type), Private) }
    }

    /// The type `TypeVar`.
    pub fn tvar_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_tvar_type), Private) }
    }

    /// The type `Any`.
    pub fn any_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_any_type), Private) }
    }

    /// The type `TypeName`.
    pub fn typename_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typename_type), Private) }
    }

    /// The type `Symbol`.
    pub fn symbol_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_symbol_type), Private) }
    }

    /// The type `Core.SSAValue`.
    pub fn ssavalue_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_ssavalue_type), Private) }
    }

    /// The type `Slot`.
    pub fn abstractslot_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_abstractslot_type), Private) }
    }

    /// The type `SlotNumber`.
    pub fn slotnumber_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_slotnumber_type), Private) }
    }

    /// The type `TypedSlot`.
    pub fn typedslot_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typedslot_type), Private) }
    }

    /// The type `SimpleVector`, or `SVec`.
    pub fn simplevector_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_simplevector_type), Private) }
    }

    /// The type `Tuple`.
    pub fn anytuple_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_anytuple_type), Private) }
    }

    /// The type `Tuple`.
    pub fn tuple_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_anytuple_type), Private) }
    }

    /// The type of an empty tuple.
    pub fn emptytuple_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_emptytuple_type), Private) }
    }

    /// The type `Function`.
    pub fn function_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_function_type), Private) }
    }

    /// The type `Builtin`.
    pub fn builtin_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_builtin_type), Private) }
    }

    /// The type `MethodInstance`.
    pub fn method_instance_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_method_instance_type), Private) }
    }

    /// The type `CodeInstance`.
    pub fn code_instance_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_code_instance_type), Private) }
    }

    /// The type `CodeInfo`.
    pub fn code_info_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_code_info_type), Private) }
    }

    /// The type `Method`.
    pub fn method_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_method_type), Private) }
    }

    /// The type `Module`.
    pub fn module_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_module_type), Private) }
    }

    /// The type `WeakRef`.
    pub fn weakref_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_weakref_type), Private) }
    }

    /// The type `AbstractString`.
    pub fn abstractstring_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_abstractstring_type), Private) }
    }

    /// The type `String`.
    pub fn string_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_string_type), Private) }
    }

    /// The type `ErrorException`.
    pub fn errorexception_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_errorexception_type), Private) }
    }

    /// The type `ArgumentError`.
    pub fn argumenterror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_argumenterror_type), Private) }
    }

    /// The type `LoadError`.
    pub fn loaderror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_loaderror_type), Private) }
    }

    /// The type `InitError`.
    pub fn initerror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_initerror_type), Private) }
    }

    /// The type `TypeError`.
    pub fn typeerror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typeerror_type), Private) }
    }

    /// The type `MethodError`.
    pub fn methoderror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_methoderror_type), Private) }
    }

    /// The type `UndefVarError`.
    pub fn undefvarerror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_undefvarerror_type), Private) }
    }

    /// The type `LineInfoNode`.
    pub fn lineinfonode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_lineinfonode_type), Private) }
    }

    /// The type `BoundsError`.
    pub fn boundserror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_boundserror_type), Private) }
    }

    /// The type `Bool`.
    pub fn bool_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_bool_type), Private) }
    }

    /// The type `Char`.
    pub fn char_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_char_type), Private) }
    }

    /// The type `Int8`.
    pub fn int8_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int8_type), Private) }
    }

    /// The type `UInt8`.
    pub fn uint8_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint8_type), Private) }
    }

    /// The type `Int16`.
    pub fn int16_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int16_type), Private) }
    }

    /// The type `UInt16`.
    pub fn uint16_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint16_type), Private) }
    }

    /// The type `Int32`.
    pub fn int32_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int32_type), Private) }
    }

    /// The type `UInt32`.
    pub fn uint32_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint32_type), Private) }
    }

    /// The type `Int64`.
    pub fn int64_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int64_type), Private) }
    }

    /// The type `UInt64`.
    pub fn uint64_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint64_type), Private) }
    }

    /// The type `Float16`.
    pub fn float16_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_float16_type), Private) }
    }

    /// The type `Float32`.
    pub fn float32_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_float32_type), Private) }
    }

    /// The type `Float64`.
    pub fn float64_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_float64_type), Private) }
    }

    /// The type `AbstractFloat`.
    pub fn floatingpoint_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_floatingpoint_type), Private) }
    }

    /// The type `Number`.
    pub fn number_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_number_type), Private) }
    }

    /// The type `Nothing`.
    pub fn nothing_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_nothing_type), Private) }
    }

    /// The type `Signed`.
    pub fn signed_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_signed_type), Private) }
    }

    /// The type `Ptr{Nothing}`.
    pub fn voidpointer_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_voidpointer_type), Private) }
    }

    /// The type `Task`.
    pub fn task_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_task_type), Private) }
    }

    /// The type `Expr`.
    pub fn expr_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_expr_type), Private) }
    }

    /// The type `GlobalRef`.
    pub fn globalref_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_globalref_type), Private) }
    }

    /// The type `LineNumberNode`.
    pub fn linenumbernode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_linenumbernode_type), Private) }
    }

    /// The type `GotoNode`.
    pub fn gotonode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_gotonode_type), Private) }
    }

    /// The type `PhiNode`.
    pub fn phinode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_phinode_type), Private) }
    }

    /// The type `PiNode`.
    pub fn pinode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_pinode_type), Private) }
    }

    /// The type `PhiCNode`.
    pub fn phicnode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_phicnode_type), Private) }
    }

    /// The type `UpsilonNode`.
    pub fn upsilonnode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_upsilonnode_type), Private) }
    }

    /// The type `QuoteNode`.
    pub fn quotenode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_quotenode_type), Private) }
    }

    /// The type `NewVarNode`.
    pub fn newvarnode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_newvarnode_type), Private) }
    }

    /// The type `Intrinsic`.
    pub fn intrinsic_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_intrinsic_type), Private) }
    }

    /// The type `MethodTable`.
    pub fn methtable_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_methtable_type), Private) }
    }

    /// The type `TypeMapLevel`.
    pub fn typemap_level_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typemap_level_type), Private) }
    }

    /// The type `TypeMapEntry`.
    pub fn typemap_entry_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typemap_entry_type), Private) }
    }
}

impl<'scope> PartialEq for DataType<'scope> {
    fn eq(&self, other: &Self) -> bool {
        self.as_value().egal(other.as_value())
    }
}

impl<'scope, 'data> PartialEq<Value<'scope, 'data>> for DataType<'scope> {
    fn eq(&self, other: &Value<'scope, 'data>) -> bool {
        self.as_value().egal(*other)
    }
}

impl<'scope> Eq for DataType<'scope> {}

impl<'frame, 'data> Debug for DataType<'frame> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("DataType").field(&self.name()).finish()
    }
}

impl_valid_layout!(DataType<'frame>, 'frame);

impl<'scope> WrapperPriv<'scope, '_> for DataType<'scope> {
    type Internal = jl_datatype_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, ::std::marker::PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
