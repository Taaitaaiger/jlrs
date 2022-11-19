//! Wrapper for `DataType`, which provides access to type properties.

use std::{
    ffi::{c_void, CStr},
    marker::PhantomData,
    ptr::NonNull,
};

use cfg_if::cfg_if;
use jl_sys::{
    jl_abstractslot_type, jl_abstractstring_type, jl_any_type, jl_anytuple_type, jl_argument_type,
    jl_argumenterror_type, jl_bool_type, jl_boundserror_type, jl_builtin_type, jl_char_type,
    jl_code_info_type, jl_code_instance_type, jl_const_type, jl_datatype_layout_t, jl_datatype_t,
    jl_datatype_type, jl_emptytuple_type, jl_errorexception_type, jl_expr_type, jl_field_index,
    jl_field_isptr, jl_field_offset, jl_field_size, jl_float16_type, jl_float32_type,
    jl_float64_type, jl_floatingpoint_type, jl_function_type, jl_get_fieldtypes, jl_globalref_type,
    jl_gotoifnot_type, jl_gotonode_type, jl_initerror_type, jl_int16_type, jl_int32_type,
    jl_int64_type, jl_int8_type, jl_intrinsic_type, jl_lineinfonode_type, jl_linenumbernode_type,
    jl_loaderror_type, jl_method_instance_type, jl_method_match_type, jl_method_type,
    jl_methoderror_type, jl_methtable_type, jl_module_type, jl_new_structv, jl_newvarnode_type,
    jl_nothing_type, jl_number_type, jl_partial_struct_type, jl_phicnode_type, jl_phinode_type,
    jl_pinode_type, jl_quotenode_type, jl_returnnode_type, jl_signed_type, jl_simplevector_type,
    jl_slotnumber_type, jl_ssavalue_type, jl_string_type, jl_symbol_type, jl_task_type,
    jl_tvar_type, jl_typedslot_type, jl_typeerror_type, jl_typemap_entry_type,
    jl_typemap_level_type, jl_typename_str, jl_typename_type, jl_typeofbottom_type, jl_uint16_type,
    jl_uint32_type, jl_uint64_type, jl_uint8_type, jl_undefvarerror_type, jl_unionall_type,
    jl_uniontype_type, jl_upsilonnode_type, jl_voidpointer_type, jl_weakref_type,
};
#[cfg(not(feature = "lts"))]
use jl_sys::{
    jl_atomicerror_type, jl_interconditional_type, jl_partial_opaque_type, jl_vararg_type,
};

use super::{simple_vector::SimpleVectorData, type_name::TypeName, value::ValueData, Ref};
use crate::{
    convert::to_symbol::ToSymbol,
    error::{AccessError, JlrsResult, CANNOT_DISPLAY_TYPE},
    impl_julia_typecheck,
    layout::typecheck::Typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{
        private::WrapperPriv,
        simple_vector::{SimpleVector, SimpleVectorRef},
        symbol::{Symbol, SymbolRef},
        value::Value,
        Wrapper,
    },
};

cfg_if! {
    if #[cfg(not(all(target_os = "windows", feature = "lts")))] {
        use super::array::Array;
        use super::value::ValueResult;
}
}

/// Julia type information. You can acquire a [`Value`]'s datatype by by calling
/// [`Value::datatype`]. If a `DataType` is concrete and not a subtype of `Array` a new instance
/// can be created with [`DataType::instantiate`]. This can also be achieved by converting the
/// `DataType` to a `Value` with [`Wrapper::as_value`] and calling it as a Julia function.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct DataType<'scope>(NonNull<jl_datatype_t>, PhantomData<&'scope ()>);

impl<'scope> DataType<'scope> {
    /*
    inspect(DataType):

    name: Core.TypeName (const)
    super: DataType (const)
    parameters: Core.SimpleVector (const)
    types: Core.SimpleVector (mut)
    instance: Any (const)
    layout: Ptr{Nothing} (mut)
    size: Int32 (mut)
    hash: Int32 (const)
    flags: UInt8 (mut)
    */

