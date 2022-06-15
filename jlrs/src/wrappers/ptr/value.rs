//! Wrapper for arbitrary Julia data.
//!
//! Julia data returned by the C API is often returned as a pointer to `jl_value_t`, which is
//! an opaque type. This pointer is wrapped in jlrs by [`Value`]. The layout of the data that is
//! pointed to depends on its underlying type. Julia guarantees that the data is preceded in
//! memory by a header which contains a pointer to the data's type information, its [`DataType`].
//!
//! For example, if the `DataType` is `UInt8`, the pointer points to a `u8`. If the
//! `DataType` is some Julia array type like `Array{Int, 2}`, the pointer points to
//! Julia's internal array type, `jl_array_t`. In the first case tha value can be unboxed as a
//! `u8`, in the second case it can be cast to [`Array`] or [`TypedArray<isize>`].
//!
//! The `Value` wrapper is very commonly used in jlrs. A `Value` can be called as a Julia
//! function, the arguments such a function takes are all `Value`s, and it will return either a
//! `Value` or an exception which is also a `Value`. This wrapper also provides methods to create
//! new `Value`s, access their fields, cast them to the appropriate pointer wrapper type, and
//! unbox their contents.
//!
//! One special kind of value is the `NamedTuple`. You will need to create values of this type in
//! order to call functions with keyword arguments. The macro [`named_tuple`] is defined in this
//! module which provides an easy way to create values of this type.
//!
//! [`TypedArray<isize>`]: crate::wrappers::ptr::array::TypedArray

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    ($name:expr => $value:expr) => {
        2
    };
    ($name:expr => $value:expr, $($rest:tt)+) => {
        count!(2, $($rest)+)
    };
    ($n:expr, $name:expr => $value:expr) => {
        $n + 1
    };
    ($n:expr, $name:expr => $value:expr, $($rest:tt)+) => {
        count!($n + 1, $($rest)+)
    };
}

/// Create a new named tuple. You will need a named tuple to call functions with keyword
/// arguments.
///
/// Example:
///
/// ```
/// # use jlrs::prelude::*;
/// # use jlrs::util::JULIA;
/// # fn main() {
/// # JULIA.with(|j| {
/// # let mut julia = j.borrow_mut();
/// // Three slots; two for the inputs and one for the output.
/// julia.scope_with_capacity(3, |global, frame| {
///     // Create the two arguments, each value requires one slot
///     let i = Value::new(&mut *frame, 2u64)?;
///     let j = Value::new(&mut *frame, 1u32)?;
///
///     let _nt = named_tuple!(&mut *frame, "i" => i, "j" => j)?;
///
///     Ok(())
/// }).unwrap();
/// # });
/// # }
/// ```
#[macro_export]
macro_rules! named_tuple {
    ($frame:expr, $name:expr => $value:expr) => {
        $crate::wrappers::ptr::value::Value::new_named_tuple($frame, &mut [$name], &mut [$value])
    };
    ($frame:expr, $name:expr => $value:expr, $($rest:tt)+) => {
        {
            let n = $crate::count!($($rest)+);
            let mut names = ::smallvec::SmallVec::<[_; $crate::wrappers::ptr::value::MAX_SIZE]>::with_capacity(n);
            let mut values = ::smallvec::SmallVec::<[_; $crate::wrappers::ptr::value::MAX_SIZE]>::with_capacity(n);

            names.push($name);
            values.push($value);
            $crate::named_tuple!($frame, &mut names, &mut values, $($rest)+)
        }
    };
    ($frame:expr, $names:expr, $values:expr, $name:expr => $value:expr, $($rest:tt)+) => {
        {
            $names.push($name);
            $values.push($value);
            named_tuple!($frame, $names, $values, $($rest)+)
        }
    };
    ($frame:expr, $names:expr, $values:expr, $name:expr => $value:expr) => {
        {
            $names.push($name);
            $values.push($value);
            $crate::wrappers::ptr::value::Value::new_named_tuple($frame, $names, $values)
        }
    };
}

use crate::{
    call::{Call, CallExt, WithKeywords},
    convert::{into_julia::IntoJulia, to_symbol::ToSymbol, unbox::Unbox},
    error::{
        AccessError, IOError, InstantiationError, JlrsError, JlrsResult, JuliaResult,
        JuliaResultRef, TypeError, CANNOT_DISPLAY_TYPE,
    },
    impl_debug,
    layout::{
        field_index::FieldIndex,
        typecheck::{NamedTuple, Typecheck},
        valid_layout::ValidLayout,
    },
    memory::{
        frame::Frame,
        get_tls,
        global::Global,
        output::Output,
        scope::{PartialScope, Scope},
    },
    private::Private,
    wrappers::ptr::{
        array::Array,
        datatype::DataType,
        module::Module,
        private::WrapperPriv,
        string::JuliaString,
        symbol::Symbol,
        union::{nth_union_component, Union},
        union_all::UnionAll,
        DataTypeRef, ValueRef, Wrapper,
    },
};
use cfg_if::cfg_if;
use jl_sys::{
    jl_an_empty_string, jl_an_empty_vec_any, jl_apply_type, jl_array_any_type, jl_array_int32_type,
    jl_array_symbol_type, jl_array_typetagdata, jl_array_uint8_type, jl_astaggedvalue,
    jl_bottom_type, jl_call, jl_call0, jl_call1, jl_call2, jl_call3, jl_diverror_exception,
    jl_egal, jl_emptytuple, jl_eval_string, jl_exception_occurred, jl_false, jl_field_index,
    jl_field_isptr, jl_gc_add_finalizer, jl_gc_add_ptr_finalizer, jl_get_nth_field,
    jl_get_nth_field_noalloc, jl_interrupt_exception, jl_isa, jl_memory_exception, jl_nothing,
    jl_object_id, jl_readonlymemory_exception, jl_set_nth_field, jl_stackovf_exception,
    jl_stderr_obj, jl_stdout_obj, jl_subtype, jl_true, jl_typeof_str, jl_undefref_exception,
    jl_value_t,
};
use std::{
    ffi::{c_void, CStr, CString},
    marker::PhantomData,
    mem::MaybeUninit,
    path::Path,
    ptr::NonNull,
    sync::atomic::Ordering,
    usize,
};

#[cfg(not(all(target_os = "windows", feature = "lts")))]
use jl_sys::{jlrs_apply_type, jlrs_result_tag_t_JLRS_RESULT_ERR, jlrs_set_nth_field};

cfg_if! {
    if #[cfg(any(not(feature = "lts"), feature = "all-features-override"))] {
        use jl_sys::{jlrs_lock, jlrs_unlock};

        use std::{
            ptr::null_mut,
            sync::atomic::{AtomicPtr, AtomicU16, AtomicU32, AtomicU64, AtomicU8},
        };
    }
}

/// In some cases it's necessary to place one or more arguments in front of the arguments a
/// function is called with. Examples include the `named_tuple` macro and `Value::call_async`.
/// If they are called with fewer than `MAX_SIZE` arguments (including the added arguments), no
/// heap allocation is required to store them.
pub const MAX_SIZE: usize = 8;

