//! Managed type for `DataType`, which provides access to type properties.

use std::{ffi::CStr, marker::PhantomData, ptr::NonNull};

use jl_sys::{
    jl_abstractstring_type, jl_any_type, jl_anytuple_type, jl_argumenterror_type, jl_bool_type,
    jl_boundserror_type, jl_char_type, jl_const_type, jl_datatype_t, jl_datatype_type,
    jl_emptytuple_type, jl_errorexception_type, jl_expr_type, jl_field_index, jl_float16_type,
    jl_float32_type, jl_float64_type, jl_floatingpoint_type, jl_function_type,
    jl_has_free_typevars, jl_initerror_type, jl_int16_type, jl_int32_type, jl_int64_type,
    jl_int8_type, jl_loaderror_type, jl_methoderror_type, jl_module_type, jl_new_structv,
    jl_nothing_type, jl_number_type, jl_signed_type, jl_simplevector_type, jl_string_type,
    jl_symbol_type, jl_task_type, jl_tvar_type, jl_typeerror_type, jl_typename_str,
    jl_typename_type, jl_typeofbottom_type, jl_uint16_type, jl_uint32_type, jl_uint64_type,
    jl_uint8_type, jl_undefvarerror_type, jl_unionall_type, jl_uniontype_type, jl_voidpointer_type,
    jlrs_datatype_align, jlrs_datatype_first_ptr, jlrs_datatype_has_layout, jlrs_datatype_instance,
    jlrs_datatype_layout, jlrs_datatype_nfields, jlrs_datatype_parameters, jlrs_datatype_size,
    jlrs_datatype_super, jlrs_datatype_typename, jlrs_datatype_zeroinit, jlrs_field_isptr,
    jlrs_field_offset, jlrs_field_size, jlrs_get_fieldtypes, jlrs_is_concrete_type,
    jlrs_is_primitivetype, jlrs_isbits, jlrs_nparams,
};
#[julia_version(since = "1.7")]
use jl_sys::{jl_atomicerror_type, jl_vararg_type};
use jlrs_macros::julia_version;

use super::{type_name::TypeName, value::ValueData, Ref};
use crate::{
    catch::{catch_exceptions, unwrap_exc},
    convert::to_symbol::ToSymbol,
    data::{
        managed::{
            array::Array,
            private::ManagedPriv,
            simple_vector::SimpleVector,
            symbol::Symbol,
            type_var::TypeVar,
            union_all::UnionAll,
            value::{Value, ValueResult},
            Managed,
        },
        types::{construct_type::TypeVarEnv, typecheck::Typecheck},
    },
    error::{InstantiationError, JlrsResult},
    impl_julia_typecheck,
    memory::target::{unrooted::Unrooted, Target, TargetResult},
    private::Private,
};

/// Julia type information.
///
/// You can access a [`Value`]'s datatype by by calling [`Value::datatype`]. If a `DataType` is
/// concrete and not a subtype of `Array` a new instance can be created with
/// [`DataType::instantiate`]. To call a constructor, convert the `DataType` to a
/// `Value` with [`Managed::as_value`] and call it as a Julia function.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct DataType<'scope>(NonNull<jl_datatype_t>, PhantomData<&'scope ()>);