    /// Returns the `TypeName` of this type.
    pub fn type_name(self) -> TypeName<'scope> {
        // Safety: the pointer points to valid data, and the typename of a type never changes
        unsafe {
            let name = self.unwrap_non_null(Private).as_ref().name;
            debug_assert!(!name.is_null());
            TypeName::wrap_non_null(NonNull::new_unchecked(name), Private)
        }
    }

    /// Returns the super-type of this type.
    pub fn super_type(self) -> DataType<'scope> {
        // Safety: the pointer points to valid data, and the super-type of a type never changes
        unsafe {
            let super_ty = self.unwrap_non_null(Private).as_ref().super_;
            debug_assert!(!super_ty.is_null());
            DataType::wrap_non_null(NonNull::new_unchecked(super_ty), Private)
        }
    }

    /// Returns the type parameters of this type.
    pub fn parameters(self) -> SimpleVector<'scope> {
        // Safety: the pointer points to valid data and this data is const
        unsafe {
            let parameters = self.unwrap_non_null(Private).as_ref().parameters;
            debug_assert!(!parameters.is_null());
            SimpleVector::wrap_non_null(NonNull::new_unchecked(parameters), Private)
        }
    }

    /// Returns the number of type parameters.
    pub fn n_parameters(self) -> usize {
        // Safety: the pointer points to valid data, the parameters field is never null
        self.parameters().len()
    }

    /// Returns the type parameter at position `idx`, or `None` if the index is out of bounds.
    pub fn parameter<'target, T>(
        self,
        target: T,
        idx: usize,
    ) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data, the parameters field is never null
        unsafe {
            Some(
                self.parameters()
                    .data()
                    .as_slice()
                    .get(idx)?
                    .as_ref()?
                    .root(target),
            )
        }
    }

    /// Returns the field types of this type.
    pub fn field_types<'target, T>(self, target: T) -> SimpleVectorData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data, the C API function is called with a valid argument
        unsafe {
            let field_types = jl_get_fieldtypes(self.unwrap(Private));
            debug_assert!(!field_types.is_null());
            SimpleVectorRef::wrap(NonNull::new_unchecked(field_types)).root(target)
        }
    }

    /// Returns the field type of the field at position `idx`, or `None` if the index is out of
    /// bounds.
    pub fn field_type<'target, T>(
        self,
        target: T,
        idx: usize,
    ) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data, the field_types field is never null
        unsafe {
            Some(
                self.field_types(&target)
                    .wrapper()
                    .data()
                    .as_slice()
                    .get(idx)?
                    .as_ref()?
                    .root(target),
            )
        }
    }

    /// Returns the field type of the field at position `idx` without performing a bounds check.
    ///
    /// Safety: `idx` must be in-bounds.
    pub unsafe fn field_type_unchecked<'target, T>(
        self,
        target: T,
        idx: usize,
    ) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        Some(
            self.field_types(&target)
                .wrapper()
                .data()
                .as_slice()
                .get_unchecked(idx)
                .as_ref()?
                .root(target),
        )
    }

    /// Returns the field type of the field at position `idx`.
    // TODO
    pub fn field_type_concrete<'target, T>(
        self,
        target: T,
        idx: usize,
    ) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data, an assert checks that the types field
        // isn't null.
        unsafe {
            Some(
                self.field_types(&target)
                    .wrapper()
                    .data()
                    .as_slice()
                    .get(idx)?
                    .as_ref()?
                    .root(target),
            )
        }
    }

    /// Returns the field names of this type.
    pub fn field_names(self) -> SimpleVector<'scope> {
        // Safety: the pointer points to valid data, so it must have a TypeName.
        self.type_name().names()
    }

    /// Returns the name of the field at position `idx`.
    pub fn field_name(self, idx: usize) -> Option<Symbol<'scope>> {
        // Safety: the pointer points to valid data, so it must have a TypeName.
        unsafe {
            self.field_names()
                .typed_data_unchecked::<SymbolRef>()
                .as_slice()
                .get(idx)?
                .map(|s| s.wrapper())
        }
    }

    /// Returns the index of the field with the name `field_name`.
    pub fn field_index<N: ToSymbol>(self, field_name: N) -> JlrsResult<usize> {
        // Safety: the pointer points to valid data, the C API function is called with valid data
        let (sym, idx) = unsafe {
            let sym = field_name.to_symbol_priv(Private);
            let idx = jl_field_index(self.unwrap(Private), sym.unwrap(Private), 0);
            (sym, idx)
        };

        if idx < 0 {
            Err(AccessError::NoSuchField {
                type_name: self.display_string_or(CANNOT_DISPLAY_TYPE),
                field_name: sym.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
            })?;
        }

        Ok(idx as usize)
    }

    /// Returns the index of the field with the name `field_name`, if the field doesn't exist the
    /// result is `-1`.
    pub fn field_index_unchecked<N: ToSymbol>(self, field_name: N) -> i32 {
        // Safety: the pointer points to valid data, the C API function is called with valid data
        unsafe {
            let sym = field_name.to_symbol_priv(Private);
            jl_field_index(self.unwrap(Private), sym.unwrap(Private), 0)
        }
    }

    /// Returns the name of the field at position `idx`.
    pub fn field_name_str(self, idx: usize) -> Option<&'scope str> {
        if let Some(sym) = self.field_name(idx) {
            return sym.as_str().ok();
        }

        None
    }

    /// Returns the instance if this type is a singleton.
    pub fn instance(self) -> Option<Value<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let instance = self.unwrap_non_null(Private).as_ref().instance;
            if instance.is_null() {
                None
            } else {
                Some(Value::wrap_non_null(
                    NonNull::new_unchecked(instance),
                    Private,
                ))
            }
        }
    }

    // TODO: Allow using this information
    /// Returns a pointer to the layout of this `DataType`.
    pub fn layout(self) -> *const c_void {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().layout as _ }
    }

    /// Returns the size of a value of this type in bytes.
    pub fn size(self) -> u32 {
        // Safety: the pointer points to valid data
        cfg_if! {
            if #[cfg(not(any(feature = "beta", feature = "nightly")))] {
                unsafe {
                    self.unwrap_non_null(Private).as_ref().size as u32
                }
            } else {
                unsafe {
                    self.layout()
                        .cast::<jl_datatype_layout_t>()
                        .as_ref()
                        .unwrap()
                        .size

                }
            }
        }
    }

    /// Returns the hash of this type.
    pub fn hash(self) -> u32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().hash }
    }

    /// Returns true if this is an abstract type.
    pub fn is_abstract(self) -> bool {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().abstract_ != 0 }
            } else {
                // Safety: the pointer points to valid data, so it must have a TypeName.
                self.type_name().abstract_()
            }
        }
    }

    /// Returns true if this is a mutable type.
    pub fn mutable(self) -> bool {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().mutabl != 0 }
            } else {
                // Safety: the pointer points to valid data, so it must have a TypeName.
                self.type_name().mutabl()
            }
        }
    }

    /// Returns true if one or more of the type parameters has not been set.
    pub fn has_free_type_vars(self) -> bool {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().hasfreetypevars != 0 }
            } else {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().hasfreetypevars() != 0 }
            }
        }
    }

    /// Returns true if this type can have instances
    pub fn is_concrete_type(self) -> bool {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().isconcretetype != 0 }
            } else {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().isconcretetype() != 0 }
            }
        }
    }

    /// Returns true if this type is a dispatch, or leaf, tuple type.
    pub fn is_dispatch_tuple(self) -> bool {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().isdispatchtuple != 0 }
            } else {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().isdispatchtuple() != 0 }
            }
        }
    }

    /// Returns true if this type is a bits-type.
    pub fn is_bits(self) -> bool {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().isbitstype != 0 }
            } else {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().isbitstype() != 0 }
            }
        }
    }

    /// Returns true if values of this type are zero-initialized.
    pub fn zero_init(self) -> bool {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().zeroinit != 0 }
            } else {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().zeroinit() != 0 }
            }
        }
    }

    /// Returns true if a value of this type stores its data inline.
    pub fn is_inline_alloc(self) -> bool {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().isinlinealloc != 0 }
            } else {
                // Safety: the pointer points to valid data, so it must have a TypeName.
                unsafe {
                    self.type_name().mayinlinealloc()
                        && !self.unwrap_non_null(Private).as_ref().layout.is_null()
                }
            }
        }
    }

    /// If false, no value will have this type.
    pub fn has_concrete_subtype(self) -> bool {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().has_concrete_subtype != 0 }
            } else {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().has_concrete_subtype() != 0 }
            }
        }
    }

    /// If true, the type is stored in hash-based set cache (instead of linear cache).
    pub fn cached_by_hash(self) -> bool {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().cached_by_hash != 0 }
            } else {
                // Safety: the pointer points to valid data
                unsafe { self.unwrap_non_null(Private).as_ref().cached_by_hash() != 0 }
            }
        }
    }
}