/// See the [module-level documentation] for more information.
///
/// A `Value` is a wrapper around a non-null pointer to some data owned by the Julia garbage
/// collector, it has two lifetimes: `'scope` and `'data`. The first of these ensures that a
/// `Value` can only be used while it's rooted, the second accounts for data borrowed from Rust.
/// The only way to borrow data from Rust is to create an Julia array that borrows its contents
///  by calling [`Array::from_slice`]; if a Julia function is called with such an array as an
/// argument the result will inherit the second lifetime of the borrowed data to ensure that
/// such a `Value` can only be used while the borrow is active.
#[repr(transparent)]
#[derive(Copy, Clone, Eq)]
pub struct Value<'scope, 'data>(
    NonNull<jl_value_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

impl PartialEq for Value<'_, '_> {
    fn eq(&self, other: &Value<'_, '_>) -> bool {
        self.egal(*other)
    }
}

/// # Create new `Value`s
///
/// Several methods are available to create new values. The simplest of these is [`Value::new`],
/// which can be used to convert relatively simple data from Rust to Julia. Data that can be
/// converted this way must implement [`IntoJulia`], which is the case for types like the
/// primitive number types. This trait is also automatically derived by JlrsReflect.jl for types
/// that are trivially guaranteed to be bits-types: the type must have no type parameters, no
/// unions, and all fields must be immutable bits-types themselves.
impl Value<'_, '_> {
    /// Create a new Julia value, any type that implements [`IntoJulia`] can be converted using
    /// this function.
    pub fn new<'target, V, S>(scope: S, value: V) -> JlrsResult<Value<'target, 'static>>
    where
        V: IntoJulia,
        S: PartialScope<'target>,
    {
        let global = scope.global();
        let v = value.into_julia(global).ptr();
        debug_assert!(!v.is_null());
        unsafe { scope.value(NonNull::new_unchecked(v), Private) }
    }

    /// Create a new Julia value, any type that implements [`IntoJulia`] can be converted using
    /// this function. Unlike [`Value::new`] this method doesn't root the allocated value.
    pub fn new_unrooted<'global, V>(global: Global<'global>, value: V) -> ValueRef<'global, 'static>
    where
        V: IntoJulia,
    {
        value.into_julia(global)
    }

    /// Create a new named tuple, you should use the `named_tuple` macro rather than this method.
    pub fn new_named_tuple<'target, 'current, 'value, 'data, S, F, N, T, V>(
        scope: S,
        field_names: N,
        values: V,
    ) -> JlrsResult<Value<'target, 'data>>
    where
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
        N: AsRef<[T]>,
        T: ToSymbol,
        V: AsRef<[Value<'value, 'data>]>,
    {
        let global = scope.global();
        let (output, scope) = scope.split()?;
        scope.scope_with_capacity(4, |frame| unsafe {
            let field_names = field_names.as_ref();
            let values_m = values.as_ref();

            let n_names = field_names.len();
            let n_values = values_m.len();

            if n_names != n_values {
                Err(InstantiationError::NamedTupleSizeMismatch { n_names, n_values })?;
            }

            let symbol_ty = DataType::symbol_type(global).as_value();
            let mut symbol_type_vec = vec![symbol_ty; n_names];

            let mut field_names_vec = field_names
                .iter()
                .map(|name| name.to_symbol_priv(Private).as_value())
                .collect::<smallvec::SmallVec<[_; MAX_SIZE]>>();

            let names = DataType::anytuple_type(global)
                .as_value()
                .apply_type_unchecked(&mut *frame, &mut symbol_type_vec)?
                .cast::<DataType>()?
                .instantiate_unchecked(&mut *frame, &mut field_names_vec)?;

            let mut field_types_vec = values_m
                .iter()
                .copied()
                .map(|val| val.datatype().as_value())
                .collect::<smallvec::SmallVec<[_; MAX_SIZE]>>();

            let field_type_tup = DataType::anytuple_type(global)
                .as_value()
                .apply_type_unchecked(&mut *frame, &mut field_types_vec)?;

            let ty = UnionAll::namedtuple_type(global)
                .as_value()
                .apply_type_unchecked(&mut *frame, &mut [names, field_type_tup])?
                .cast::<DataType>()?;

            ty.instantiate_unchecked(output, values)
        })
    }

    /// Apply the given types to `self`.
    ///
    /// If `self` is the [`DataType`] `anytuple_type`, calling this method will return a new
    /// tuple type with the given types as its field types. If it is the [`DataType`]
    /// `uniontype_type`, calling this method is equivalent to calling [`Union::new`]. If
    /// the value is a `UnionAll`, the given types will be applied and the resulting type is
    /// returned.
    ///
    /// If the types can't be applied to `self` this methods catches and returns the exception.
    ///
    /// [`Union::new`]: crate::wrappers::ptr::union::Union::new
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn apply_type<'target, 'value, 'data, V, S>(
        self,
        scope: S,
        types: V,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
        V: AsRef<[Value<'value, 'data>]>,
    {
        unsafe {
            let types = types.as_ref();
            let applied =
                jlrs_apply_type(self.unwrap(Private), types.as_ptr() as *mut _, types.len());

            if applied.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                scope.call_result(Err(NonNull::new_unchecked(applied.data)), Private)
            } else {
                scope.call_result(Ok(NonNull::new_unchecked(applied.data)), Private)
            }
        }
    }

    /// Apply the given types to `self`.
    ///
    /// If `self` is the [`DataType`] `anytuple_type`, calling this method will return a new
    /// tuple type with the given types as its field types. If it is the [`DataType`]
    /// `uniontype_type`, calling this method is equivalent to calling [`Union::new`]. If
    /// the value is a `UnionAll`, the given types will be applied and the resulting type is
    /// returned.
    ///
    /// If an exception is thrown it isn't caught
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    ///
    /// [`Union::new`]: crate::wrappers::ptr::union::Union::new
    pub unsafe fn apply_type_unchecked<'target, 'value, 'data, S, V>(
        self,
        scope: S,
        types: V,
    ) -> JlrsResult<Value<'target, 'data>>
    where
        S: PartialScope<'target>,
        V: AsRef<[Value<'value, 'data>]>,
    {
        let types = types.as_ref();
        let applied = jl_apply_type(self.unwrap(Private), types.as_ptr() as *mut _, types.len());
        scope.value(NonNull::new_unchecked(applied), Private)
    }
}

/// # Type information
///
/// Every value is guaranteed to have a [`DataType`]. This contains all of the value's type
/// information.
impl<'scope, 'data> Value<'scope, 'data> {
    /// Returns the `DataType` of this value.
    pub fn datatype(self) -> DataType<'scope> {
        unsafe {
            let header = NonNull::new_unchecked(jl_astaggedvalue(self.unwrap(Private)))
                .as_ref()
                .__bindgen_anon_1
                .header;
            let ptr = (header & !15usize) as *mut jl_value_t;
            DataType::wrap(ptr.cast(), Private)
        }
    }

    /// Returns the name of this value's [`DataType`] as a string slice.
    pub fn datatype_name(self) -> JlrsResult<&'scope str> {
        unsafe {
            let type_name = jl_typeof_str(self.unwrap(Private));
            let type_name_ref = CStr::from_ptr(type_name);
            Ok(type_name_ref.to_str().map_err(JlrsError::other)?)
        }
    }
}

/// # Type checking
///
/// Many properties of Julia types can be checked, including whether instances of the type are
/// mutable, if the value is an array, and so on. The method [`Value::is`] can be used to perform
/// these checks. All these checks implement the [`Typecheck`] trait. If the type that implements
/// this trait also implements [`ValidLayout`], the typecheck indicates whether or not the value
/// can be cast to or unboxed as that type.
impl Value<'_, '_> {
    /// Performs the given typecheck:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    /// julia.scope(|_global, frame| {
    ///     let i = Value::new(frame, 2u64)?;
    ///     assert!(i.is::<u64>());
    ///     Ok(())
    /// }).unwrap();
    /// # });
    /// # }
    /// ```
    ///
    /// A full list of supported checks can be found [here].
    ///
    /// [`JuliaStruct`]: crate::wrappers::ptr::traits::julia_struct::JuliaStruct
    /// [here]: ../../../layout/typecheck/trait.Typecheck.html#implementors
    pub fn is<T: Typecheck>(self) -> bool {
        self.datatype().is::<T>()
    }

    /// Returns true if the value is an array with elements of type `T`.
    pub fn is_array_of<T: ValidLayout>(self) -> bool {
        match self.cast::<Array>() {
            Ok(arr) => arr.contains::<T>(),
            Err(_) => false,
        }
    }

    /// Returns true if `self` is a subtype of `sup`.
    pub fn subtype(self, sup: Value) -> bool {
        unsafe { jl_subtype(self.unwrap(Private), sup.unwrap(Private)) != 0 }
    }

    /// Returns true if `self` is the type of a `DataType`, `UnionAll`, `Union`, or `Union{}` (the
    /// bottom type).
    pub fn is_kind(self) -> bool {
        unsafe {
            let global = Global::new();
            let ptr = self.unwrap(Private);
            ptr == DataType::datatype_type(global).unwrap(Private).cast()
                || ptr == DataType::unionall_type(global).unwrap(Private).cast()
                || ptr == DataType::uniontype_type(global).unwrap(Private).cast()
                || ptr == DataType::typeofbottom_type(global).unwrap(Private).cast()
        }
    }