impl<'scope> DataType<'scope> {
    /// Returns the `TypeName` of this type.
    #[inline]
    pub fn type_name(self) -> TypeName<'scope> {
        // Safety: the pointer points to valid data, and the typename of a type never changes
        unsafe {
            let name = jlrs_datatype_typename(self.unwrap(Private));
            debug_assert!(!name.is_null());
            TypeName::wrap_non_null(NonNull::new_unchecked(name), Private)
        }
    }

    /// Returns the super-type of this type.
    #[inline]
    pub fn super_type(self) -> DataType<'scope> {
        // Safety: the pointer points to valid data, and the super-type of a type never changes
        unsafe {
            let super_ty = jlrs_datatype_super(self.unwrap(Private));
            debug_assert!(!super_ty.is_null());
            DataType::wrap_non_null(NonNull::new_unchecked(super_ty), Private)
        }
    }

    /// Returns the type parameters of this type.
    #[inline]
    pub fn parameters(self) -> SimpleVector<'scope> {
        // Safety: the pointer points to valid data and this data is const
        unsafe {
            let parameters = jlrs_datatype_parameters(self.unwrap(Private));
            debug_assert!(!parameters.is_null());
            SimpleVector::wrap_non_null(NonNull::new_unchecked(parameters), Private)
        }
    }

    /// Returns the number of type parameters.
    #[inline]
    pub fn n_parameters(self) -> usize {
        // Safety: the pointer points to valid data, the parameters field is never null
        unsafe { jlrs_nparams(self.unwrap(Private)) }
    }

    /// Returns the type parameter at position `idx`, or `None` if the index is out of bounds.
    #[inline]
    pub fn parameter(self, idx: usize) -> Option<Value<'scope, 'static>> {
        // Safety: the pointer points to valid data, the parameters field is never null
        unsafe {
            let unrooted = Unrooted::new();
            Some(self.parameters().data().get(unrooted, idx)?.as_value())
        }
    }

    /// Returns the type parameter at position `idx`.
    ///
    /// Safety: `idx` must be in-bounds and the parameter must not be a null pointer.
    #[inline]
    pub unsafe fn parameter_unchecked(self, idx: usize) -> Value<'scope, 'static> {
        let unrooted = Unrooted::new();
        self.parameters()
            .data()
            .get(unrooted, idx)
            .unwrap_unchecked()
            .as_value()
    }

    /// Returns `true` if this type has free type parameters.
    #[inline]
    pub fn has_free_type_vars(self) -> bool {
        unsafe { jl_has_free_typevars(self.unwrap(Private).cast()) != 0 }
    }

    /// Returns the field types of this type.
    #[inline]
    pub fn field_types(self) -> SimpleVector<'scope> {
        // Safety: the pointer points to valid data, the C API function is called with a valid argument
        unsafe {
            let field_types = jlrs_get_fieldtypes(self.unwrap(Private));
            debug_assert!(!field_types.is_null());
            SimpleVector::wrap_non_null(NonNull::new_unchecked(field_types), Private)
        }
    }

    /// Returns the field type of the field at position `idx`, or `None` if the index is out of
    /// bounds.
    #[inline]
    pub fn field_type(self, idx: usize) -> Option<Value<'scope, 'static>> {
        // Safety: the pointer points to valid data, the field_types field is never null
        unsafe {
            let unrooted = Unrooted::new();
            Some(self.field_types().data().get(unrooted, idx)?.as_value())
        }
    }

    /// Returns the field type of the field at position `idx` without performing a bounds check.
    ///
    /// Safety: `idx` must be in-bounds.
    #[inline]
    pub unsafe fn field_type_unchecked(self, idx: usize) -> Value<'scope, 'static> {
        let unrooted = Unrooted::new();
        self.field_types()
            .data()
            .get(unrooted, idx)
            .unwrap_unchecked()
            .as_value()
    }

    /// Returns the field names of this type.
    #[inline]
    pub fn field_names(self) -> SimpleVector<'scope> {
        // Safety: the pointer points to valid data, so it must have a TypeName.
        self.type_name().names()
    }

    /// Returns the name of the field at position `idx`.
    pub fn field_name(self, idx: usize) -> Option<Symbol<'scope>> {
        // Safety: the pointer points to valid data, so it must have a TypeName.
        unsafe {
            let unrooted = Unrooted::new();
            self.field_names()
                .typed_data_unchecked::<Symbol>()
                .get(unrooted, idx)
                .map(|s| s.as_managed())
        }
    }

    /// Returns the index of the field with the name `field_name`.
    pub fn field_index<N: ToSymbol>(self, field_name: N) -> Option<usize> {
        // Safety: the pointer points to valid data, the C API function is called with valid data
        let idx = unsafe {
            let sym = field_name.to_symbol_priv(Private);
            jl_field_index(self.unwrap(Private), sym.unwrap(Private), 0)
        };

        if idx < 0 {
            return None;
        }

        Some(idx as usize)
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
    #[inline]
    pub fn field_name_str(self, idx: usize) -> Option<&'scope str> {
        if let Some(sym) = self.field_name(idx) {
            return sym.as_str().ok();
        }

        None
    }

    /// Returns the instance if this type is a singleton.
    #[inline]
    pub fn instance(self) -> Option<Value<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let instance = jlrs_datatype_instance(self.unwrap(Private));
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

    #[julia_version(until = "1.8")]
    /// Returns the size of a value of this type in bytes.
    #[inline]
    pub fn size(self) -> Option<u32> {
        if self.is_abstract() {
            return None;
        }
        // Safety: the pointer points to valid data
        unsafe { Some(jlrs_datatype_size(self.unwrap(Private))) }
    }

    #[julia_version(since = "1.9")]
    /// Returns the size of a value of this type in bytes.
    #[inline]
    pub fn size(self) -> Option<u32> {
        unsafe {
            let t = self.unwrap(Private);
            if jlrs_datatype_has_layout(t) == 0 || jlrs_datatype_layout(t).is_null() {
                return None;
            }

            Some(jlrs_datatype_size(t))
        }
    }

    /// Returns true if this is an abstract type.
    #[inline]
    pub fn is_abstract(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { jl_sys::jlrs_datatype_abstract(self.unwrap(Private)) != 0 }
    }

    /// Returns true if this is a mutable type.
    #[inline]
    pub fn mutable(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { jl_sys::jlrs_datatype_mutable(self.unwrap(Private)) != 0 }
    }

    /// Returns true if this type can have instances
    #[inline]
    pub fn is_concrete_type(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { jlrs_is_concrete_type(self.as_value().unwrap(Private)) != 0 }
    }

    /// Returns true if this type is a bits-type.
    #[inline]
    pub fn is_bits(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { jlrs_isbits(self.unwrap(Private).cast()) != 0 }
    }

    /// Returns true if values of this type are zero-initialized.
    #[inline]
    pub fn zero_init(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { jlrs_datatype_zeroinit(self.unwrap(Private)) != 0 }
    }

    /// Returns true if a value of this type stores its data inline.
    #[inline]
    pub fn is_inline_alloc(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { jl_sys::jlrs_datatype_isinlinealloc(self.unwrap(Private)) != 0 }
    }

    /// Whether this is declared with 'primitive type' keyword (sized, no fields, and immutable)
    #[inline]
    pub fn is_primitive_type(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { jlrs_is_primitivetype(self.unwrap(Private).cast()) != 0 }
    }

    /// Performs the given typecheck on this type.
    #[inline]
    pub fn is<T: Typecheck>(self) -> bool {
        T::typecheck(self)
    }

    /// Returns the alignment of a value of this type in bytes.
    #[inline]
    pub fn align(self) -> Option<u16> {
        // Safety: the pointer points to valid data, if the layout is null the code
        // panics.
        if !self.has_layout() {
            return None;
        }

        unsafe { Some(jlrs_datatype_align(self.unwrap(Private))) }
    }

    /// Returns `true` if this type has a layout.
    #[inline]
    pub fn has_layout(self) -> bool {
        unsafe {
            jlrs_datatype_has_layout(self.unwrap(Private)) != 0
                && !jlrs_datatype_layout(self.unwrap(Private).cast()).is_null()
        }
    }

    /// Returns the size of a value of this type in bits.
    #[inline]
    pub fn n_bits(self) -> Option<u32> {
        Some(self.size()? * 8)
    }

    /// Returns the number of fields of a value of this type.
    #[inline]
    pub fn n_fields(self) -> Option<u32> {
        if !self.has_layout() {
            return None;
        }

        unsafe { Some(jlrs_datatype_nfields(self.unwrap(Private))) }
    }

    /// Returns the name of this type.
    #[inline]
    pub fn name(self) -> &'scope str {
        // Safety: the pointer points to valid data, so it must have a name. If it's not
        // a valid UTF-8 encoded string the code panics.
        unsafe {
            let name = jl_typename_str(self.unwrap(Private).cast());
            CStr::from_ptr(name).to_str().unwrap()
        }
    }

    /// Returns the size of the field at position `idx` in this type.
    pub fn field_size(self, idx: usize) -> Option<u32> {
        let n_fields = self.n_fields()?;
        if idx >= n_fields as usize {
            return None;
        }

        // Safety: the pointer points to valid data, and the field exists
        unsafe { Some(jlrs_field_size(self.unwrap(Private), idx as _)) }
    }

    /// Returns the size of the field at position `idx` in this type.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    #[inline]
    pub unsafe fn field_size_unchecked(self, idx: usize) -> u32 {
        jlrs_field_size(self.unwrap(Private), idx as _)
    }

    /// Returns the offset where the field at position `idx` is stored.
    pub fn field_offset(self, idx: usize) -> Option<u32> {
        let n_fields = self.n_fields()?;

        if idx >= n_fields as usize {
            return None;
        }

        // Safety: the pointer points to valid data, and the field exists
        unsafe { Some(jlrs_field_offset(self.unwrap(Private), idx as _)) }
    }

    /// Returns the offset where the field at position `idx` is stored.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    #[inline]
    pub unsafe fn field_offset_unchecked(self, idx: usize) -> u32 {
        jlrs_field_offset(self.unwrap(Private), idx as _)
    }

    /// Returns true if the field at position `idx` is stored as a pointer.
    pub fn is_pointer_field(self, idx: usize) -> Option<bool> {
        let n_fields = self.n_fields()?;
        if idx >= n_fields as usize {
            return None;
        }

        // Safety: the pointer points to valid data, and the field exists
        unsafe { Some(jlrs_field_isptr(self.unwrap(Private), idx as _) != 0) }
    }

    /// Returns true if the field at position `idx` is stored as a pointer.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    #[inline]
    pub unsafe fn is_pointer_field_unchecked(self, idx: usize) -> bool {
        jlrs_field_isptr(self.unwrap(Private), idx as _) != 0
    }

    #[julia_version(since = "1.7")]
    /// Returns true if the field at position `idx` is an atomic field.
    pub fn is_atomic_field(self, idx: usize) -> Option<bool> {
        let n_fields = self.n_fields()?;

        if idx >= n_fields as usize {
            return None;
        }

        // Safety: the pointer points to valid data, and the field exists
        unsafe { Some(self.is_atomic_field_unchecked(idx)) }
    }

    #[julia_version(since = "1.7")]
    /// Returns true if the field at position `idx` is an atomic field.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    #[inline]
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

    #[julia_version(since = "1.8")]
    /// Returns true if the field at position `idx` is a constant field.
    pub fn is_const_field(self, idx: usize) -> Option<bool> {
        let n_fields = self.n_fields()?;

        if idx >= n_fields as usize {
            return None;
        }

        // Safety: the pointer points to valid data, and the field exists
        unsafe { Some(self.is_const_field_unchecked(idx)) }
    }

    #[julia_version(since = "1.8")]
    /// Returns true if the field at position `idx` is a constant field.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    #[inline]
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
        if !tn.is_mutable() {
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
    ///
    /// This calls the function's `new` function. This functions returns an error if the given
    /// `DataType` isn't concrete or is an array type. For custom array types you must use
    /// [`Array::new_for`].
    ///
    /// To call a constructor of the type, convert it to a `Value` and call it as a function.
    pub fn instantiate<'target, 'value, 'data, V, Tgt>(
        self,
        target: Tgt,
        values: V,
    ) -> JlrsResult<ValueResult<'target, 'data, Tgt>>
    where
        Tgt: Target<'target>,
        V: AsRef<[Value<'value, 'data>]>,
    {
        // Safety: the pointer points to valid data, if an exception is thrown it's caught
        unsafe {
            if self.is::<Array>() {
                Err(InstantiationError::ArrayNotSupported)?;
            }

            let values = values.as_ref();
            let callback = || {
                jl_new_structv(
                    self.unwrap(Private),
                    values.as_ptr() as *mut _,
                    values.len() as _,
                )
            };

            let res = match catch_exceptions(callback, unwrap_exc) {
                Ok(ptr) => Ok(NonNull::new_unchecked(ptr)),
                Err(e) => Err(e),
            };

            Ok(target.result_from_ptr(res, Private))
        }
    }

    /// Create a new instance of this `DataType`, using `values` to set the fields.
    ///
    /// This calls the function's `new` function. This functions returns an error if the given
    /// `DataType` isn't concrete or is an array type. For custom array types you must use
    /// [`Array::new_for`].
    ///
    /// To call a constructor of the type, convert it to a `Value` and call it as a function.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    #[inline]
    pub unsafe fn instantiate_unchecked<'target, 'value, 'data, V, Tgt>(
        self,
        target: Tgt,
        values: V,
    ) -> ValueData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
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

    /// Returns `true` if this type has pointer fields.
    pub fn has_pointer_fields(self) -> Option<bool> {
        if !self.has_layout() {
            return None;
        }

        unsafe { Some(jlrs_datatype_first_ptr(self.unwrap(Private)) != -1) }
    }

    /// Wraps this type as a `UnionAll` if it has free `TypeVar`s, returns `self` otherwise.
    #[inline]
    pub fn rewrap<'target, Tgt: Target<'target>>(
        self,
        target: Tgt,
    ) -> ValueData<'target, 'static, Tgt> {
        UnionAll::rewrap(target, self)
    }
}