impl<'scope> DataType<'scope> {
    /// Performs the given typecheck on this type.
    pub fn is<T: Typecheck>(self) -> bool {
        T::typecheck(self)
    }

    /// Returns the alignment of a value of this type in bytes.
    pub fn align(self) -> u16 {
        // Safety: the pointer points to valid data, if the layout is null the code
        // panics.
        unsafe {
            self.layout()
                .cast::<jl_datatype_layout_t>()
                .as_ref()
                .unwrap()
                .alignment
        }
    }

    /// Returns the size of a value of this type in bits.
    pub fn n_bits(self) -> u32 {
        self.size() * 8
    }

    /// Returns the number of fields of a value of this type.
    pub fn n_fields(self) -> u32 {
        // Safety: the pointer points to valid data, if the layout is null the code
        // panics.
        unsafe {
            self.layout()
                .cast::<jl_datatype_layout_t>()
                .as_ref()
                .unwrap()
                .nfields
        }
    }

    /// Returns the name of this type.
    pub fn name(self) -> &'scope str {
        // Safety: the pointer points to valid data, so it must have a name. If it's not
        // a valid UTF-8 encoded string the code panics.
        unsafe {
            let name = jl_typename_str(self.unwrap(Private).cast());
            CStr::from_ptr(name).to_str().unwrap()
        }
    }

    /// Returns the size of the field at position `idx` in this type.
    pub fn field_size(self, idx: usize) -> JlrsResult<u32> {
        if idx >= self.n_fields() as usize {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields: self.n_fields() as usize,
                value_type: self.display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        // Safety: the pointer points to valid data, and the field exists
        unsafe { Ok(jl_field_size(self.unwrap(Private), idx as _)) }
    }

    /// Returns the size of the field at position `idx` in this type.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn field_size_unchecked(self, idx: usize) -> u32 {
        jl_field_size(self.unwrap(Private), idx as _)
    }

    /// Returns the offset where the field at position `idx` is stored.
    pub fn field_offset(self, idx: usize) -> JlrsResult<u32> {
        if idx >= self.n_fields() as usize {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields: self.n_fields() as usize,
                value_type: self.display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        // Safety: the pointer points to valid data, and the field exists
        unsafe { Ok(jl_field_offset(self.unwrap(Private), idx as _)) }
    }

    /// Returns the offset where the field at position `idx` is stored.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn field_offset_unchecked(self, idx: usize) -> u32 {
        jl_field_offset(self.unwrap(Private), idx as _)
    }

    /// Returns true if the field at position `idx` is stored as a pointer.
    pub fn is_pointer_field(self, idx: usize) -> JlrsResult<bool> {
        if idx >= self.n_fields() as usize {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields: self.n_fields() as usize,
                value_type: self.display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        // Safety: the pointer points to valid data, and the field exists
        unsafe { Ok(jl_field_isptr(self.unwrap(Private), idx as _)) }
    }

    /// Returns true if the field at position `idx` is stored as a pointer.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn is_pointer_field_unchecked(self, idx: usize) -> bool {
        jl_field_isptr(self.unwrap(Private), idx as _)
    }

    #[cfg(not(feature = "lts"))]
    /// Returns true if the field at position `idx` is an atomic field.
    pub fn is_atomic_field(self, idx: usize) -> JlrsResult<bool> {
        if idx >= self.n_fields() as usize {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields: self.n_fields() as usize,
                value_type: self.display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        // Safety: the pointer points to valid data, and the field exists
        unsafe { Ok(self.is_atomic_field_unchecked(idx)) }
    }

    #[cfg(not(feature = "lts"))]
    /// Returns true if the field at position `idx` is an atomic field.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn is_atomic_field_unchecked(self, idx: usize) -> bool {
        /*
            const uint32_t *atomicfields = st->name->atomicfields;
            if (atomicfields != NULL) {
                if (atomicfields[i / 32] & (1 << (i % 32)))
                    return 1;
            }
            return 0;
        */
        let atomicfields = self.type_name().atomicfields();
        if atomicfields.is_null() {
            return false;
        }

        let isatomic = (*atomicfields.add(idx / 32)) & (1 << (idx % 32));
        isatomic != 0
    }

    #[cfg(not(feature = "lts"))]
    /// Returns true if the field at position `idx` is a constant field.
    pub fn is_const_field(self, idx: usize) -> JlrsResult<bool> {
        if idx >= self.n_fields() as usize {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields: self.n_fields() as usize,
                value_type: self.display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        // Safety: the pointer points to valid data, and the field exists
        unsafe { Ok(self.is_const_field_unchecked(idx)) }
    }

    #[cfg(not(feature = "lts"))]
    /// Returns true if the field at position `idx` is a constant field.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn is_const_field_unchecked(self, idx: usize) -> bool {
        /*
        jl_typename_t *tn = st->name;
        if (!tn->mutabl)
            return 1;
        const uint32_t *constfields = tn->constfields;
        if (constfields != NULL) {
            if (constfields[i / 32] & (1 << (i % 32)))
                return 1;
        }
        return 0;
        */
        let tn = self.type_name();
        if !tn.mutabl() {
            return true;
        }

        let constfields = tn.constfields();
        if constfields.is_null() {
            return false;
        }

        let isconst = (*constfields.add(idx / 32)) & (1 << (idx % 32));
        isconst != 0
    }

    /// Create a new instance of this `DataType`, using `values` to set the fields.
    /// This is essentially a more powerful version of [`Value::new`] that can instantiate
    /// arbitrary concrete `DataType`s, at the cost that each of its fields must have already been
    /// allocated as a `Value`. This functions returns an error if the given `DataType` isn't
    /// concrete or is an array type. For custom array types you must use [`Array::new_for`].
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn instantiate<'target, 'value, 'data, V, T>(
        self,
        target: T,
        values: V,
    ) -> JlrsResult<ValueResult<'target, 'data, T>>
    where
        T: Target<'target>,
        V: AsRef<[Value<'value, 'data>]>,
    {
        use std::mem::MaybeUninit;

        use jl_sys::jl_value_t;

        use crate::{catch::catch_exceptions, error::InstantiationError};

        // Safety: the pointer points to valid data, if an exception is thrown it's caught
        unsafe {
            if self.is::<Array>() {
                Err(InstantiationError::ArrayNotSupported)?;
            }

            let values = values.as_ref();
            let mut callback = |result: &mut MaybeUninit<*mut jl_value_t>| {
                let v = jl_new_structv(
                    self.unwrap(Private),
                    values.as_ptr() as *mut _,
                    values.len() as _,
                );

                result.write(v);
                Ok(())
            };

            let res = match catch_exceptions(&mut callback).unwrap() {
                Ok(ptr) => Ok(NonNull::new_unchecked(ptr)),
                Err(e) => Err(e.ptr()),
            };

            Ok(target.result_from_ptr(res, Private))
        }
    }

    /// Create a new instance of this `DataType`, using `values` to set the fields.
    /// This is essentially a more powerful version of [`Value::new`] that can instantiate
    /// arbitrary concrete `DataType`s, at the cost that each of its fields must have already been
    /// allocated as a `Value`.
    ///
    /// This method performs no checks whether or not the value can be constructed with these
    /// values.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn instantiate_unchecked<'target, 'value, 'data, V, T>(
        self,
        target: T,
        values: V,
    ) -> ValueData<'target, 'data, T>
    where
        T: Target<'target>,
        V: AsRef<[Value<'value, 'data>]>,
    {
        let values = values.as_ref();
        let value = jl_new_structv(
            self.unwrap(Private),
            values.as_ptr() as *mut _,
            values.len() as _,
        );

        target.data_from_ptr(NonNull::new_unchecked(value), Private)
    }

    pub fn has_pointer_fields(self) -> JlrsResult<bool> {
        // Safety: the pointer points to valid data, if the layout is null the code
        // panics.
        unsafe {
            Ok(self
                .layout()
                .cast::<jl_datatype_layout_t>()
                .as_ref()
                .unwrap()
                .first_ptr
                != -1)
        }
    }
}