    /// Returns true if the value is a type, ie a `DataType`, `UnionAll`, `Union`, or `Union{}`
    /// (the bottom type).
    pub fn is_type(self) -> bool {
        Value::is_kind(self.datatype().as_value())
    }

    /// Returns true if `self` is of type `ty`.
    pub fn isa(self, ty: Value) -> bool {
        unsafe { jl_isa(self.unwrap(Private), ty.unwrap(Private)) != 0 }
    }
}

/// # Lifetime management
///
/// Values have two lifetimes, `'scope` and `'data`. The first ensures that a value can only be
/// used while it's rooted, the second ensures that values that (might) borrow array data from
/// Rust are also restricted by the lifetime of that borrow. This second restriction can be
/// relaxed with [`Value::assume_owned`] if it doesn't borrow any data from Rust.
impl<'scope, 'data> Value<'scope, 'data> {
    /// If you call a Julia function with one or more borrowed arrays as arguments, its result can
    /// only be used when all the borrows are active. If this result doesn't contain any borrowed
    /// data this function can be used to relax its second lifetime to `'static`.
    ///
    /// Safety: The value must not contain any data borrowed from Rust.
    pub unsafe fn assume_owned(self) -> Value<'scope, 'static> {
        Value::wrap_non_null(self.unwrap_non_null(Private), Private)
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> Value<'target, 'data> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<Value>(ptr);
            Value::wrap_non_null(ptr, Private)
        }
    }
}

/// # Conversions
///
/// There are two ways to convert a [`Value`] to some other type. The first is casting, which is
/// used to convert a [`Value`] to the appropriate pointer wrapper type. For example, if the
/// [`Value`] is a Julia array it can be cast to [`Array`]. Because this only involves a pointer
/// cast it's always possible to convert a wrapper to a [`Value`] by calling
/// [`Wrapper::as_value`]. The second way is unboxing, which is used to copy the data the
/// [`Value`] points to to Rust. If a [`Value`] is a `UInt8`, it can be unboxed as a `u8`. By
/// default, jlrs can unbox the default primitive types and Julia strings, but the [`Unbox`] trait
/// can be implemented for other types. It's recommended that you use JlrsReflect.jl to do so.
/// Unlike casting, unboxing dereferences the pointer. As a result it loses its header, so an
/// unboxed value can't be used as a [`Value`] again without reallocating it.
impl<'scope, 'data> Value<'scope, 'data> {
    /// Cast the value to a pointer wrapper type `T`. Returns an error if the conversion is
    /// invalid.
    pub fn cast<T: Wrapper<'scope, 'data> + Typecheck>(self) -> JlrsResult<T> {
        if self.is::<T>() {
            unsafe { Ok(T::cast(self, Private)) }
        } else {
            Err(AccessError::InvalidLayout {
                value_type_str: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }
    }

    /// Cast the value to a pointer wrapper type `T` without checking if this conversion is valid.
    ///
    /// Safety: You must guarantee `self.is::<T>()` would have returned `true`.
    pub unsafe fn cast_unchecked<T: Wrapper<'scope, 'data>>(self) -> T {
        T::cast(self, Private)
    }

    /// Unbox the contents of the value as the output type associated with `T`. Returns an error
    /// if the layout of `T::Output` is incompatible with the layout of the type in Julia.
    pub fn unbox<T: Unbox + Typecheck>(self) -> JlrsResult<T::Output> {
        if !self.is::<T>() {
            Err(AccessError::InvalidLayout {
                value_type_str: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe { Ok(T::unbox(self)) }
    }

    /// Unbox the contents of the value as the output type associated with `T` without checking
    /// if the layout of `T::Output` is compatible with the layout of the type in Julia.
    ///
    /// Safety: You must guarantee `self.is::<T>()` would have returned `true`.
    pub unsafe fn unbox_unchecked<T: Unbox>(self) -> T::Output {
        T::unbox(self)
    }

    /// Returns a pointer to the data, this is useful when the output type of `Unbox` is different
    /// than the implementation type and you have to write a custom unboxing function. It's your
    /// responsibility this pointer is used correctly.
    pub fn data_ptr(self) -> NonNull<c_void> {
        self.unwrap_non_null(Private).cast()
    }
}

/// # Fields
///
/// Most Julia values have fields. For example, if the value is an instance of this struct:
///
/// ```julia
/// struct Example
///    fielda
///    fieldb::UInt32
/// end
/// ```
///
/// it will have two fields, `fielda` and `fieldb`. The first field is a pointer field, the second
/// is stored inline as a `u32`. It's possible to safely access the raw contents of these fields
/// with the method [`Value::field_accessor`]. The first field can be accessed as a [`ValueRef`],
/// the second as a `u32`.
impl<'scope, 'data> Value<'scope, 'data> {
    /// Returns the field names of this value as a slice of `Symbol`s.
    pub fn field_names(self) -> &'scope [Symbol<'scope>] {
        unsafe {
            // Symbol and SymbolRef have the same layout, and this data is non-null
            std::mem::transmute(
                self.datatype()
                    .field_names()
                    .wrapper_unchecked()
                    .unrestricted_data()
                    .as_slice(),
            )
        }
    }

    /// Returns the number of fields the underlying Julia value has.
    pub fn n_fields(self) -> usize {
        self.datatype().n_fields() as _
    }

    /// Returns an accessor to access the contents of this value without allocating temporary Julia data.
    pub fn field_accessor<'current, 'borrow, F: Frame<'current>>(
        self,
        _frame: &'borrow F,
    ) -> FieldAccessor<'scope, 'data, 'borrow> {
        FieldAccessor {
            value: self.as_ref(),
            current_field_type: self.datatype().as_ref(),
            offset: 0,
            #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
            buffer: AtomicBuffer::new(),
            state: ViewState::Unlocked,
            _frame: PhantomData,
        }
    }

    /// Roots the field at index `idx` if it exists and returns it, or a
    /// `JlrsError::AccessError` if the index is out of bounds.
    pub fn get_nth_field<'target, S>(
        self,
        scope: S,
        idx: usize,
    ) -> JlrsResult<Value<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        unsafe {
            if idx >= self.n_fields() {
                Err(AccessError::OutOfBoundsField {
                    idx,
                    n_fields: self.n_fields(),
                    value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                })?
            }

            let fld_ptr = jl_get_nth_field(self.unwrap(Private), idx as _);
            if fld_ptr.is_null() {
                Err(AccessError::UndefRef)?;
            }

            scope.value(NonNull::new_unchecked(fld_ptr), Private)
        }
    }

    /// Returns the field at index `idx` if it's a pointer field.
    ///
    /// If the field doesn't exist or if the field can't be referenced because its data is stored
    /// inline, a `JlrsError::AccessError` is returned.
    pub fn get_nth_field_ref(self, idx: usize) -> JlrsResult<ValueRef<'scope, 'data>> {
        let ty = self.datatype();
        if idx >= ty.n_fields() as _ {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields: self.n_fields(),
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        unsafe {
            if !jl_field_isptr(ty.unwrap(Private), idx as _) {
                let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE);

                let field_name = if let Some(field_name) = self.field_names().get(idx) {
                    field_name
                        .as_str()
                        .unwrap_or("<Cannot display field name>")
                        .to_string()
                } else {
                    format!("{}", idx)
                };

                Err(AccessError::NotAPointerField {
                    value_type: value_type_str,
                    field_name,
                })?
            }

            Ok(ValueRef::wrap(jl_get_nth_field_noalloc(
                self.unwrap(Private),
                idx,
            )))
        }
    }