impl DataType<'_> {
    /// Returns `true` if the type depends on a type parameter outside its parameter list.
    pub fn has_indirect_typevar(self, tvar: TypeVar) -> bool {
        let params = self.parameters();
        let svec = params.data();
        unsafe {
            let unrooted = Unrooted::new();

            for pidx in 0..svec.len() {
                let param = svec.get(unrooted, pidx);
                let param = param.expect("encountered null param").as_value();
                if param.is::<TypeVar>() {
                    let param = param.cast_unchecked::<TypeVar>();
                    if param.has_indirect_typevar(tvar) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Wrap this type with an environment.
    pub fn wrap_with_env<'target, Tgt>(
        self,
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let svec = env.to_svec();
        let tvars = svec.data();
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let mut reusable_slot = frame.local_reusable_slot();
            unsafe {
                let mut out = self.root(&mut reusable_slot).as_value();

                for tidx in (0..tvars.len()).rev() {
                    let Some(tv) = tvars.get(&reusable_slot, tidx) else {
                        continue;
                    };

                    let tv = tv.as_value();

                    // rooted via env
                    debug_assert!(tv.is::<TypeVar>());
                    let tv = tv.cast_unchecked::<TypeVar>();

                    if self.as_value().has_typevar(tv) {
                        out = UnionAll::new_unchecked(&mut reusable_slot, tv, out).as_value();
                    } else if self.has_indirect_typevar(tv) {
                        out = UnionAll::new_unchecked(&mut reusable_slot, tv, out).as_value();
                    }
                }

                out.root(target)
            }
        })
    }
}