impl<'base> DataType<'base> {
    /// The type of the bottom type, `Union{}`.
    pub fn typeofbottom_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typeofbottom_type), Private) }
    }

    /// The type `DataType`.
    pub fn datatype_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_datatype_type), Private) }
    }

    /// The type `Union`.
    pub fn uniontype_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uniontype_type), Private) }
    }

    /// The type `UnionAll`.
    pub fn unionall_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_unionall_type), Private) }
    }

    /// The type `TypeVar`.
    pub fn tvar_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_tvar_type), Private) }
    }

    /// The type `Any`.
    pub fn any_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_any_type), Private) }
    }

    /// The type `TypeName`.
    pub fn typename_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typename_type), Private) }
    }

    /// The type `Symbol`.
    pub fn symbol_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_symbol_type), Private) }
    }

    /// The type `SSAValue`.
    pub fn ssavalue_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_ssavalue_type), Private) }
    }

    /// The type `Slot`.
    pub fn abstractslot_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_abstractslot_type), Private) }
    }

    /// The type `SlotNumber`.
    pub fn slotnumber_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_slotnumber_type), Private) }
    }

    /// The type `TypedSlot`.
    pub fn typedslot_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typedslot_type), Private) }
    }

    /// The type `Core.Argument`
    pub fn argument_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_argument_type), Private) }
    }

    /// The type `Core.Const`
    pub fn const_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_const_type), Private) }
    }

    /// The type `Core.PartialStruct`
    pub fn partial_struct_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_partial_struct_type), Private) }
    }

    /// The type `Core.PartialOpaque`
    #[cfg(not(feature = "lts"))]
    pub fn partial_opaque_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_partial_opaque_type), Private) }
    }

    /// The type `Core.InterConditional`
    #[cfg(not(feature = "lts"))]
    pub fn interconditional_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_interconditional_type), Private) }
    }

    /// The type `MethodMatch`
    pub fn method_match_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_method_match_type), Private) }
    }

    /// The type `SimpleVector`.
    pub fn simplevector_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_simplevector_type), Private) }
    }

    /// The type `Tuple`.
    pub fn anytuple_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_anytuple_type), Private) }
    }

    /// The type of an empty tuple.
    pub fn emptytuple_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_emptytuple_type), Private) }
    }

    /// The type `Tuple`.
    pub fn tuple_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_anytuple_type), Private) }
    }

    /// The type `Vararg`.
    #[cfg(not(feature = "lts"))]
    pub fn vararg_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { DataType::wrap_non_null(NonNull::new_unchecked(jl_vararg_type), Private) }
    }

    /// The type `Function`.
    pub fn function_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_function_type), Private) }
    }

    /// The type `Builtin`.
    pub fn builtin_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_builtin_type), Private) }
    }

    /// The type `MethodInstance`.
    pub fn method_instance_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_method_instance_type), Private) }
    }

    /// The type `CodeInstance`.
    pub fn code_instance_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_code_instance_type), Private) }
    }

    /// The type `CodeInfo`.
    pub fn code_info_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_code_info_type), Private) }
    }

    /// The type `Method`.
    pub fn method_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_method_type), Private) }
    }

    /// The type `Module`.
    pub fn module_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_module_type), Private) }
    }

    /// The type `WeakRef`.
    pub fn weakref_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_weakref_type), Private) }
    }

    /// The type `AbstractString`.
    pub fn abstractstring_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_abstractstring_type), Private) }
    }

    /// The type `String`.
    pub fn string_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_string_type), Private) }
    }

    /// The type `ErrorException`.
    pub fn errorexception_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_errorexception_type), Private) }
    }

    /// The type `ArgumentError`.
    pub fn argumenterror_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_argumenterror_type), Private) }
    }

    /// The type `LoadError`.
    pub fn loaderror_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_loaderror_type), Private) }
    }

    /// The type `InitError`.
    pub fn initerror_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_initerror_type), Private) }
    }

    /// The type `TypeError`.
    pub fn typeerror_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typeerror_type), Private) }
    }

    /// The type `MethodError`.
    pub fn methoderror_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_methoderror_type), Private) }
    }

    /// The type `UndefVarError`.
    pub fn undefvarerror_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_undefvarerror_type), Private) }
    }

    /// The type `Core.AtomicError`.
    #[cfg(not(feature = "lts"))]
    pub fn atomicerror_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_atomicerror_type), Private) }
    }

    /// The type `LineInfoNode`.
    pub fn lineinfonode_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_lineinfonode_type), Private) }
    }

    /// The type `BoundsError`.
    pub fn boundserror_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_boundserror_type), Private) }
    }

    /// The type `Bool`.
    pub fn bool_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_bool_type), Private) }
    }

    /// The type `Char`.
    pub fn char_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_char_type), Private) }
    }

    /// The type `Int8`.
    pub fn int8_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int8_type), Private) }
    }

    /// The type `UInt8`.
    pub fn uint8_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint8_type), Private) }
    }

    /// The type `Int16`.
    pub fn int16_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int16_type), Private) }
    }

    /// The type `UInt16`.
    pub fn uint16_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint16_type), Private) }
    }

    /// The type `Int32`.
    pub fn int32_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int32_type), Private) }
    }

    /// The type `UInt32`.
    pub fn uint32_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint32_type), Private) }
    }

    /// The type `Int64`.
    pub fn int64_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int64_type), Private) }
    }

    /// The type `UInt64`.
    pub fn uint64_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint64_type), Private) }
    }

    /// The type `Float16`.
    pub fn float16_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_float16_type), Private) }
    }

    /// The type `Float32`.
    pub fn float32_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_float32_type), Private) }
    }

    /// The type `Float64`.
    pub fn float64_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_float64_type), Private) }
    }

    /// The type `AbstractFloat`.
    pub fn floatingpoint_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_floatingpoint_type), Private) }
    }

    /// The type `Number`.
    pub fn number_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_number_type), Private) }
    }

    /// The type `Nothing`.
    pub fn nothing_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_nothing_type), Private) }
    }

    /// The type `Signed`.
    pub fn signed_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_signed_type), Private) }
    }

    /// The type `Ptr{Nothing}`.
    pub fn voidpointer_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_voidpointer_type), Private) }
    }

    /// The type `Task`.
    pub fn task_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_task_type), Private) }
    }

    /// The type `Expr`.
    pub fn expr_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_expr_type), Private) }
    }

    /// The type `GlobalRef`.
    pub fn globalref_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_globalref_type), Private) }
    }

    /// The type `LineNumberNode`.
    pub fn linenumbernode_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_linenumbernode_type), Private) }
    }

    /// The type `GotoNode`.
    pub fn gotonode_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_gotonode_type), Private) }
    }

    /// The type `GotoIfNot`.
    pub fn gotoifnot_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_gotoifnot_type), Private) }
    }

    /// The type `ReturnNode`.
    pub fn returnnode_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_returnnode_type), Private) }
    }

    /// The type `PhiNode`.
    pub fn phinode_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_phinode_type), Private) }
    }

    /// The type `PiNode`.
    pub fn pinode_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_pinode_type), Private) }
    }

    /// The type `PhiCNode`.
    pub fn phicnode_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_phicnode_type), Private) }
    }

    /// The type `UpsilonNode`.
    pub fn upsilonnode_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_upsilonnode_type), Private) }
    }

    /// The type `QuoteNode`.
    pub fn quotenode_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_quotenode_type), Private) }
    }

    /// The type `NewVarNode`.
    pub fn newvarnode_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_newvarnode_type), Private) }
    }

    /// The type `Intrinsic`.
    pub fn intrinsic_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_intrinsic_type), Private) }
    }

    /// The type `MethodTable`.
    pub fn methtable_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_methtable_type), Private) }
    }

    /// The type `TypeMapLevel`.
    pub fn typemap_level_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typemap_level_type), Private) }
    }

    /// The type `TypeMapEntry`.
    pub fn typemap_entry_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typemap_entry_type), Private) }
    }
}