    /// Returns the field at index `idx` if it exists as a `ValueRef`. If the field is an inline
    /// field a new value is allocated which is left unrooted.
    ///
    /// If the field doesn't exist `JlrsError::OutOfBoundsField` is returned.
    pub fn get_nth_field_unrooted(self, idx: usize) -> JlrsResult<ValueRef<'scope, 'data>> {
        if idx >= self.n_fields() {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields: self.n_fields(),
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        unsafe { Ok(ValueRef::wrap(jl_get_nth_field(self.unwrap(Private), idx))) }
    }

    /// Roots the field with the name `field_name` if it exists and returns it, or a
    /// `JlrsError::AccessError` if there's no field with that name.
    pub fn get_field<'target, N, S>(
        self,
        scope: S,
        field_name: N,
    ) -> JlrsResult<Value<'target, 'data>>
    where
        N: ToSymbol,
        S: PartialScope<'target>,
    {
        unsafe {
            let symbol = field_name.to_symbol_priv(Private);
            let idx = jl_field_index(self.datatype().unwrap(Private), symbol.unwrap(Private), 0);

            if idx < 0 {
                Err(AccessError::NoSuchField {
                    type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            let fld_ptr = jl_get_nth_field(self.unwrap(Private), idx as _);
            if fld_ptr.is_null() {
                Err(AccessError::UndefRef)?;
            }

            scope.value(NonNull::new_unchecked(fld_ptr), Private)
        }
    }

    /// Returns the field with the name `field_name` if it's a pointer field.
    ///
    /// If the field doesn't exist or if the field can't be referenced because its data is stored
    /// inline, a `JlrsError::AccessError` is returned.
    pub fn get_field_ref<N>(self, field_name: N) -> JlrsResult<ValueRef<'scope, 'data>>
    where
        N: ToSymbol,
    {
        unsafe {
            let symbol = field_name.to_symbol_priv(Private);
            let ty = self.datatype();
            let idx = jl_field_index(ty.unwrap(Private), symbol.unwrap(Private), 0);

            if idx < 0 {
                Err(AccessError::NoSuchField {
                    type_name: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            if !jl_field_isptr(ty.unwrap(Private), idx as _) {
                let idx = idx as usize;
                let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE);

                let field_name = self.field_names()[idx]
                    .as_str()
                    .unwrap_or("<Cannot display field name>")
                    .to_string();

                Err(AccessError::NotAPointerField {
                    value_type: value_type_str,
                    field_name,
                })?
            }

            Ok(ValueRef::wrap(jl_get_nth_field_noalloc(
                self.unwrap(Private),
                idx as _,
            )))
        }
    }

    /// Returns the field with the name `field_name` if it exists. If the field is an inline field
    /// a new value is allocated which is left unrooted.
    ///
    /// If the field doesn't exist a `JlrsError::AccessError` is returned.
    pub fn get_field_unrooted<N>(self, field_name: N) -> JlrsResult<ValueRef<'scope, 'data>>
    where
        N: ToSymbol,
    {
        unsafe {
            let symbol = field_name.to_symbol_priv(Private);
            let idx = jl_field_index(self.datatype().unwrap(Private), symbol.unwrap(Private), 0);

            if idx < 0 {
                Err(AccessError::NoSuchField {
                    type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            Ok(ValueRef::wrap(jl_get_nth_field(
                self.unwrap(Private),
                idx as _,
            )))
        }
    }

    /// Set the value of the field at `idx`. If Julia throws an exception it's caught, rooted in
    /// the frame, and returned. If the index is out of bounds or the value is not a subtype of
    /// the field an error is returned,
    ///
    /// Safety: Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is not prevented.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn set_nth_field<'frame, F>(
        self,
        frame: &mut F,
        idx: usize,
        value: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'frame, 'data, ()>>
    where
        F: Frame<'frame>,
    {
        let n_fields = self.n_fields();
        if n_fields <= idx {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields,
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        let field_type = self
            .datatype()
            .field_types()
            .wrapper_unchecked()
            .unrestricted_data()
            .as_slice()[idx as usize]
            .value_unchecked();
        let dt = value.datatype();

        if !Value::subtype(dt.as_value(), field_type) {
            Err(TypeError::NotASubtype {
                field_type: field_type.display_string_or(CANNOT_DISPLAY_TYPE),
                value_type: value.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        let res = jlrs_set_nth_field(self.unwrap(Private), idx, value.unwrap(Private));
        if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
            let ptr = res.data;
            let err = crate::memory::scope::private::PartialScopePriv::value(
                frame,
                NonNull::new_unchecked(ptr),
                Private,
            )?;
            Ok(Err(err))
        } else {
            Ok(Ok(()))
        }
    }

    /// Set the value of the field at `idx`. If Julia throws an exception it's caught and
    /// returned but not rooted. If the index is out of bounds or the value is not a subtype of
    /// the field an error is returned,
    ///
    /// Safety: Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is not prevented.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn set_nth_field_unrooted(
        self,
        idx: usize,
        value: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResultRef<'scope, 'data, ()>> {
        let n_fields = self.n_fields();
        if n_fields <= idx {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields,
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        let field_type = self
            .datatype()
            .field_types()
            .wrapper_unchecked()
            .unrestricted_data()
            .as_slice()[idx as usize]
            .value_unchecked();
        let dt = value.datatype();

        if !Value::subtype(dt.as_value(), field_type) {
            Err(TypeError::NotASubtype {
                field_type: field_type.display_string_or(CANNOT_DISPLAY_TYPE),
                value_type: value.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        let res = jlrs_set_nth_field(self.unwrap(Private), idx, value.unwrap(Private));
        if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
            Ok(Err(ValueRef::wrap(res.data)))
        } else {
            Ok(Ok(()))
        }
    }

    /// Set the value of the field at `idx`. If Julia throws an exception the process aborts.
    ///
    /// Safety: this method doesn't check if the type of the value is a subtype of the field's
    /// type. Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is also not prevented.
    pub unsafe fn set_nth_field_unchecked(self, idx: usize, value: Value<'_, 'data>) {
        jl_set_nth_field(self.unwrap(Private), idx, value.unwrap(Private))
    }

    /// Set the value of the field with the name `field_name`. If Julia throws an exception it's
    /// caught, rooted in the frame, and returned. If there's no field with the given name or the
    /// value is not a subtype of the field an error is returned.
    ///
    /// Safety: Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is not prevented.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn set_field<'frame, F, N>(
        self,
        frame: &mut F,
        field_name: N,
        value: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'frame, 'data, ()>>
    where
        F: Frame<'frame>,
        N: ToSymbol,
    {
        let symbol = field_name.to_symbol_priv(Private);
        let idx = jl_field_index(self.datatype().unwrap(Private), symbol.unwrap(Private), 0);

        if idx < 0 {
            Err(AccessError::NoSuchField {
                type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
            })?
        }

        let field_type = self
            .datatype()
            .field_types()
            .wrapper_unchecked()
            .unrestricted_data()
            .as_slice()[idx as usize]
            .value_unchecked();
        let dt = value.datatype();

        if !Value::subtype(dt.as_value(), field_type) {
            Err(TypeError::NotASubtype {
                field_type: field_type.display_string_or(CANNOT_DISPLAY_TYPE),
                value_type: value.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        let res = jlrs_set_nth_field(self.unwrap(Private), idx as usize, value.unwrap(Private));
        if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
            let ptr = res.data;
            let err = crate::memory::scope::private::PartialScopePriv::value(
                frame,
                NonNull::new_unchecked(ptr),
                Private,
            )?;
            Ok(Err(err))
        } else {
            Ok(Ok(()))
        }
    }

    /// Set the value of the field with the name `field_name`. If Julia throws an exception it's
    /// caught, and returned but not rooted. If there's no field with the given name or the value
    /// is not a subtype of the field an error is returned.
    ///
    /// Safety: Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is not prevented.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn set_field_unrooted<N>(
        self,
        field_name: N,
        value: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResultRef<'scope, 'data, ()>>
    where
        N: ToSymbol,
    {
        let symbol = field_name.to_symbol_priv(Private);
        let idx = jl_field_index(self.datatype().unwrap(Private), symbol.unwrap(Private), 0);

        if idx < 0 {
            Err(AccessError::NoSuchField {
                type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
            })?
        }

        let field_type = self
            .datatype()
            .field_types()
            .wrapper_unchecked()
            .unrestricted_data()
            .as_slice()[idx as usize]
            .value_unchecked();
        let dt = value.datatype();

        if !Value::subtype(dt.as_value(), field_type) {
            Err(TypeError::NotASubtype {
                field_type: field_type.display_string_or(CANNOT_DISPLAY_TYPE),
                value_type: value.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        let res = jlrs_set_nth_field(self.unwrap(Private), idx as usize, value.unwrap(Private));
        if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
            Ok(Err(ValueRef::wrap(res.data)))
        } else {
            Ok(Ok(()))
        }
    }

    /// Set the value of the field with the name `field_name`. If Julia throws an exception the
    /// process aborts. If there's no field with the given name an error is returned.
    ///
    /// Safety: this method doesn't check if the type of the value is a subtype of the field's
    /// type. Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is also not prevented.
    pub unsafe fn set_field_unchecked<N>(
        self,
        field_name: N,
        value: Value<'_, 'data>,
    ) -> JlrsResult<()>
    where
        N: ToSymbol,
    {
        let symbol = field_name.to_symbol_priv(Private);
        let idx = jl_field_index(self.datatype().unwrap(Private), symbol.unwrap(Private), 0);

        if idx < 0 {
            Err(AccessError::NoSuchField {
                type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
            })?
        }
        Ok(jl_set_nth_field(
            self.unwrap(Private),
            idx as usize,
            value.unwrap(Private),
        ))
    }
}

/// # Evaluate Julia code
///
/// The easiest way to call Julia from Rust is by evaluating some Julia code directly. This can be
/// used to call simple functions without any arguments provided from Rust and to execute
/// using-statements.
impl Value<'_, '_> {
    /// Execute a Julia command `cmd`, for example `Value::eval_string(&mut *frame, "sqrt(2)")` or
    /// `Value::eval_string(&mut *frame, "using LinearAlgebra")`.
    ///
    /// Safety: The command can't be checked for correctness, nothing prevents you from causing a
    /// segmentation fault with a command like `unsafe_load(Ptr{Float64}(C_NULL))`.
    pub unsafe fn eval_string<'target, C, S>(
        scope: S,
        cmd: C,
    ) -> JlrsResult<JuliaResult<'target, 'static>>
    where
        C: AsRef<str>,
        S: PartialScope<'target>,
    {
        let cmd = cmd.as_ref();
        let cmd_cstring = CString::new(cmd).map_err(JlrsError::other)?;
        let cmd_ptr = cmd_cstring.as_ptr();
        let res = jl_eval_string(cmd_ptr);
        let exc = jl_exception_occurred();
        let output = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };
        scope.call_result(output, Private)
    }

    /// Execute a Julia command `cmd`. This is equivalent to `Value::eval_string`, but uses a
    /// null-terminated string.
    ///
    /// Safety: The command can't be checked for correctness, nothing prevents you from causing a
    /// segmentation fault with a command like `unsafe_load(Ptr{Float64}(C_NULL))`.
    pub unsafe fn eval_cstring<'target, C, S>(
        scope: S,
        cmd: C,
    ) -> JlrsResult<JuliaResult<'target, 'static>>
    where
        C: AsRef<CStr>,
        S: PartialScope<'target>,
    {
        let cmd = cmd.as_ref();
        let cmd_ptr = cmd.as_ptr();
        let res = jl_eval_string(cmd_ptr);
        let exc = jl_exception_occurred();
        let output = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };
        scope.call_result(output, Private)
    }

    /// Calls `include` in the `Main` module in Julia, which evaluates the file's contents in that
    /// module. This has the same effect as calling `include` in the Julia REPL.
    ///
    /// Safety: The content of the file can't be checked for correctness, nothing prevents you
    /// from causing a segmentation fault with code like `unsafe_load(Ptr{Float64}(C_NULL))`.
    pub unsafe fn include<'target, 'current, P, S, F>(
        scope: S,
        path: P,
    ) -> JlrsResult<JuliaResult<'target, 'static>>
    where
        P: AsRef<Path>,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        if path.as_ref().exists() {
            let global = scope.global();
            let (output, scope) = scope.split()?;
            return scope.scope(|frame| {
                let path_jl_str = JuliaString::new(&mut *frame, path.as_ref().to_string_lossy())?;
                let include_func = Module::main(global)
                    .function_ref("include")?
                    .wrapper_unchecked();

                let scope = output.into_scope(frame);
                include_func.call1(scope, path_jl_str.as_value())
            });
        }

        Err(IOError::NotFound {
            path: path.as_ref().to_string_lossy().into(),
        })?
    }
}

/// # Equality
impl Value<'_, '_> {
    /// Returns the object id of this value.
    pub fn object_id(self) -> usize {
        unsafe { jl_object_id(self.unwrap(Private)) }
    }

    /// Returns true if `self` and `other` are equal.
    pub fn egal(self, other: Value) -> bool {
        unsafe { jl_egal(self.unwrap(Private), other.unwrap(Private)) != 0 }
    }
}

/// # Finalization
impl Value<'_, '_> {
    /// Add a finalizer `f` to this value. The finalizer must be a Julia function, it will be
    /// called when this value is about to be freed by the garbage collector.
    ///
    /// Safety: the finalizer must be compatible with the data.
    pub unsafe fn add_finalizer(self, f: Value<'_, 'static>) {
        jl_gc_add_finalizer(self.unwrap(Private), f.unwrap(Private))
    }

    /// Add a finalizer `f` to this value. The finalizer must be an `extern "C"` function that
    /// takes one argument, the value as a void pointer.
    ///
    /// Safety: the finalizer must be compatible with the data.
    pub unsafe fn add_ptr_finalizer(self, f: unsafe extern "C" fn(*mut c_void) -> ()) {
        jl_gc_add_ptr_finalizer(get_tls(), self.unwrap(Private), f as *mut c_void)
    }
}

/// # Constant values.
impl<'scope> Value<'scope, 'static> {
    /// `Union{}`.
    pub fn bottom_type(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_bottom_type), Private) }
    }

    /// `StackOverflowError`.
    pub fn stackovf_exception(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_stackovf_exception), Private) }
    }

    /// `OutOfMemoryError`.
    pub fn memory_exception(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_memory_exception), Private) }
    }

    /// `ReadOnlyMemoryError`.
    pub fn readonlymemory_exception(_: Global<'scope>) -> Self {
        unsafe {
            Value::wrap_non_null(NonNull::new_unchecked(jl_readonlymemory_exception), Private)
        }
    }

    /// `DivideError`.
    pub fn diverror_exception(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_diverror_exception), Private) }
    }