impl<'target> DataType<'target> {
    /// The type of the bottom type, `Union{}`.
    #[inline]
    pub fn typeofbottom_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typeofbottom_type), Private) }
    }

    /// The type `DataType`.
    #[inline]
    pub fn datatype_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_datatype_type), Private) }
    }

    /// The type `Union`.
    #[inline]
    pub fn uniontype_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uniontype_type), Private) }
    }

    /// The type `UnionAll`.
    #[inline]
    pub fn unionall_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_unionall_type), Private) }
    }

    /// The type `TypeVar`.
    #[inline]
    pub fn tvar_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_tvar_type), Private) }
    }

    /// The type `Any`.
    #[inline]
    pub fn any_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_any_type), Private) }
    }

    /// The type `TypeName`.
    #[inline]
    pub fn typename_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typename_type), Private) }
    }

    /// The type `Symbol`.
    #[inline]
    pub fn symbol_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_symbol_type), Private) }
    }

    /// The type `Core.Const`
    #[inline]
    pub fn const_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_const_type), Private) }
    }

    /// The type `SimpleVector`.
    #[inline]
    pub fn simplevector_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_simplevector_type), Private) }
    }

    /// The type `Tuple`.
    #[inline]
    pub fn anytuple_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_anytuple_type), Private) }
    }

    /// The type of an empty tuple.
    #[inline]
    pub fn emptytuple_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_emptytuple_type), Private) }
    }

    /// The type `Tuple`.
    #[inline]
    pub fn tuple_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_anytuple_type), Private) }
    }

    #[julia_version(since = "1.7")]
    /// The type `Vararg`.
    #[inline]
    pub fn vararg_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { DataType::wrap_non_null(NonNull::new_unchecked(jl_vararg_type), Private) }
    }

    /// The type `Function`.
    #[inline]
    pub fn function_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_function_type), Private) }
    }

    /// The type `Module`.
    #[inline]
    pub fn module_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_module_type), Private) }
    }

    /// The type `AbstractString`.
    #[inline]
    pub fn abstractstring_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_abstractstring_type), Private) }
    }

    /// The type `String`.
    #[inline]
    pub fn string_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_string_type), Private) }
    }

    /// The type `ErrorException`.
    #[inline]
    pub fn errorexception_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_errorexception_type), Private) }
    }

    /// The type `ArgumentError`.
    #[inline]
    pub fn argumenterror_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_argumenterror_type), Private) }
    }

    /// The type `LoadError`.
    #[inline]
    pub fn loaderror_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_loaderror_type), Private) }
    }

    /// The type `InitError`.
    #[inline]
    pub fn initerror_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_initerror_type), Private) }
    }

    /// The type `TypeError`.
    #[inline]
    pub fn typeerror_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_typeerror_type), Private) }
    }

    /// The type `MethodError`.
    #[inline]
    pub fn methoderror_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_methoderror_type), Private) }
    }

    /// The type `UndefVarError`.
    #[inline]
    pub fn undefvarerror_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_undefvarerror_type), Private) }
    }

    #[julia_version(since = "1.7")]
    /// The type `Core.AtomicError`.
    #[inline]
    pub fn atomicerror_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_atomicerror_type), Private) }
    }

    /// The type `BoundsError`.
    #[inline]
    pub fn boundserror_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_boundserror_type), Private) }
    }

    /// The type `Bool`.
    #[inline]
    pub fn bool_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_bool_type), Private) }
    }

    /// The type `Char`.
    #[inline]
    pub fn char_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_char_type), Private) }
    }

    /// The type `Int8`.
    #[inline]
    pub fn int8_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int8_type), Private) }
    }

    /// The type `UInt8`.
    #[inline]
    pub fn uint8_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint8_type), Private) }
    }

    /// The type `Int16`.
    #[inline]
    pub fn int16_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int16_type), Private) }
    }

    /// The type `UInt16`.
    #[inline]
    pub fn uint16_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint16_type), Private) }
    }

    /// The type `Int32`.
    #[inline]
    pub fn int32_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int32_type), Private) }
    }

    /// The type `UInt32`.
    #[inline]
    pub fn uint32_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint32_type), Private) }
    }

    /// The type `Int64`.
    #[inline]
    pub fn int64_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_int64_type), Private) }
    }

    /// The type `UInt64`.
    #[inline]
    pub fn uint64_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_uint64_type), Private) }
    }

    /// The type `Float16`.
    #[inline]
    pub fn float16_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_float16_type), Private) }
    }

    /// The type `Float32`.
    #[inline]
    pub fn float32_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_float32_type), Private) }
    }

    /// The type `Float64`.
    #[inline]
    pub fn float64_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_float64_type), Private) }
    }

    /// The type `AbstractFloat`.
    #[inline]
    pub fn floatingpoint_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_floatingpoint_type), Private) }
    }

    /// The type `Number`.
    #[inline]
    pub fn number_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_number_type), Private) }
    }

    /// The type `Nothing`.
    #[inline]
    pub fn nothing_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_nothing_type), Private) }
    }

    /// The type `Signed`.
    #[inline]
    pub fn signed_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_signed_type), Private) }
    }

    /// The type `Ptr{Nothing}`.
    #[inline]
    pub fn voidpointer_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_voidpointer_type), Private) }
    }

    /// The type `Task`.
    #[inline]
    pub fn task_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_task_type), Private) }
    }

    /// The type `Expr`.
    #[inline]
    pub fn expr_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_expr_type), Private) }
    }

    #[julia_version(since = "1.11")]
    /// The type `BFloat16`.
    #[inline]
    pub fn bfloat16_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_sys::jl_bfloat16_type), Private) }
    }

    /// The type `Ptr{UInt8}`.
    #[inline]
    pub fn uint8pointer_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'target>,
    {
        // Safety: global constant
        unsafe {
            Self::wrap_non_null(
                NonNull::new_unchecked(jl_sys::jl_uint8pointer_type),
                Private,
            )
        }
    }
}