impl<'scope> PartialEq for DataType<'scope> {
    fn eq(&self, other: &Self) -> bool {
        self.as_value() == other.as_value()
    }
}

impl<'scope, 'data> PartialEq<Value<'scope, 'data>> for DataType<'scope> {
    fn eq(&self, other: &Value<'scope, 'data>) -> bool {
        self.as_value() == *other
    }
}

impl<'scope> Eq for DataType<'scope> {}
impl_debug!(DataType<'_>);
impl_julia_typecheck!(DataType<'frame>, jl_datatype_type, 'frame);

impl<'scope> WrapperPriv<'scope, '_> for DataType<'scope> {
    type Wraps = jl_datatype_t;
    type TypeConstructorPriv<'target, 'da> = DataType<'target>;
    const NAME: &'static str = "DataType";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, ::std::marker::PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// A reference to a [`DataType`] that has not been explicitly rooted.
pub type DataTypeRef<'scope> = Ref<'scope, 'static, DataType<'scope>>;
impl_valid_layout!(DataTypeRef, DataType);

use crate::memory::target::target_type::TargetType;

/// `DataType` or `DataTypeRef`, depending on the target type `T`.
pub type DataTypeData<'target, T> = <T as TargetType<'target>>::Data<'static, DataType<'target>>;

/// `JuliaResult<DataType>` or `JuliaResultRef<DataTypeRef>`, depending on the target type `T`.
pub type DataTypeResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, DataType<'target>>;