    /// `UndefRefError`.
    pub fn undefref_exception(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_undefref_exception), Private) }
    }

    /// `InterruptException`.
    pub fn interrupt_exception(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_interrupt_exception), Private) }
    }

    /// An empty `Array{Any, 1}.
    pub fn an_empty_vec_any(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_an_empty_vec_any), Private) }
    }

    /// An empty immutable String, "".
    pub fn an_empty_string(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_an_empty_string), Private) }
    }

    /// `Array{UInt8, 1}`
    pub fn array_uint8_type(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_uint8_type), Private) }
    }

    /// `Array{Any, 1}`
    pub fn array_any_type(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_any_type), Private) }
    }

    /// `Array{Symbol, 1}`
    pub fn array_symbol_type(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_symbol_type), Private) }
    }

    /// `Array{Int32, 1}`
    pub fn array_int32_type(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_int32_type), Private) }
    }

    /// The empty tuple, `()`.
    pub fn emptytuple(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_emptytuple), Private) }
    }

    /// The instance of `true`.
    pub fn true_v(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_true), Private) }
    }

    /// The instance of `false`.
    pub fn false_v(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_false), Private) }
    }

    /// The instance of `Nothing`, `nothing`.
    pub fn nothing(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_nothing), Private) }
    }

    /// The handle to `stdout` as a Julia value.
    pub fn stdout(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_stdout_obj()), Private) }
    }

    /// The handle to `stderr` as a Julia value.
    pub fn stderr(_: Global<'scope>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_stderr_obj()), Private) }
    }
}

