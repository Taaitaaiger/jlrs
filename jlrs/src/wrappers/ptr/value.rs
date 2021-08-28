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
/// julia.scope_with_slots(3, |global, frame| {
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
    convert::{into_julia::IntoJulia, temporary_symbol::TemporarySymbol, unbox::Unbox},
    error::{JlrsError, JlrsResult, JuliaResult, JuliaResultRef, CANNOT_DISPLAY_TYPE},
    impl_debug,
    layout::{
        typecheck::{NamedTuple, Typecheck},
        valid_layout::ValidLayout,
    },
    memory::{frame::Frame, global::Global, scope::Scope},
    prelude::JuliaString,
    private::Private,
    wrappers::ptr::{
        array::Array,
        call::{Call, CallExt, WithKeywords},
        datatype::DataType,
        module::Module,
        private::Wrapper as WrapperPriv,
        symbol::Symbol,
        union::{nth_union_component, Union},
        union_all::UnionAll,
        ValueRef, Wrapper,
    },
};
use jl_sys::{
    jl_an_empty_string, jl_an_empty_vec_any, jl_apply_type, jl_array_any_type, jl_array_int32_type,
    jl_array_symbol_type, jl_array_uint8_type, jl_bottom_type, jl_call, jl_call0, jl_call1,
    jl_call2, jl_call3, jl_diverror_exception, jl_egal, jl_emptytuple, jl_eval_string,
    jl_exception_occurred, jl_false, jl_field_index, jl_field_isptr, jl_field_names,
    jl_field_offset, jl_fieldref, jl_fieldref_noalloc, jl_finalize, jl_gc_add_finalizer,
    jl_gc_add_ptr_finalizer, jl_get_ptls_states, jl_interrupt_exception, jl_is_kind, jl_isa,
    jl_memory_exception, jl_nfields, jl_nothing, jl_object_id, jl_readonlymemory_exception,
    jl_set_nth_field, jl_stackovf_exception, jl_stderr_obj, jl_stdout_obj, jl_subtype,
    jl_svec_data, jl_svec_len, jl_true, jl_typeof, jl_typeof_str, jl_undefref_exception,
    jl_value_t, jlrs_apply_type, jlrs_result_tag_t_JLRS_RESULT_ERR, jlrs_set_nth_field,
};
use std::{
    ffi::{c_void, CStr, CString},
    marker::PhantomData,
    path::Path,
    ptr::NonNull,
    slice, usize,
};

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
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Value<'scope, 'data>(
    NonNull<jl_value_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

/// # Create new `Value`s
///
/// Several methods are available to create new values. The simplest of these is [`Value::new`],
/// which can be used to convert relatively simple data from Rust to Julia. Data that can be
/// converted this way must implement [`IntoJulia`], which is the case for types like the
/// primitive number types. This trait is also automatically derived by JlrsReflect.jl for types
/// that are trivially guaranteed to be bits-types: the type must have no type parameters, no
/// unions, and all fields must be immutable bits-types themselves.
impl<'scope, 'data> Value<'scope, 'data> {
    /// Create a new Julia value, any type that implements [`IntoJulia`] can be converted using
    /// this function.
    pub fn new<'target, 'current, V, S, F>(scope: S, value: V) -> JlrsResult<S::Value>
    where
        V: IntoJulia,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            let global = scope.global();
            let v = value.into_julia(global).ptr();
            debug_assert!(!v.is_null());
            scope.value(NonNull::new_unchecked(v), Private)
        }
    }

    /// Create a new Julia value, any type that implements [`IntoJulia`] can be converted using
    /// this function. Unlike [`Value::new`] this method doesn't root the allocated value.
    pub fn new_unrooted<'global, V>(global: Global<'global>, value: V) -> ValueRef<'global, 'static>
    where
        V: IntoJulia,
    {
        unsafe { value.into_julia(global) }
    }

    /// Create a new named tuple, you should use the `named_tuple` macro rather than this method.
    pub fn new_named_tuple<'target, 'current, S, F, N, T, V>(
        scope: S,
        mut field_names: N,
        mut values: V,
    ) -> JlrsResult<S::Value>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
        N: AsMut<[T]>,
        T: TemporarySymbol,
        V: AsMut<[Value<'scope, 'data>]>,
    {
        scope.value_scope_with_slots(4, |output, frame| unsafe {
            let global = frame.global();
            let field_names = field_names.as_mut();
            let values_m = values.as_mut();

            let n_names = field_names.len();
            let n_values = values_m.len();

            if n_names != n_values {
                Err(JlrsError::NamedTupleSizeMismatch { n_names, n_values })?;
            }

            let symbol_ty = DataType::symbol_type(global).as_value();
            let mut symbol_type_vec = vec![symbol_ty; n_names];

            let mut field_names_vec = field_names
                .iter()
                .map(|name| name.temporary_symbol(Private).as_value())
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

            let output = output.into_scope(frame);
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
    pub fn apply_type<'target, 'current, S, F, V>(
        self,
        scope: S,
        mut types: V,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
        V: AsMut<[Value<'scope, 'data>]>,
    {
        unsafe {
            let types = types.as_mut();
            let applied =
                jlrs_apply_type(self.unwrap(Private), types.as_mut_ptr().cast(), types.len());

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
    /// If the types can't be applied to `self` the program will abort.
    ///
    /// [`Union::new`]: crate::wrappers::ptr::union::Union::new
    pub fn apply_type_unchecked<'target, 'current, S, F, V>(
        self,
        scope: S,
        mut types: V,
    ) -> JlrsResult<S::Value>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
        V: AsMut<[Value<'scope, 'data>]>,
    {
        unsafe {
            let types = types.as_mut();
            let applied =
                jl_apply_type(self.unwrap(Private), types.as_mut_ptr().cast(), types.len());
            scope.value(NonNull::new_unchecked(applied), Private)
        }
    }
}

/// # Type information
///
/// Every value is guaranteed to have a [`DataType`]. This contains all of the value's type
/// information.
impl<'scope, 'data> Value<'scope, 'data> {
    /// Returns the `DataType` of this value.
    pub fn datatype(self) -> DataType<'scope> {
        unsafe { DataType::wrap(jl_typeof(self.unwrap(Private)).cast(), Private) }
    }

    /// Returns the name of this value's [`DataType`] as a string slice.
    pub fn datatype_name(self) -> JlrsResult<&'scope str> {
        unsafe {
            let type_name = jl_typeof_str(self.unwrap(Private));
            let type_name_ref = CStr::from_ptr(type_name);
            Ok(type_name_ref.to_str().map_err(|_| JlrsError::NotUTF8)?)
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
        unsafe { jl_is_kind(self.unwrap(Private)) }
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
            Err(JlrsError::WrongType {
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }
    }

    /// Cast the value to a pointer wrapper type `T` without checking if this conversion is valid.
    ///
    /// Safety:
    ///
    /// You must guarantee `self.is::<T>()` would have returned `true`.
    pub unsafe fn cast_unchecked<T: Wrapper<'scope, 'data>>(self) -> T {
        T::cast(self, Private)
    }

    /// Unbox the contents of the value as the output type associated with `T`. Returns an error
    /// if the layout of `T::Output` is incompatible with the layout of the type in Julia.
    pub fn unbox<T: Unbox + Typecheck>(self) -> JlrsResult<T::Output> {
        if !self.is::<T>() {
            Err(JlrsError::WrongType {
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe { Ok(T::unbox(self)) }
    }

    /// Unbox the contents of the value as the output type associated with `T` without checking
    /// if the layout of `T::Output` is compatible with the layout of the type in Julia.
    ///
    /// Safety:
    ///
    /// You must guarantee `self.is::<T>()` would have returned `true`.
    pub fn unbox_unchecked<T: Unbox>(self) -> T::Output {
        unsafe { T::unbox(self) }
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
/// with the methods [`Value::get_raw_field`] and [`Value::get_nth_raw_field`]. The first field can be
/// accessed as a [`ValueRef`], the second as a `u32`.
impl<'scope, 'data> Value<'scope, 'data> {
    /// Returns the field names of this value as a slice of `Symbol`s.
    pub fn field_names(self) -> &'scope [Symbol<'scope>] {
        unsafe {
            let tp = jl_typeof(self.unwrap(Private));
            let field_names = jl_field_names(tp.cast());
            let len = jl_svec_len(field_names);
            let items = jl_svec_data(field_names);
            slice::from_raw_parts(items.cast(), len)
        }
    }

    /// Returns the number of fields the underlying Julia value has.
    pub fn n_fields(self) -> usize {
        unsafe { jl_nfields(self.unwrap(Private)) as _ }
    }

    /// Access the contents of the field at index `idx`. If the field is a bits union, this method
    /// will try to unbox the active variant. Pointer fields can also be accessed by using the
    /// approriate [`Ref`]. Returns an error if the index is out of bounds or if the layout of `T`
    /// is incompatible with the layout of that field in Julia.
    ///
    /// [`Ref`]: crate::wrappers::ptr::Ref
    pub fn get_nth_raw_field<T>(self, idx: usize) -> JlrsResult<T>
    where
        T: ValidLayout,
    {
        unsafe {
            if idx >= self.n_fields() as usize {
                Err(JlrsError::OutOfBounds {
                    idx,
                    n_fields: self.n_fields(),
                    value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                })?
            }

            self.read_field(idx as _)
        }
    }

    /// Access the contents of the field at index `idx` without checking bounds or if the layouts
    /// of the types are compatible.
    ///
    /// Safety: the layout out `T` must be compatible with the layout of the field and `idx` must
    /// not be out of bounds.
    pub unsafe fn get_nth_raw_field_unchecked<T>(self, idx: usize) -> T {
        let ty = self.datatype();
        let jl_type = ty.unwrap(Private);
        let field_offset = jl_field_offset(jl_type, idx as _);

        self.unwrap(Private)
            .cast::<u8>()
            .add(field_offset as usize)
            .cast::<T>()
            .read()
    }

    /// Access the contents of the field with the name `field_name`. If the field is a bits union,
    /// this method will try to unbox the active variant. Pointer fields can also be accessed by
    /// using the approriate [`Ref`]. Returns an error if there's no field with the given name or
    /// if the layout of `T` is incompatible with the layout of that field in Julia.
    ///
    /// [`Ref`]: crate::wrappers::ptr::Ref
    pub fn get_raw_field<T, N>(self, field_name: N) -> JlrsResult<T>
    where
        T: ValidLayout,
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = field_name.temporary_symbol(Private);
            let ty = self.datatype();
            let jl_type = ty.unwrap(Private);
            let idx = jl_field_index(jl_type, symbol.unwrap(Private), 0);

            if idx < 0 {
                Err(JlrsError::NoSuchField {
                    type_name: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            self.read_field(idx)
        }
    }

    /// Access the contents of the field with the name `field_name` without checking if the layouts
    /// of the types are compatible. Panics if the field doesn't exist.
    ///
    /// Safety: the layout out `T` must be compatible with the layout of the field.
    pub unsafe fn get_raw_field_unchecked<T, N>(self, field_name: N) -> T
    where
        N: TemporarySymbol,
    {
        let symbol = field_name.temporary_symbol(Private);
        let ty = self.datatype();
        let jl_type = ty.unwrap(Private);
        let idx = jl_field_index(jl_type, symbol.unwrap(Private), 0);

        assert!(idx >= 0, "Field {:?} doesn't exist", symbol);

        let field_offset = jl_field_offset(jl_type, idx as _);
        self.unwrap(Private)
            .cast::<u8>()
            .add(field_offset as usize)
            .cast::<T>()
            .read()
    }

    /// Roots the field at index `idx` if it exists and returns it, or
    /// `JlrsError::OutOfBounds` if the index is out of bounds.
    pub fn get_nth_field<'target, 'current, S, F>(
        self,
        scope: S,
        idx: usize,
    ) -> JlrsResult<S::Value>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            if idx >= self.n_fields() {
                Err(JlrsError::OutOfBounds {
                    idx,
                    n_fields: self.n_fields(),
                    value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                })?
            }

            let fld_ptr = jl_fieldref(self.unwrap(Private), idx as _);
            if fld_ptr.is_null() {
                Err(JlrsError::UndefRef)?;
            }

            scope.value(NonNull::new_unchecked(fld_ptr), Private)
        }
    }

    /// Returns the field at index `idx` if it's a pointer field.
    ///
    /// If the field doesn't exist `JlrsError::OutOfBounds` is returned. If the field can't be
    /// referenced because its data is stored inline, `JlrsError::NotAPointerField` is returned.
    pub fn get_nth_field_ref(self, idx: usize) -> JlrsResult<ValueRef<'scope, 'data>> {
        let ty = self.datatype();
        if idx >= ty.n_fields() as _ {
            Err(JlrsError::OutOfBounds {
                idx,
                n_fields: self.n_fields(),
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        unsafe {
            if !jl_field_isptr(ty.unwrap(Private), idx as _) {
                let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE);

                let field_name = if let Some(field_name) =
                    ty.field_names().wrapper_unchecked().data().get(idx)
                {
                    field_name
                        .wrapper_unchecked()
                        .as_str()
                        .unwrap_or("<Cannot display field name>")
                        .to_string()
                } else {
                    format!("{}", idx)
                };

                let field_type = ty.field_types().wrapper_unchecked().data()[idx]
                    .value_unchecked()
                    .display_string_or(CANNOT_DISPLAY_TYPE);

                Err(JlrsError::NotAPointerField {
                    value_type: value_type_str,
                    field_idx: idx,
                    field_name,
                    field_type,
                })?
            }

            Ok(ValueRef::wrap(jl_fieldref_noalloc(
                self.unwrap(Private),
                idx,
            )))
        }
    }

    /// Returns the field at index `idx` if it exists as a `ValueRef`. If the field is an inline
    /// field a new value is allocated which is left unrooted.
    ///
    /// If the field doesn't exist `JlrsError::OutOfBounds` is returned.
    pub fn get_nth_field_unrooted(self, idx: usize) -> JlrsResult<ValueRef<'scope, 'data>> {
        if idx >= self.n_fields() {
            Err(JlrsError::OutOfBounds {
                idx,
                n_fields: self.n_fields(),
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        unsafe { Ok(ValueRef::wrap(jl_fieldref(self.unwrap(Private), idx))) }
    }

    /// Roots the field with the name `field_name` if it exists and returns it, or
    /// `JlrsError::NoSuchField` if there's no field with that name.
    pub fn get_field<'target, 'current, N, S, F>(
        self,
        scope: S,
        field_name: N,
    ) -> JlrsResult<S::Value>
    where
        N: TemporarySymbol,
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            let symbol = field_name.temporary_symbol(Private);

            let jl_type = jl_typeof(self.unwrap(Private)).cast();
            let idx = jl_field_index(jl_type, symbol.unwrap(Private), 0);

            if idx < 0 {
                Err(JlrsError::NoSuchField {
                    type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            let fld_ptr = jl_fieldref(self.unwrap(Private), idx as _);
            if fld_ptr.is_null() {
                Err(JlrsError::UndefRef)?;
            }

            scope.value(NonNull::new_unchecked(fld_ptr), Private)
        }
    }

    /// Returns the field with the name `field_name` if it's a pointer field.
    ///
    /// If the field doesn't exist `JlrsError::NoSuchField` is returned. If it isn't a pointer
    /// field, `JlrsError::NotAPointerField` is returned.
    pub fn get_field_ref<N>(self, field_name: N) -> JlrsResult<ValueRef<'scope, 'data>>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = field_name.temporary_symbol(Private);
            let ty = self.datatype();

            let jl_type = jl_typeof(self.unwrap(Private)).cast();
            let idx = jl_field_index(jl_type, symbol.unwrap(Private), 0);

            if idx < 0 {
                Err(JlrsError::NoSuchField {
                    type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            if !jl_field_isptr(ty.unwrap(Private), idx as _) {
                let idx = idx as usize;
                let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE);

                let field_name = ty.field_names().wrapper_unchecked().data()[idx]
                    .wrapper_unchecked()
                    .as_str()
                    .unwrap_or("<Cannot display field name>")
                    .to_string();

                let field_type = ty.field_types().wrapper_unchecked().data()[idx]
                    .value_unchecked()
                    .display_string_or(CANNOT_DISPLAY_TYPE);

                Err(JlrsError::NotAPointerField {
                    value_type: value_type_str,
                    field_idx: idx,
                    field_name,
                    field_type,
                })?
            }

            Ok(ValueRef::wrap(jl_fieldref_noalloc(
                self.unwrap(Private),
                idx as _,
            )))
        }
    }

    /// Returns the field with the name `field_name` if it exists. If the field is an inline field
    /// a new value is allocated which is left unrooted.
    ///
    /// If the field doesn't  exist `JlrsError::NoSuchField` is returned.
    pub fn get_field_unrooted<N>(self, field_name: N) -> JlrsResult<ValueRef<'scope, 'data>>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = field_name.temporary_symbol(Private);

            let jl_type = jl_typeof(self.unwrap(Private)).cast();
            let idx = jl_field_index(jl_type, symbol.unwrap(Private), 0);

            if idx < 0 {
                Err(JlrsError::NoSuchField {
                    type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            Ok(ValueRef::wrap(jl_fieldref(self.unwrap(Private), idx as _)))
        }
    }

    /// Set the value of the field at `idx`. If Julia throws an exception it's caught, rooted in
    /// the frame, and returned. If the index is out of bounds or the value is not a subtype of
    /// the field an error is returned,
    ///
    /// Safety: Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is not prevented.
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
            Err(JlrsError::OutOfBounds {
                idx,
                n_fields,
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        let field_type = self.datatype().field_types().wrapper_unchecked().data()[idx as usize]
            .value_unchecked();
        let dt = value.datatype();

        if !Value::subtype(dt.as_value(), field_type) {
            Err(JlrsError::NotSubtype {
                field_type: field_type.display_string_or(CANNOT_DISPLAY_TYPE),
                value_type: value.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        let res = jlrs_set_nth_field(self.unwrap(Private), idx, value.unwrap(Private));
        if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
            let ptr = res.data;
            let err = crate::memory::scope::private::Scope::value(
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
    pub unsafe fn set_nth_field_unrooted(
        self,
        idx: usize,
        value: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResultRef<'scope, 'data, ()>> {
        let n_fields = self.n_fields();
        if n_fields <= idx {
            Err(JlrsError::OutOfBounds {
                idx,
                n_fields,
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        let field_type = self.datatype().field_types().wrapper_unchecked().data()[idx as usize]
            .value_unchecked();
        let dt = value.datatype();

        if !Value::subtype(dt.as_value(), field_type) {
            Err(JlrsError::NotSubtype {
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
    /// value is not a subtype of the field an error is returned,
    ///
    /// Safety: Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is not prevented.
    pub unsafe fn set_field<'frame, F, N>(
        self,
        frame: &mut F,
        field_name: N,
        value: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'frame, 'data, ()>>
    where
        F: Frame<'frame>,
        N: TemporarySymbol,
    {
        let symbol = field_name.temporary_symbol(Private);
        let jl_type = jl_typeof(self.unwrap(Private)).cast();
        let idx = jl_field_index(jl_type, symbol.unwrap(Private), 0);

        if idx < 0 {
            Err(JlrsError::NoSuchField {
                type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
            })?
        }

        let field_type = self.datatype().field_types().wrapper_unchecked().data()[idx as usize]
            .value_unchecked();
        let dt = value.datatype();

        if !Value::subtype(dt.as_value(), field_type) {
            Err(JlrsError::NotSubtype {
                field_type: field_type.display_string_or(CANNOT_DISPLAY_TYPE),
                value_type: value.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        let res = jlrs_set_nth_field(self.unwrap(Private), idx as usize, value.unwrap(Private));
        if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
            let ptr = res.data;
            let err = crate::memory::scope::private::Scope::value(
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
    /// is not a subtype of the field an error is returned,
    ///
    /// Safety: Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is not prevented.
    pub unsafe fn set_field_unrooted<N>(
        self,
        field_name: N,
        value: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResultRef<'scope, 'data, ()>>
    where
        N: TemporarySymbol,
    {
        let symbol = field_name.temporary_symbol(Private);
        let jl_type = jl_typeof(self.unwrap(Private)).cast();
        let idx = jl_field_index(jl_type, symbol.unwrap(Private), 0);

        if idx < 0 {
            Err(JlrsError::NoSuchField {
                type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
            })?
        }

        let field_type = self.datatype().field_types().wrapper_unchecked().data()[idx as usize]
            .value_unchecked();
        let dt = value.datatype();

        if !Value::subtype(dt.as_value(), field_type) {
            Err(JlrsError::NotSubtype {
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
        N: TemporarySymbol,
    {
        let symbol = field_name.temporary_symbol(Private);
        let jl_type = jl_typeof(self.unwrap(Private)).cast();
        let idx = jl_field_index(jl_type, symbol.unwrap(Private), 0);

        if idx < 0 {
            Err(JlrsError::NoSuchField {
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

    unsafe fn read_field<T>(self, idx: i32) -> JlrsResult<T>
    where
        T: ValidLayout,
    {
        let ty = self.datatype();
        let jl_type = ty.unwrap(Private);
        let field_offset = jl_field_offset(jl_type, idx as _);
        let mut field_type =
            ty.field_types().wrapper_unchecked().data()[idx as usize].value_unchecked();

        if let Ok(u) = field_type.cast::<Union>() {
            // If the field is a bits union, we want to access its current variant
            let mut size = 0;

            if u.isbits_size_align(&mut size, &mut 0) {
                let flag_offset = field_offset as usize + size;
                let mut flag = self.unwrap(Private).cast::<u8>().add(flag_offset).read() as i32;

                match nth_union_component(u.as_value(), &mut flag) {
                    Some(active_ty) => {
                        field_type = active_ty;
                    }
                    _ => (),
                }
            }
        } else if let Ok(_) = field_type.cast::<UnionAll>() {
            // If the field is a unionall, we want to take the concrete type into account
            field_type = self
                .unwrap(Private)
                .cast::<u8>()
                .add(field_offset as usize)
                .cast::<Value>()
                .read()
                .datatype()
                .as_value();
        }

        if T::valid_layout(field_type) {
            Ok(self
                .unwrap(Private)
                .cast::<u8>()
                .add(field_offset as usize)
                .cast::<T>()
                .read())
        } else {
            let value_type_str = field_type.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(JlrsError::InvalidLayout { value_type_str })?
        }
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
    pub unsafe fn eval_string<'target, 'current, C, S, F>(
        scope: S,
        cmd: C,
    ) -> JlrsResult<S::JuliaResult>
    where
        C: AsRef<str>,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
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
    pub unsafe fn eval_cstring<'target, 'current, C, S, F>(
        scope: S,
        cmd: C,
    ) -> JlrsResult<S::JuliaResult>
    where
        C: AsRef<CStr>,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
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
    pub unsafe fn include<'target, 'current, P, S, F>(
        scope: S,
        path: P,
    ) -> JlrsResult<S::JuliaResult>
    where
        P: AsRef<Path>,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        if path.as_ref().exists() {
            return scope.result_scope(|output, frame| {
                let global = frame.global();
                let path_jl_str = JuliaString::new(&mut *frame, path.as_ref().to_string_lossy())?;
                let include_func = Module::main(global)
                    .function_ref("include")?
                    .wrapper_unchecked();

                let scope = output.into_scope(frame);
                include_func.call1(scope, path_jl_str)
            });
        }

        Err(JlrsError::IncludeNotFound {
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
    pub unsafe fn add_finalizer(self, f: Value<'_, 'static>) {
        jl_gc_add_finalizer(self.unwrap(Private), f.unwrap(Private))
    }

    /// Add a finalizer `f` to this value. The finalizer must be an `extern "C"` function that
    /// takes one argument, the value as a void pointer.
    pub unsafe fn add_ptr_finalizer(self, f: unsafe extern "C" fn(*mut c_void) -> ()) {
        jl_gc_add_ptr_finalizer(jl_get_ptls_states(), self.unwrap(Private), f as *mut c_void)
    }

    /// Call all finalizers.
    pub unsafe fn finalize(self) {
        jl_finalize(self.unwrap(Private))
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

unsafe impl<'scope, 'data> ValidLayout for Value<'scope, 'data> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            !dt.is_inline_alloc()
        } else if v.cast::<UnionAll>().is_ok() {
            true
        } else if let Ok(u) = v.cast::<Union>() {
            !u.is_bits_union()
        } else {
            false
        }
    }
}

impl<'data> Call<'data> for Value<'_, 'data> {
    unsafe fn call0<'target, 'current, S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        let res = self.call0_unrooted(scope.global());
        scope.unrooted_call_result(res, Private)
    }

    unsafe fn call1<'target, 'current, S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        let res = self.call1_unrooted(scope.global(), arg0);
        scope.unrooted_call_result(res, Private)
    }

    unsafe fn call2<'target, 'current, S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        let res = self.call2_unrooted(scope.global(), arg0, arg1);
        scope.unrooted_call_result(res, Private)
    }

    unsafe fn call3<'target, 'current, S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        let res = self.call3_unrooted(scope.global(), arg0, arg1, arg2);
        scope.unrooted_call_result(res, Private)
    }

    unsafe fn call<'target, 'current, 'value, V, S, F>(
        self,
        scope: S,
        args: V,
    ) -> JlrsResult<S::JuliaResult>
    where
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        let res = self.call_unrooted(scope.global(), args);
        scope.unrooted_call_result(res, Private)
    }

    unsafe fn call0_unrooted<'target>(self, _: Global<'target>) -> JuliaResultRef<'target, 'data> {
        let res = jl_call0(self.unwrap(Private));
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Err(ValueRef::wrap(exc))
        }
    }

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

    unsafe fn call_unrooted<'target, 'value, V>(
        self,
        _: Global<'target>,
        mut args: V,
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        let args = args.as_mut();
        let n = args.len();
        let res = jl_call(
            self.unwrap(Private).cast(),
            args.as_mut_ptr().cast(),
            n as _,
        );
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Err(ValueRef::wrap(exc))
        }
    }
}

impl<'target, 'current, 'value, 'data> CallExt<'target, 'current, 'value, 'data>
    for Value<'value, 'data>
{
    fn with_keywords(self, kws: Value<'value, 'data>) -> JlrsResult<WithKeywords<'value, 'data>> {
        if !kws.is::<NamedTuple>() {
            let type_str = kws.datatype().display_string_or(CANNOT_DISPLAY_TYPE);
            Err(JlrsError::NotANamedTuple { type_str })?
        }
        Ok(WithKeywords::new(self, kws))
    }
}

impl_debug!(Value<'_, '_>);

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Value<'scope, 'data> {
    type Wraps = jl_value_t;
    const NAME: &'static str = "Value";

    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// While jlrs generally enforces that Julia data can only exist and be used while a frame is
/// active, it's possible to leak global values: [`Symbol`]s, [`Module`]s, and globals defined in
/// those modules.
pub struct LeakedValue(Value<'static, 'static>);

impl LeakedValue {
    pub(crate) unsafe fn wrap(ptr: *mut jl_value_t) -> Self {
        LeakedValue(Value::wrap(ptr, Private))
    }

    pub(crate) unsafe fn wrap_non_null(ptr: NonNull<jl_value_t>) -> Self {
        LeakedValue(Value::wrap_non_null(ptr, Private))
    }

    /// Convert this [`LeakedValue`] back to a [`Value`]. This requires a [`Global`], so this
    /// method can only be called inside a closure taken by one of the `scope`-methods.
    ///
    /// Safety: you must guarantee this value has not been freed by the garbage collector. While
    /// `Symbol`s are never garbage collected, modules and their contents can be redefined.
    pub unsafe fn as_value<'scope>(self, _: Global<'scope>) -> Value<'scope, 'static> {
        self.0
    }
}