impl<'scope> PartialEq for DataType<'scope> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_value() == other.as_value()
    }
}

impl<'scope, 'data> PartialEq<Value<'scope, 'data>> for DataType<'scope> {
    #[inline]
    fn eq(&self, other: &Value<'scope, 'data>) -> bool {
        self.as_value() == *other
    }
}

impl<'scope> Eq for DataType<'scope> {}
impl_debug!(DataType<'_>);
impl_julia_typecheck!(DataType<'frame>, jl_datatype_type, 'frame);

impl<'scope> ManagedPriv<'scope, '_> for DataType<'scope> {
    type Wraps = jl_datatype_t;
    type WithLifetimes<'target, 'da> = DataType<'target>;
    const NAME: &'static str = "DataType";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, ::std::marker::PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_construct_type_managed!(DataType, 1, jl_datatype_type);

/// A reference to a [`DataType`] that has not been explicitly rooted.
pub type DataTypeRef<'scope> = Ref<'scope, 'static, DataType<'scope>>;

/// A [`DataTypeRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`DataType`].
pub type DataTypeRet = Ref<'static, 'static, DataType<'static>>;

impl_valid_layout!(DataTypeRef, DataType, jl_datatype_type);

use crate::memory::target::TargetType;

/// `DataType` or `DataTypeRef`, depending on the target type `Tgt`.
pub type DataTypeData<'target, Tgt> =
    <Tgt as TargetType<'target>>::Data<'static, DataType<'target>>;

/// `JuliaResult<DataType>` or `JuliaResultRef<DataTypeRef>`, depending on the target type `Tgt`.
pub type DataTypeResult<'target, Tgt> = TargetResult<'target, 'static, DataType<'target>, Tgt>;

impl_ccall_arg_managed!(DataType, 1);
impl_into_typed!(DataType);