impl<'data> Call<'data> for Value<'_, 'data> {
    #[inline(always)]
    unsafe fn call0<'target, S>(self, scope: S) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        let res = match self.call0_unrooted(Global::new()) {
            Ok(v) => Ok(NonNull::new_unchecked(v.ptr())),
            Err(e) => Err(NonNull::new_unchecked(e.ptr())),
        };
        scope.call_result(res, Private)
    }

    #[inline(always)]
    unsafe fn call1<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        let res = match self.call1_unrooted(Global::new(), arg0) {
            Ok(v) => Ok(NonNull::new_unchecked(v.ptr())),
            Err(e) => Err(NonNull::new_unchecked(e.ptr())),
        };
        scope.call_result(res, Private)
    }

    #[inline(always)]
    unsafe fn call2<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        let res = match self.call2_unrooted(Global::new(), arg0, arg1) {
            Ok(v) => Ok(NonNull::new_unchecked(v.ptr())),
            Err(e) => Err(NonNull::new_unchecked(e.ptr())),
        };
        scope.call_result(res, Private)
    }

    #[inline(always)]
    unsafe fn call3<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        let res = match self.call3_unrooted(Global::new(), arg0, arg1, arg2) {
            Ok(v) => Ok(NonNull::new_unchecked(v.ptr())),
            Err(e) => Err(NonNull::new_unchecked(e.ptr())),
        };
        scope.call_result(res, Private)
    }

    #[inline(always)]
    unsafe fn call<'target, 'value, V, S>(
        self,
        scope: S,
        args: V,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        V: AsRef<[Value<'value, 'data>]>,
        S: PartialScope<'target>,
    {
        let res = match self.call_unrooted(Global::new(), args) {
            Ok(v) => Ok(NonNull::new_unchecked(v.ptr())),
            Err(e) => Err(NonNull::new_unchecked(e.ptr())),
        };
        scope.call_result(res, Private)
    }

    #[inline(always)]
    unsafe fn call0_unrooted<'target>(self, _: Global<'target>) -> JuliaResultRef<'target, 'data> {
        let res = jl_call0(self.unwrap(Private));
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Err(ValueRef::wrap(exc))
        }
    }

    #[inline(always)]
    unsafe fn call1_unrooted<'target>(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        let res = jl_call1(self.unwrap(Private), arg0.unwrap(Private));
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Err(ValueRef::wrap(exc))
        }
    }

    #[inline(always)]
    unsafe fn call2_unrooted<'target>(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        let res = jl_call2(
            self.unwrap(Private),
            arg0.unwrap(Private),
            arg1.unwrap(Private),
        );
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Err(ValueRef::wrap(exc))
        }
    }

    #[inline(always)]
    unsafe fn call3_unrooted<'target>(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        let res = jl_call3(
            self.unwrap(Private),
            arg0.unwrap(Private),
            arg1.unwrap(Private),
            arg2.unwrap(Private),
        );
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Err(ValueRef::wrap(exc))
        }
    }

    #[inline(always)]
    unsafe fn call_unrooted<'target, 'value, V>(
        self,
        _: Global<'target>,
        args: V,
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsRef<[Value<'value, 'data>]>,
    {
        let args = args.as_ref();
        let n = args.len();
        let res = jl_call(self.unwrap(Private).cast(), args.as_ptr() as *mut _, n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Err(ValueRef::wrap(exc))
        }
    }
}

impl<'value, 'data> CallExt<'value, 'data> for Value<'value, 'data> {
    fn with_keywords(self, kws: Value<'value, 'data>) -> JlrsResult<WithKeywords<'value, 'data>> {
        if !kws.is::<NamedTuple>() {
            let type_str = kws.datatype().display_string_or(CANNOT_DISPLAY_TYPE);
            Err(TypeError::NotANamedTuple { type_str })?
        }
        Ok(WithKeywords::new(self, kws))
    }
}

impl_debug!(Value<'_, '_>);

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Value<'scope, 'data> {
    type Wraps = jl_value_t;
    const NAME: &'static str = "Value";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// While jlrs generally enforces that Julia data can only exist and be used while a frame is
/// active, it's possible to leak global values: [`Symbol`]s, [`Module`]s, and globals defined in
/// those modules.
#[derive(Copy, Clone)]
pub struct LeakedValue(Value<'static, 'static>);

impl LeakedValue {
    #[inline(always)]
    pub(crate) unsafe fn wrap(ptr: *mut jl_value_t) -> Self {
        LeakedValue(Value::wrap(ptr, Private))
    }

    #[inline(always)]
    pub(crate) unsafe fn wrap_non_null(ptr: NonNull<jl_value_t>) -> Self {
        LeakedValue(Value::wrap_non_null(ptr, Private))
    }

    /// Convert this [`LeakedValue`] back to a [`Value`]. This requires a [`Global`], so this
    /// method can only be called inside a closure taken by one of the `scope`-methods.
    ///
    /// Safety: you must guarantee this value has not been freed by the garbage collector. While
    /// `Symbol`s are never garbage collected, modules and their contents can be redefined.
    #[inline(always)]
    pub unsafe fn as_value<'scope>(self, _: Global<'scope>) -> Value<'scope, 'static> {
        self.0
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
union AtomicBuffer {
    bytes: [MaybeUninit<u8>; 8],
    ptr: *mut jl_value_t,
}

#[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
impl AtomicBuffer {
    fn new() -> Self {
        AtomicBuffer { ptr: null_mut() }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum ViewState {
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    Locked,
    Unlocked,
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    AtomicBuffer,
    Array,
}

/// Access the raw contents of a Julia value.
///
/// A `FieldAccessor` for a value can be created with [`Value::field_accessor`]. By chaining calls
/// to the `field` and `atomic_field` methods you can access deeply nested fields without
/// allocating temporary Julia data. These two methods support three kinds of field identifiers:
/// field names, numerical field indices, and n-dimensional array indices. The first two can be
/// used with types that have named fields, the second must be used with tuples, and the last one
/// with arrays.
pub struct FieldAccessor<'scope, 'data, 'borrow> {
    value: ValueRef<'scope, 'data>,
    current_field_type: DataTypeRef<'scope>,
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    buffer: AtomicBuffer,
    offset: u32,
    state: ViewState,
    _frame: PhantomData<&'borrow ()>,
}

impl<'scope, 'data> FieldAccessor<'scope, 'data, '_> {
    /// Access the field the accessor is currenty pointing to as a value of type `T`.
    ///
    /// This method accesses the field using its concrete type. If the concrete type of the field
    /// has a matching pointer wrapper type it can be accessed as a `ValueRef` or a `Ref` of to
    /// that pointer wrapper type. For example, a field that contains a `Module` can be accessed
    /// as a `ModuleRef`. In all other cases an inline wrapper type must be used. For example, an
    /// untyped field that currently holds a `Float64` must be accessed as `f64`.
    pub fn access<T: ValidLayout>(self) -> JlrsResult<T> {
        if self.current_field_type.is_undefined() {
            Err(AccessError::UndefRef)?;
        }

        unsafe {
            let ty = self.current_field_type.value_unchecked();
            if !T::valid_layout(ty) {
                let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
                Err(AccessError::InvalidLayout { value_type_str })?;
            }

            #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
            if self.state == ViewState::AtomicBuffer {
                debug_assert!(!T::IS_REF);
                debug_assert!(std::mem::size_of::<T>() <= 8);
                return Ok(std::ptr::read(
                    self.buffer.bytes[self.offset as usize..].as_ptr() as *const T,
                ));
            }

            if T::IS_REF {
                Ok(std::mem::transmute_copy(&self.value))
            } else if self.state == ViewState::Array {
                Ok(self
                    .value
                    .value_unchecked()
                    .cast_unchecked::<Array>()
                    .data_ptr()
                    .cast::<u8>()
                    .add(self.offset as usize)
                    .cast::<T>()
                    .read())
            } else {
                Ok(self
                    .value
                    .ptr()
                    .cast::<u8>()
                    .add(self.offset as usize)
                    .cast::<T>()
                    .read())
            }
        }
    }

    /// Returns `true` if `self.access::<T>()` will succeed, `false` if it will fail.
    pub fn can_access_as<T: ValidLayout>(&self) -> bool {
        if self.current_field_type.is_undefined() {
            return false;
        }

        unsafe {
            let ty = self.current_field_type.value_unchecked();
            if !T::valid_layout(ty) {
                return false;
            }
        }

        true
    }

    /// Update the accessor to point to `field`.
    ///
    /// Three kinds of field indices exist: field names, numerical field indices, and
    /// n-dimensional array indices. The first two can be used with types that have named fields,
    /// the second must be used with tuples, and the last one with arrays.
    ///
    /// If `field` is an invalid identifier an error is returned. Calls to `field` can be chained
    /// to access nested fields.
    ///
    /// If the field is an atomic field the same ordering is used as Julia uses by default:
    /// `Relaxed` for pointer fields, `SeqCst` for small inline fields, and a lock for large
    /// inline fields.
    pub fn field<F: FieldIndex>(mut self, field: F) -> JlrsResult<Self> {
        if self.value.is_undefined() {
            Err(AccessError::UndefRef)?
        }

        if self.current_field_type.is_undefined() {
            Err(AccessError::UndefRef)?
        }

        unsafe {
            let current_field_type = self.current_field_type.wrapper_unchecked();
            if self.state == ViewState::Array && current_field_type.is::<Array>() {
                let arr = self.value.value_unchecked().cast_unchecked::<Array>();
                // accessing an array, find the offset of the requested element
                let index = field.array_index(arr, Private)?;
                self.get_array_field(arr, index);
                return Ok(self);
            }

            let index = field.field_index(current_field_type, Private)?;

            let next_field_type = current_field_type.field_type_unchecked(index);
            if next_field_type.is_undefined() {
                Err(AccessError::UndefRef)?
            }

            let next_field_type = next_field_type.wrapper_unchecked();
            let is_pointer_field = current_field_type.is_pointer_field_unchecked(index);
            let field_offset = current_field_type.field_offset_unchecked(index);
            self.offset += field_offset;

            match self.state {
                ViewState::Array => {
                    self.get_inline_array_field(is_pointer_field, next_field_type)?
                }
                ViewState::Unlocked => self.get_unlocked_inline_field(
                    is_pointer_field,
                    current_field_type,
                    next_field_type,
                    index,
                    Ordering::Relaxed,
                    Ordering::SeqCst,
                ),
                #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
                ViewState::Locked => {
                    self.get_locked_inline_field(is_pointer_field, next_field_type)
                }
                #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
                ViewState::AtomicBuffer => {
                    self.get_atomic_buffer_field(is_pointer_field, next_field_type)
                }
            }
        }

        Ok(self)
    }

    /// Update the accessor to point to `field`.
    ///
    /// If the field is a small atomic field `ordering` is used to read it. The ordering is
    /// ignored for non-atomic fields and fields that require a lock to access. See
    /// [`FieldAccessor::field`] for more information.
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    pub fn atomic_field<F: FieldIndex>(mut self, field: F, ordering: Ordering) -> JlrsResult<Self> {
        if self.value.is_undefined() {
            Err(AccessError::UndefRef)?
        }

        if self.current_field_type.is_undefined() {
            Err(AccessError::UndefRef)?
        }

        unsafe {
            let current_field_type = self.current_field_type.wrapper_unchecked();
            if self.state == ViewState::Array && current_field_type.is::<Array>() {
                let arr = self.value.value_unchecked().cast_unchecked::<Array>();
                // accessing an array, find the offset of the requested element
                let index = field.array_index(arr, Private)?;
                self.get_array_field(arr, index);
                return Ok(self);
            }

            let index = field.field_index(current_field_type, Private)?;

            let next_field_type = current_field_type.field_type_unchecked(index);
            if next_field_type.is_undefined() {
                Err(AccessError::UndefRef)?
            }

            let next_field_type = next_field_type.wrapper_unchecked();
            let is_pointer_field = current_field_type.is_pointer_field_unchecked(index);
            let field_offset = current_field_type.field_offset_unchecked(index);
            self.offset += field_offset;

            match self.state {
                ViewState::Array => {
                    self.get_inline_array_field(is_pointer_field, next_field_type)?
                }
                ViewState::Unlocked => self.get_unlocked_inline_field(
                    is_pointer_field,
                    current_field_type,
                    next_field_type,
                    index,
                    ordering,
                    ordering,
                ),
                ViewState::Locked => {
                    self.get_locked_inline_field(is_pointer_field, next_field_type)
                }
                ViewState::AtomicBuffer => {
                    self.get_atomic_buffer_field(is_pointer_field, next_field_type)
                }
            }
        }

        Ok(self)
    }

    /// Try to clone this accessor and its state.
    ///
    /// If the current value this accessor is accessing is locked an error is returned.
    pub fn try_clone(&self) -> JlrsResult<Self> {
        #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
        if self.state == ViewState::Locked {
            Err(AccessError::Locked)?;
        }

        Ok(FieldAccessor {
            value: self.value,
            current_field_type: self.current_field_type,
            offset: self.offset,
            #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
            buffer: self.buffer.clone(),
            state: self.state,
            _frame: PhantomData,
        })
    }

    /// Returns `true` if the current value the accessor is accessing is locked.
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    pub fn is_locked(&self) -> bool {
        self.state == ViewState::Locked
    }

    /// Returns `true` if the current value the accessor is accessing is locked.
    #[cfg(all(feature = "lts", not(feature = "all-features-override")))]
    pub fn is_locked(&self) -> bool {
        false
    }

    /// Returns the type of the field the accessor is currently pointing at.
    pub fn current_field_type(&self) -> DataTypeRef<'scope> {
        self.current_field_type
    }

    /// Returns the value the accessor is currently inspecting.
    pub fn value(&self) -> ValueRef<'scope, 'data> {
        self.value
    }

    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    unsafe fn get_atomic_buffer_field(
        &mut self,
        is_pointer_field: bool,
        next_field_type: Value<'scope, 'data>,
    ) {
        if is_pointer_field {
            debug_assert_eq!(self.offset, 0);
            self.value = ValueRef::wrap(self.buffer.ptr);
            self.state = ViewState::Unlocked;
            if self.value.is_undefined() {
                if let Ok(ty) = next_field_type.cast::<DataType>() {
                    if ty.is_concrete_type() {
                        self.current_field_type = ty.as_ref();
                    } else {
                        self.current_field_type = DataTypeRef::undefined_ref();
                    }
                } else {
                    self.current_field_type = DataTypeRef::undefined_ref();
                }
            } else {
                self.current_field_type = self.value.wrapper_unchecked().datatype().as_ref();
            }
        } else {
            debug_assert!(next_field_type.is::<DataType>());
            self.current_field_type = next_field_type.cast_unchecked::<DataType>().as_ref();
        }
    }

    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    unsafe fn get_unlocked_inline_field(
        &mut self,
        is_pointer_field: bool,
        current_field_type: DataType<'scope>,
        next_field_type: Value<'scope, 'data>,
        index: usize,
        pointer_ordering: Ordering,
        inline_ordering: Ordering,
    ) {
        let is_atomic_field = current_field_type.is_atomic_field_unchecked(index);
        if is_pointer_field {
            if is_atomic_field {
                self.get_atomic_pointer_field(next_field_type, pointer_ordering);
            } else {
                self.get_pointer_field(false, next_field_type);
            }
        } else if let Ok(un) = next_field_type.cast::<Union>() {
            self.get_bits_union_field(un);
        } else {
            debug_assert!(next_field_type.is::<DataType>());
            self.current_field_type = next_field_type.cast_unchecked::<DataType>().as_ref();

            if is_atomic_field {
                self.lock_or_copy_atomic(inline_ordering);
            }
        }
    }

    #[cfg(all(feature = "lts", not(feature = "all-features-override")))]
    unsafe fn get_unlocked_inline_field(
        &mut self,
        is_pointer_field: bool,
        _current_field_type: DataType<'scope>,
        next_field_type: Value<'scope, 'data>,
        _index: usize,
        _pointer_ordering: Ordering,
        _inline_ordering: Ordering,
    ) {
        if is_pointer_field {
            self.get_pointer_field(false, next_field_type);
        } else if let Ok(un) = next_field_type.cast::<Union>() {
            self.get_bits_union_field(un);
        } else {
            debug_assert!(next_field_type.is::<DataType>());
            self.current_field_type = next_field_type.cast_unchecked::<DataType>().as_ref();
        }
    }

    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    unsafe fn get_locked_inline_field(
        &mut self,
        is_pointer_field: bool,
        next_field_type: Value<'scope, 'data>,
    ) {
        if is_pointer_field {
            self.get_pointer_field(true, next_field_type);
        } else if let Ok(un) = next_field_type.cast::<Union>() {
            self.get_bits_union_field(un);
        } else {
            debug_assert!(next_field_type.is::<DataType>());
            self.current_field_type = next_field_type.cast_unchecked::<DataType>().as_ref();
        }
    }

    unsafe fn get_inline_array_field(
        &mut self,
        is_pointer_field: bool,
        next_field_type: Value<'scope, 'data>,
    ) -> JlrsResult<()> {
        // Inline field of the current array
        if is_pointer_field {
            self.value = self
                .value
                .value_unchecked()
                .cast::<Array>()?
                .data_ptr()
                .cast::<MaybeUninit<u8>>()
                .add(self.offset as usize)
                .cast::<ValueRef>()
                .read();

            self.offset = 0;
            self.state = ViewState::Unlocked;

            if self.value.is_undefined() {
                if let Ok(ty) = next_field_type.cast::<DataType>() {
                    if ty.is_concrete_type() {
                        self.current_field_type = ty.as_ref();
                    } else {
                        self.current_field_type = DataTypeRef::undefined_ref();
                    }
                } else {
                    self.current_field_type = DataTypeRef::undefined_ref();
                }
            } else {
                self.current_field_type = self.value.value_unchecked().datatype().as_ref();
            }
        } else {
            self.current_field_type = next_field_type.cast::<DataType>()?.as_ref();
        }

        Ok(())
    }

    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    unsafe fn lock_or_copy_atomic(&mut self, ordering: Ordering) {
        let ptr = self
            .value
            .ptr()
            .cast::<MaybeUninit<u8>>()
            .add(self.offset as usize);

        match self.current_field_type.wrapper_unchecked().size() {
            0 => (),
            1 => {
                let atomic = &*ptr.cast::<AtomicU8>();
                let v = atomic.load(ordering);
                let dst_ptr = self.buffer.bytes.as_mut_ptr();
                std::ptr::copy_nonoverlapping(&v as *const _ as *const u8, dst_ptr as _, 1);
                self.state = ViewState::AtomicBuffer;
                self.offset = 0;
            }
            2 => {
                let atomic = &*ptr.cast::<AtomicU16>();
                let v = atomic.load(ordering);
                let dst_ptr = self.buffer.bytes.as_mut_ptr();
                std::ptr::copy_nonoverlapping(&v as *const _ as *const u8, dst_ptr as _, 2);
                self.state = ViewState::AtomicBuffer;
                self.offset = 0;
            }
            sz if sz <= 4 => {
                let atomic = &*ptr.cast::<AtomicU32>();
                let v = atomic.load(ordering);
                let dst_ptr = self.buffer.bytes.as_mut_ptr();
                std::ptr::copy_nonoverlapping(
                    &v as *const _ as *const u8,
                    dst_ptr as _,
                    sz as usize,
                );
                self.state = ViewState::AtomicBuffer;
                self.offset = 0;
            }
            sz if sz <= 8 => {
                let atomic = &*ptr.cast::<AtomicU64>();
                let v = atomic.load(ordering);
                let dst_ptr = self.buffer.bytes.as_mut_ptr();
                std::ptr::copy_nonoverlapping(
                    &v as *const _ as *const u8,
                    dst_ptr as _,
                    sz as usize,
                );
                self.state = ViewState::AtomicBuffer;
                self.offset = 0;
            }
            _ => {
                jlrs_lock(self.value.ptr());
                self.state = ViewState::Locked;
            }
        }
    }

    unsafe fn get_array_field(&mut self, arr: Array<'scope, 'data>, index: usize) {
        debug_assert!(self.state == ViewState::Array);
        let el_size = arr.element_size();
        self.offset = (index * el_size) as u32;

        if arr.is_value_array() {
            self.value = arr.data_ptr().cast::<ValueRef>().add(index).read();
            self.offset = 0;
            if self.value.is_undefined() {
                if let Ok(ty) = arr.element_type().cast::<DataType>() {
                    if ty.is_concrete_type() {
                        self.current_field_type = ty.as_ref();
                    } else {
                        self.current_field_type = DataTypeRef::undefined_ref();
                    }

                    if !ty.is::<Array>() {
                        self.state = ViewState::Unlocked;
                    }
                } else {
                    self.current_field_type = DataTypeRef::undefined_ref();
                    self.state = ViewState::Unlocked;
                }
            } else {
                let ty = self.value.value_unchecked().datatype();
                self.current_field_type = ty.as_ref();
                if !ty.is::<Array>() {
                    self.state = ViewState::Unlocked;
                }
            }
        } else if arr.is_union_array() {
            let mut tag = *jl_array_typetagdata(arr.unwrap(Private)).add(index) as i32;
            let component = nth_union_component(arr.element_type(), &mut tag);
            debug_assert!(component.is_some());
            let ty = component.unwrap_unchecked();
            debug_assert!(ty.is::<DataType>());
            let ty = ty.cast_unchecked::<DataType>();
            debug_assert!(ty.is_concrete_type());
            self.current_field_type = ty.as_ref();
        } else {
            let ty = arr.element_type();
            debug_assert!(ty.is::<DataType>());
            self.current_field_type = ty.cast_unchecked::<DataType>().as_ref();
        }
    }

    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    unsafe fn get_pointer_field(&mut self, locked: bool, next_field_type: Value<'scope, 'data>) {
        let value = self
            .value
            .ptr()
            .cast::<u8>()
            .add(self.offset as usize)
            .cast::<ValueRef>()
            .read();

        if locked {
            jlrs_unlock(self.value.ptr());
            self.state = ViewState::Unlocked;
        }

        self.value = value;
        self.offset = 0;

        if self.value.is_undefined() {
            if let Ok(ty) = next_field_type.cast::<DataType>() {
                if ty.is_concrete_type() {
                    self.current_field_type = ty.as_ref();
                } else {
                    self.current_field_type = DataTypeRef::undefined_ref();
                }
            } else {
                self.current_field_type = DataTypeRef::undefined_ref();
            }
        } else {
            let value = self.value.value_unchecked();
            self.current_field_type = value.datatype().as_ref();
            if value.is::<Array>() {
                self.state = ViewState::Array;
            }
        }
    }

    #[cfg(all(feature = "lts", not(feature = "all-features-override")))]
    unsafe fn get_pointer_field(&mut self, _locked: bool, next_field_type: Value<'scope, 'data>) {
        let value = self
            .value
            .ptr()
            .cast::<u8>()
            .add(self.offset as usize)
            .cast::<ValueRef>()
            .read();

        self.value = value;
        self.offset = 0;

        if self.value.is_undefined() {
            if let Ok(ty) = next_field_type.cast::<DataType>() {
                if ty.is_concrete_type() {
                    self.current_field_type = ty.as_ref();
                } else {
                    self.current_field_type = DataTypeRef::undefined_ref();
                }
            } else {
                self.current_field_type = DataTypeRef::undefined_ref();
            }
        } else {
            let value = self.value.value_unchecked();
            self.current_field_type = value.datatype().as_ref();
            if value.is::<Array>() {
                self.state = ViewState::Array;
            }
        }
    }

    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    unsafe fn get_atomic_pointer_field(
        &mut self,
        next_field_type: Value<'scope, 'data>,
        ordering: Ordering,
    ) {
        let v = &*self
            .value
            .ptr()
            .cast::<u8>()
            .add(self.offset as usize)
            .cast::<AtomicPtr<jl_value_t>>();

        let ptr = v.load(ordering);
        self.value = ValueRef::wrap(ptr);
        self.offset = 0;

        if self.value.is_undefined() {
            if let Ok(ty) = next_field_type.cast::<DataType>() {
                if ty.is_concrete_type() {
                    self.current_field_type = ty.as_ref();
                } else {
                    self.current_field_type = DataTypeRef::undefined_ref();
                }
            } else {
                self.current_field_type = DataTypeRef::undefined_ref();
            }
        } else {
            let value = self.value.value_unchecked();
            self.current_field_type = value.datatype().as_ref();
            if value.is::<Array>() {
                self.state = ViewState::Array;
            }
        }
    }

    unsafe fn get_bits_union_field(&mut self, union: Union<'scope>) {
        let mut size = 0;
        let isbits = union.isbits_size_align(&mut size, &mut 0);
        debug_assert!(isbits);
        let flag_offset = self.offset as usize + size;
        let mut flag = self.value.ptr().cast::<u8>().add(flag_offset).read() as i32;

        let active_ty = nth_union_component(union.as_value(), &mut flag);
        debug_assert!(active_ty.is_some());
        let active_ty = active_ty.unwrap_unchecked();
        debug_assert!(active_ty.is::<DataType>());

        let ty = active_ty.cast_unchecked::<DataType>();
        debug_assert!(ty.is_concrete_type());
        self.current_field_type = ty.as_ref();
    }
}

impl Drop for FieldAccessor<'_, '_, '_> {
    fn drop(&mut self) {
        #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
        if self.state == ViewState::Locked {
            debug_assert!(!self.value.is_undefined());
            unsafe { jlrs_unlock(self.value.ptr()) }
        }
    }
}

impl_root!(Value, 2);
