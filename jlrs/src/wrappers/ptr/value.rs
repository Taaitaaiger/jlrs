//! Wrapper for arbitrary Julia data.
//!
//! Julia data returned by the C API is normally returned as a pointer to `jl_value_t`, which is
//! an opaque type. This pointer is wrapped in jlrs by [`Value`]. The layout of the data that is
//! pointed to depends on its underlying type. Julia guarantees that the data is preceded in
//! memory by a header which contains a pointer to the data's [`DataType`], its type information.
//! For example, if the [`DataType`] is `UInt8`, the pointer points to a `u8`. If the [`DataType`]
//! is some Julia array type like `Array{Int, 2}`, the pointer points to Julia's internal array
//! type, `jl_array_t`.
//!
//! The [`Value`] wrapper is very commonly used in jlrs. A [`Value`] can be called as a Julia
//! function, the arguments this functions takes are all [`Value`]s, and it will return either a
//! [`Value`] or an exception, which is also a [`Value`].
//!
//! One special kind of value is the `NamedTuple`. You will need to create values of this type in
//! order to call functions with keyword arguments. The macro [`named_tuple`] is defined in this
//! module which provides an easy way to create values of this type.

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
    error::{JlrsError, JlrsResult, JuliaResult},
    layout::{
        typecheck::{Mutable, Typecheck},
        valid_layout::ValidLayout,
    },
    memory::{
        global::Global,
        traits::{
            frame::{private::Frame as _, Frame},
            scope::{private::Scope as _, Scope},
        },
    },
    private::Private,
    wrappers::ptr::{
        array::{
            dimensions::{Dimensions, Dims},
            Array,
        },
        datatype::DataType,
        module::Module,
        private::Wrapper as WrapperPriv,
        symbol::Symbol,
        type_var::TypeVar,
        union::Union,
        union_all::UnionAll,
        ValueRef, Wrapper,
    },
};
use jl_sys::{
    jl_alloc_array_1d, jl_alloc_array_2d, jl_alloc_array_3d, jl_an_empty_string,
    jl_an_empty_vec_any, jl_any_type, jl_apply_array_type, jl_apply_tuple_type_v, jl_apply_type,
    jl_array_any_type, jl_array_int32_type, jl_array_symbol_type, jl_array_uint8_type,
    jl_bottom_type, jl_datatype_t, jl_diverror_exception, jl_egal, jl_emptytuple, jl_eval_string,
    jl_exception_occurred, jl_false, jl_field_index, jl_field_isptr, jl_field_names, jl_fieldref,
    jl_fieldref_noalloc, jl_finalize, jl_gc_add_finalizer, jl_gc_wb, jl_get_nth_field,
    jl_get_nth_field_noalloc, jl_interrupt_exception, jl_is_kind, jl_isa, jl_memory_exception,
    jl_new_array, jl_new_struct_uninit, jl_new_typevar, jl_nfields, jl_nothing, jl_object_id,
    jl_pchar_to_string, jl_ptr_to_array, jl_ptr_to_array_1d, jl_readonlymemory_exception,
    jl_set_nth_field, jl_stackovf_exception, jl_subtype, jl_svec_data, jl_svec_len, jl_true,
    jl_type_union, jl_type_unionall, jl_typeof, jl_typeof_str, jl_undefref_exception, jl_value_t,
};
use std::{
    cell::UnsafeCell,
    ffi::{CStr, CString},
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
    slice, usize,
};

/// In some cases it's necessary to place one or more arguments in front of the arguments a
/// function is called with. Examples include the `named_tuple` macro and `Value::call_async`.
/// If they are called with fewer than `MAX_SIZE` arguments (including the added arguments), no
/// heap allocation is required to store them.
pub const MAX_SIZE: usize = 8;

thread_local! {
    // Used to convert dimensions to tuples. Safe because a thread local is initialized
    // when `with` is first called, which happens after `Julia::init` has been called. The C API
    // requires a mutable pointer to this array so an `UnsafeCell` is used to store it.
    static JL_LONG_TYPE: UnsafeCell<[*mut jl_datatype_t; 8]> = unsafe {
        let global = Global::new();
        let t = usize::julia_type(global).ptr();
        UnsafeCell::new([
            t,
            t,
            t,
            t,
            t,
            t,
            t,
            t
        ])
    };
}

/// A `Value` is a wrapper around a pointer to some data owned by the Julia garbage collector, it
/// has two lifetimes: `'frame` and `'data`. The first of these ensures that a `Value` can only be
/// used while it's rooted in a `GcFrame`, the second accounts for data borrowed from
/// Rust. The only way to borrow data from Rust is to create an Julia array that borrows its
/// contents by calling `Value::borrow_array`, if a Julia function is called with such an array as
/// an argument the result will inherit the second lifetime of the borrowed data to ensure that
/// such a `Value` can onl be used while the borrow is active.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Value<'frame, 'data>(
    NonNull<jl_value_t>,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
);

/// # Create new `Value`s
///
/// Several methods are available to create new values. The simplest of these is [`Value::new`],
/// which can be used to convert relatively simple data from Rust to Julia. Data that can be
/// converted this way must implement [`IntoJulia`], which is the case for some simple types like
/// primitive number types. This trait is also automatically derived by `JlrsReflect.jl` for types
/// that are trivially guaranteed to be bits-types: the type must have no type parameters, no
/// unions, and all fields must be immutable bits-types themselves.
///
/// Data that isn't supported by [`Value::new`] can still be created from Rust in many cases.
/// Strings can be allocated with [`Value::new_string`]. For types that implement `IntoJulia`,
/// arrays can be created with [`Value::new_array`]. If you want to have the array be backed by
/// data from Rust, [`Value::borrow_array`] and [`Value::move_array`] can be used. In order to
/// create a new array for other types [`Value::new_array_for`] must be used. There are also
/// methods to create new [`UnionAll`]s, [`Union`]s and [`TypeVar`]s.
///
/// Finally, it's possible to instantiate arbitrary concrete types with [`Value::instantiate`],
/// the type parameters of types that have them can be set with [`Value::apply_type`]. These
/// methods don't support creating new arrays.
impl<'frame, 'data> Value<'frame, 'data> {
    /// Create a new Julia value, any type that implements [`IntoJulia`] can be converted using
    /// this function.
    pub fn new<'scope, V, S, F>(scope: S, value: V) -> JlrsResult<S::Value>
    where
        V: IntoJulia,
        S: Scope<'scope, 'frame, 'static, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let global = scope.global();
            let v = value.into_julia(global).ptr();
            debug_assert!(!v.is_null());
            scope.value(NonNull::new_unchecked(v), Private)
        }
    }

    /// Create a new Julia string.
    pub fn new_string<'scope, V, S, F>(scope: S, value: V) -> JlrsResult<S::Value>
    where
        V: AsRef<str>,
        S: Scope<'scope, 'frame, 'static, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let ptr = value.as_ref().as_ptr().cast();
            let len = value.as_ref().len();
            let s = jl_pchar_to_string(ptr, len);
            debug_assert!(!s.is_null());
            scope.value(NonNull::new_unchecked(s), Private)
        }
    }

    /// Create a new instance of a value with `DataType` `ty`, using `values` to set the fields.
    /// This is essentially a more powerful version of [`Value::new`] and can instantiate
    /// arbitrary concrete `DataType`s, at the cost that each of its fields must have already been
    /// allocated as a `Value`. This functions returns an error if the given `DataType` is not
    /// concrete or an array type.
    pub fn instantiate<'scope, 'value, 'borrow, V, S, F>(
        scope: S,
        ty: DataType,
        values: V,
    ) -> JlrsResult<S::Value>
    where
        V: AsMut<[Value<'value, 'borrow>]>,
        S: Scope<'scope, 'frame, 'borrow, F>,
        F: Frame<'frame>,
    {
        ty.instantiate(scope, values)
    }

    /// Allocates a new n-dimensional array in Julia. This method can only be used in combination
    /// with types that implement `IntoJulia`. These traits are implemented for primitive types
    /// like `u8` and [`Bool`], and can be derived for bits-types with `JlrsReflect.jl`. If you
    /// want to create an array for a type that does not implement these traits you can use
    /// [`Value::new_array_for`].
    pub fn new_array<'scope, T, D, S, F>(scope: S, dims: D) -> JlrsResult<S::Value>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'scope, 'frame, 'static, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let global = scope.global();
            let array_type =
                jl_apply_array_type(T::julia_type(global).ptr().cast(), dims.n_dimensions());

            match dims.n_dimensions() {
                1 => scope.value(
                    NonNull::new_unchecked(
                        jl_alloc_array_1d(array_type, dims.n_elements(0)).cast(),
                    ),
                    Private,
                ),
                2 => scope.value(
                    NonNull::new_unchecked(
                        jl_alloc_array_2d(array_type, dims.n_elements(0), dims.n_elements(1))
                            .cast(),
                    ),
                    Private,
                ),
                3 => scope.value(
                    NonNull::new_unchecked(
                        jl_alloc_array_3d(
                            array_type,
                            dims.n_elements(0),
                            dims.n_elements(1),
                            dims.n_elements(2),
                        )
                        .cast(),
                    ),
                    Private,
                ),
                n if n <= 8 => scope.value_scope_with_slots(1, |output, frame| {
                    let tuple = small_dim_tuple(frame, &dims.into_dimensions())?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(
                            jl_new_array(array_type, tuple.unwrap(Private)).cast(),
                        ),
                        Private,
                    )
                }),
                _ => scope.value_scope_with_slots(1, |output, frame| {
                    let tuple = large_dim_tuple(frame, &dims.into_dimensions())?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(
                            jl_new_array(array_type, tuple.unwrap(Private)).cast(),
                        ),
                        Private,
                    )
                }),
            }
        }
    }

    /// Allocates a new n-dimensional array in Julia for elements of type `ty`, which must be a
    /// `Union`, `UnionAll` or `DataType`.
    pub fn new_array_for<'scope, D, S, F>(scope: S, dims: D, ty: Value) -> JlrsResult<S::Value>
    where
        D: Dims,
        S: Scope<'scope, 'frame, 'static, F>,
        F: Frame<'frame>,
    {
        if !ty.is_type() {
            Err(JlrsError::NotAType)?
        }

        unsafe {
            let array_type = jl_apply_array_type(ty.unwrap(Private), dims.n_dimensions());

            match dims.n_dimensions() {
                1 => scope.value(
                    NonNull::new_unchecked(
                        jl_alloc_array_1d(array_type, dims.n_elements(0)).cast(),
                    ),
                    Private,
                ),
                2 => scope.value(
                    NonNull::new_unchecked(
                        jl_alloc_array_2d(array_type, dims.n_elements(0), dims.n_elements(1))
                            .cast(),
                    ),
                    Private,
                ),
                3 => scope.value(
                    NonNull::new_unchecked(
                        jl_alloc_array_3d(
                            array_type,
                            dims.n_elements(0),
                            dims.n_elements(1),
                            dims.n_elements(2),
                        )
                        .cast(),
                    ),
                    Private,
                ),
                n if n <= 8 => scope.value_scope_with_slots(1, |output, frame| {
                    let tuple = small_dim_tuple(frame, &dims.into_dimensions())?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(
                            jl_new_array(array_type, tuple.unwrap(Private)).cast(),
                        ),
                        Private,
                    )
                }),
                _ => scope.value_scope_with_slots(1, |output, frame| {
                    let tuple = large_dim_tuple(frame, &dims.into_dimensions())?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(
                            jl_new_array(array_type, tuple.unwrap(Private)).cast(),
                        ),
                        Private,
                    )
                }),
            }
        }
    }

    /// Borrows an n-dimensional array from Rust for use in Julia.
    pub fn borrow_array<'scope, T, D, V, S, F>(
        scope: S,
        mut data: V,
        dims: D,
    ) -> JlrsResult<S::Value>
    where
        T: IntoJulia,
        D: Dims,
        V: AsMut<[T]> + 'data,
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let global = scope.global();
            let array_type =
                jl_apply_array_type(T::julia_type(global).ptr().cast(), dims.n_dimensions());

            match dims.n_dimensions() {
                1 => scope.value(
                    NonNull::new_unchecked(
                        jl_ptr_to_array_1d(
                            array_type,
                            data.as_mut().as_mut_ptr().cast(),
                            dims.n_elements(0),
                            0,
                        )
                        .cast(),
                    ),
                    Private,
                ),
                n if n <= 8 => scope.value_scope_with_slots(1, |output, frame| {
                    let tuple = small_dim_tuple(frame, &dims.into_dimensions())?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(
                            jl_ptr_to_array(
                                array_type,
                                data.as_mut().as_mut_ptr().cast(),
                                tuple.unwrap(Private),
                                0,
                            )
                            .cast(),
                        ),
                        Private,
                    )
                }),
                _ => scope.value_scope_with_slots(1, |output, frame| {
                    let tuple = large_dim_tuple(frame, &dims.into_dimensions())?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(
                            jl_ptr_to_array(
                                array_type,
                                data.as_mut().as_mut_ptr().cast(),
                                tuple.unwrap(Private),
                                0,
                            )
                            .cast(),
                        ),
                        Private,
                    )
                }),
            }
        }
    }

    /// Moves an n-dimensional array from Rust to Julia.
    pub fn move_array<'scope, T, D, S, F>(scope: S, data: Vec<T>, dims: D) -> JlrsResult<S::Value>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'scope, 'frame, 'static, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let global = scope.global();
            let finalizer = Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("clean")?
                .wrapper_unchecked();

            scope.value_scope_with_slots(2, |output, frame| {
                let array_type =
                    jl_apply_array_type(T::julia_type(global).ptr().cast(), dims.n_dimensions());
                let _ = frame
                    .push_root(NonNull::new_unchecked(array_type), Private)
                    .map_err(JlrsError::alloc_error)?;

                match dims.n_dimensions() {
                    1 => {
                        let array = jl_ptr_to_array_1d(
                            array_type,
                            Box::into_raw(data.into_boxed_slice()).cast(),
                            dims.n_elements(0),
                            0,
                        )
                        .cast();

                        jl_gc_add_finalizer(array, finalizer.unwrap(Private));
                        output
                            .into_scope(frame)
                            .value(NonNull::new_unchecked(array), Private)
                    }
                    n if n <= 8 => {
                        let tuple = small_dim_tuple(frame, &dims.into_dimensions())?;
                        let array = jl_ptr_to_array(
                            array_type,
                            Box::into_raw(data.into_boxed_slice()).cast(),
                            tuple.unwrap(Private),
                            0,
                        )
                        .cast();

                        jl_gc_add_finalizer(array, finalizer.unwrap(Private));
                        output
                            .into_scope(frame)
                            .value(NonNull::new_unchecked(array), Private)
                    }
                    _ => {
                        let tuple = large_dim_tuple(frame, &dims.into_dimensions())?;
                        let array = jl_ptr_to_array(
                            array_type,
                            Box::into_raw(data.into_boxed_slice()).cast(),
                            tuple.unwrap(Private),
                            0,
                        )
                        .cast();

                        jl_gc_add_finalizer(array, finalizer.unwrap(Private));
                        output
                            .into_scope(frame)
                            .value(NonNull::new_unchecked(array), Private)
                    }
                }
            })
        }
    }

    /// Returns the union of all types in `types`. For each of these types, [`Value::is_kind`]
    /// must return `true`. Note that the result is not necessarily a [`Union`], for example the
    /// union of a single [`DataType`] is that type, not a `Union` with a single variant.
    ///
    /// [`Union`]: crate::wrappers::ptr::union::Union
    pub fn new_union<'scope, S, F>(scope: S, types: &mut [Value<'_, 'data>]) -> JlrsResult<S::Value>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            if let Some(v) = types
                .iter()
                .find_map(|v| if v.is_kind() { None } else { Some(v) })
            {
                Err(JlrsError::NotAKind(v.type_name()?.into()))?;
            }

            let un = jl_type_union(types.as_mut_ptr().cast(), types.len());
            scope.value(NonNull::new_unchecked(un), Private)
        }
    }

    /// Create a new [`UnionAll`].
    pub fn new_unionall<'scope, S, F>(
        scope: S,
        tvar: TypeVar,
        body: Value<'_, 'data>,
    ) -> JlrsResult<S::Value>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        if !body.is_type() && !body.is::<TypeVar>() {
            Err(JlrsError::InvalidBody(body.type_name()?.into()))?;
        }

        unsafe {
            let ua = jl_type_unionall(tvar.unwrap(Private), body.unwrap(Private));
            scope.value(NonNull::new_unchecked(ua), Private)
        }
    }

    /// Create a new named tuple, you can use the `named_tuple` macro instead of this method.
    pub fn new_named_tuple<'scope, 'value, S, F, N, T, V>(
        scope: S,
        mut field_names: N,
        mut values: V,
    ) -> JlrsResult<S::Value>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
        N: AsMut<[T]>,
        T: TemporarySymbol,
        V: AsMut<[Value<'value, 'data>]>,
    {
        scope.value_scope_with_slots(4, |output, frame| unsafe {
            let global = frame.global();
            let field_names = field_names.as_mut();
            let values_m = values.as_mut();

            let n_field_names = field_names.len();
            let n_values = values_m.len();

            if n_field_names != n_values {
                Err(JlrsError::NamedTupleSizeMismatch(n_field_names, n_values))?;
            }

            let symbol_ty = DataType::symbol_type(global).as_value();
            let mut symbol_type_vec = vec![symbol_ty; n_field_names];

            let mut field_names_vec = field_names
                .iter()
                .map(|name| name.temporary_symbol(Private).as_value())
                .collect::<Vec<_>>();

            let names = DataType::anytuple_type(global)
                .as_value()
                .apply_type(&mut *frame, &mut symbol_type_vec)?
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut field_names_vec)?;

            let mut field_types_vec = values_m
                .iter()
                .copied()
                .map(|val| val.datatype().as_value())
                .collect::<Vec<_>>();

            let field_type_tup = DataType::anytuple_type(global)
                .as_value()
                .apply_type(&mut *frame, &mut field_types_vec)?;

            let ty = UnionAll::namedtuple_type(global)
                .as_value()
                .apply_type(&mut *frame, &mut [names, field_type_tup])?
                .cast::<DataType>()?;

            let output = output.into_scope(frame);
            ty.instantiate(output, values)
        })
    }

    /// Create a new `TypeVar`, the optional lower and upper bounds must be subtypes of `Type`,
    /// their default values are `Union{}` and `Any` respectively.
    pub fn new_typevar<'scope, S, F, N>(
        scope: S,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> JlrsResult<S::Value>
    where
        F: Frame<'frame>,
        S: Scope<'scope, 'frame, 'data, F>,
        N: TemporarySymbol,
    {
        unsafe {
            let global = Global::new();
            let name = name.temporary_symbol(Private);

            let lb = lower_bound.map_or(jl_bottom_type.cast(), |v| v.unwrap(Private));
            if !Value::wrap(lb, Private)
                .datatype()
                .as_value()
                .subtype(UnionAll::type_type(global).as_value())
            {
                Err(JlrsError::NotATypeLB(
                    name.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                ))?;
            }

            let ub = upper_bound.map_or(jl_any_type.cast(), |v| v.unwrap(Private));
            if !Value::wrap(ub, Private)
                .datatype()
                .as_value()
                .subtype(UnionAll::type_type(global).as_value())
            {
                Err(JlrsError::NotATypeUB(
                    name.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                ))?;
            }

            let tvar = jl_new_typevar(name.unwrap(Private), lb, ub);
            scope.value(NonNull::new_unchecked(tvar.cast()), Private)
        }
    }

    /// Apply the given types to `self`.
    ///
    /// If `self` is the [`DataType`] `anytuple_type`, calling this function will return a new
    /// tuple type with the given types as its field types. If it is the [`DataType`]
    /// `uniontype_type`, calling this function is equivalent to calling [`Value::new_union`]. If
    /// the value is a `UnionAll`, the given types will be applied and the resulting type is
    /// returned.
    ///
    /// If the types cannot be applied to `self` your program will abort.
    pub fn apply_type<'scope, 'fr, 'value, 'borrow, S, F, V>(
        self,
        scope: S,
        mut types: V,
    ) -> JlrsResult<S::Value>
    where
        S: Scope<'scope, 'fr, 'borrow, F>,
        F: Frame<'fr>,
        V: AsMut<[Value<'value, 'borrow>]>,
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
impl<'frame, 'data> Value<'frame, 'data> {
    /// Returns the `DataType` of this value.
    pub fn datatype(self) -> DataType<'frame> {
        unsafe { DataType::wrap(jl_typeof(self.unwrap(Private)).cast(), Private) }
    }

    // TODO: rename, type_name clashes with TypeName
    /// Returns the type name of this value as a string slice.
    pub fn type_name(self) -> JlrsResult<&'frame str> {
        unsafe {
            let type_name = jl_typeof_str(self.unwrap(Private));
            let type_name_ref = CStr::from_ptr(type_name);
            Ok(type_name_ref.to_str().map_err(|_| JlrsError::NotUnicode)?)
        }
    }
}

/// # Type checking
///
/// Many properties of Julia types can be checked, including whether instances of the type are
/// mutable, if the value is an array, and so on. The method [`Value::is`] can be used to perform
/// these checks. All these checks implement the [`Typecheck`] trait. If the type that implements
/// this trait also implements `ValidLayout`, the typecheck indicates whether or not the value can
/// be cast to or unboxed as that type.
impl<'frame, 'data> Value<'frame, 'data> {
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
    /// If you derive [`JuliaStruct`] for some type, that type will be supported by this method. A
    /// full list of supported checks can be found [here].
    ///
    /// [`JuliaStruct`]: crate::wrappers::ptr::traits::julia_struct::JuliaStruct
    /// [here]: ../layout/typecheck/trait.Typecheck.html#implementors
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
/// Values have two lifetimes, `'frame` and `'data`. The first ensures that a value can only be
/// used while it's rooted in a frame, the second ensures that values that (might) borrow array
/// data from Rust are also restricted by the lifetime of that borrow. This second restriction
/// can be relaxed with [`Value::assume_owned`] if it doesn't borrow any data from Rust.
impl<'frame, 'data> Value<'frame, 'data> {
    /// If you call a function with one or more borrowed arrays as arguments, its result can only
    /// be used when all the borrows are active. If this result doesn't reference any borrowed
    /// data this function can be used to relax its second lifetime to `'static`.
    ///
    /// Safety: The value must not contain a reference any borrowed data.
    pub unsafe fn assume_owned(self) -> Value<'frame, 'static> {
        Value::wrap_non_null(self.unwrap_non_null(Private), Private)
    }

    /// Root the value in some `scope`.
    pub fn root<'scope, 'f, S, F>(self, scope: S) -> JlrsResult<S::Value>
    where
        F: Frame<'f>,
        S: Scope<'scope, 'f, 'data, F>,
    {
        unsafe { scope.value(self.unwrap_non_null(Private), Private) }
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
/// can be implemented for other types. It's recommended that you use `JlrsReflect.jl` to do so.
/// Unlike casting, unboxing dereferences the pointer. As a result it loses its header, so an
/// unboxed value can't be used as a [`Value`] again without reallocating it.
impl<'frame, 'data> Value<'frame, 'data> {
    /// Cast the value to a pointer wrapper type `T`. Returns an error if the conversion is
    /// invalid.
    pub fn cast<T: Wrapper<'frame, 'data> + Typecheck>(self) -> JlrsResult<T> {
        if self.is::<T>() {
            unsafe { Ok(T::cast(self, Private)) }
        } else {
            Err(JlrsError::WrongType)?
        }
    }

    /// Cast the value to a pointer wrapper type `T` without checking if this conversion is valid.
    ///
    /// Safety:
    ///
    /// You must guarantee `self.is::<T>()` would have returned `true`.
    pub unsafe fn cast_unchecked<T: Wrapper<'frame, 'data>>(self) -> T {
        T::cast(self, Private)
    }

    /// Unbox the contents of the value as the output type associated with `T`. Returns an error
    /// if the layout of `T::Output` is incompatible with the layout of the type in Julia.
    pub fn unbox<T: Unbox + Typecheck>(self) -> JlrsResult<T::Output> {
        if !self.is::<T>() {
            Err(JlrsError::WrongType)?;
        }

        unsafe { Ok(T::unbox(self)) }
    }

    ///  Unbox the contents of the value as the output type associated with `T` without checking
    /// if the layout of `T::Output` is compatible with the layout of the type in Julia.
    ///
    /// Safety:
    ///
    /// You must guarantee `self.is::<T>()` would have returned `true`.
    pub fn unbox_unchecked<T: Unbox>(self) -> T::Output {
        unsafe { T::unbox(self) }
    }
}

/// # Fields
///
/// Julia values can have fields. For example, if the value contains an instance of this struct:
///
/// ```julia
/// struct Example
///    fielda
///    fieldb::UInt32
/// end
/// ```
///
/// it will have two fields, `fielda` and `fieldb`. The first field is stored as a [`Value`], the
/// second field is stored inline as a `u32`. If the second field is converted to a [`Value`] with
/// one of the field access methods below, a new value is allocated. The first field can be
/// accessed without allocating because it already is a [`Value`].
///
/// However, there is a technical detail you should be aware of: while no new value has to be
/// allocated when accessing a field that is already stored as a `Value`, when dealing with
/// mutable types these values can become unreachable. For this reason, when a field that is
/// stored as a `Value` is accessed without rooting it, it is returned as a [`ValueRef`].
impl<'frame, 'data> Value<'frame, 'data> {
    /// Returns the field names of this value as a slice of `Symbol`s. These symbols can be used
    /// to access their fields with [`Value::get_field`].
    pub fn field_names(self) -> &'frame [Symbol<'frame>] {
        unsafe {
            let tp = jl_typeof(self.unwrap(Private));
            let field_names = jl_field_names(tp.cast());
            let len = jl_svec_len(field_names);
            let items = jl_svec_data(field_names);
            slice::from_raw_parts(items.cast(), len)
        }
    }

    /// Returns the number of fields the underlying Julia value has. These fields can be accessed
    /// with [`Value::get_nth_field`].
    pub fn n_fields(self) -> usize {
        unsafe { jl_nfields(self.unwrap(Private)) as _ }
    }

    /// Returns the field at index `idx` if it exists. If it does not exist
    /// `JlrsError::OutOfBounds` is returned.
    pub fn get_nth_field<'scope, 'fr, S, F>(self, scope: S, idx: usize) -> JlrsResult<S::Value>
    where
        S: Scope<'scope, 'fr, 'data, F>,
        F: Frame<'fr>,
    {
        unsafe {
            if idx >= self.n_fields() {
                return Err(JlrsError::OutOfBounds(idx, self.n_fields()).into());
            }

            scope.value(
                NonNull::new_unchecked(jl_fieldref(self.unwrap(Private), idx)),
                Private,
            )
        }
    }

    /// Returns the field at index `idx` if it exists as a `ValueRef`.
    ///
    /// If the field does not exist `JlrsError::OutOfBounds` is returned. If the field can't be
    /// referenced because it's data is stored inline, `JlrsError::NotAPointerField` is returned.
    pub fn get_nth_field_ref(self, idx: usize) -> JlrsResult<ValueRef<'frame, 'data>> {
        if idx >= self.n_fields() {
            Err(JlrsError::OutOfBounds(idx, self.n_fields()))?
        }

        unsafe {
            if !jl_field_isptr(self.datatype().unwrap(Private), idx as _) {
                Err(JlrsError::NotAPointerField(idx))?;
            }

            Ok(ValueRef::wrap(jl_fieldref_noalloc(
                self.unwrap(Private),
                idx,
            )))
        }
    }

    /// Returns the field with the name `field_name` if it exists as a `ValueRef`.
    ///
    /// If the field does not exist `JlrsError::NoSuchField` is returned. If the field can't be
    /// referenced because it's data is stored inline, `JlrsError::NotAPointerField` is returned.
    pub fn get_field<'scope, 'fr, N, S, F>(self, scope: S, field_name: N) -> JlrsResult<S::Value>
    where
        N: TemporarySymbol,
        S: Scope<'scope, 'fr, 'data, F>,
        F: Frame<'fr>,
    {
        unsafe {
            let symbol = field_name.temporary_symbol(Private);

            let jl_type = jl_typeof(self.unwrap(Private)).cast();
            let idx = jl_field_index(jl_type, symbol.unwrap(Private), 0);

            if idx < 0 {
                return Err(JlrsError::NoSuchField(
                    symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                )
                .into());
            }

            scope.value(
                NonNull::new_unchecked(jl_get_nth_field(self.unwrap(Private), idx as _)),
                Private,
            )
        }
    }

    /// Returns the field with the name `field_name` if it exists and no allocation is required
    /// to return it. Allocation is not required if the field is a pointer to another value.
    ///
    /// If the field does not exist `JlrsError::NoSuchField` is returned. If allocating is
    /// required to return the field, `JlrsError::NotAPointerField` is returned.
    pub fn get_field_ref<N>(self, field_name: N) -> JlrsResult<ValueRef<'frame, 'data>>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = field_name.temporary_symbol(Private);

            let jl_type = jl_typeof(self.unwrap(Private)).cast();
            let idx = jl_field_index(jl_type, symbol.unwrap(Private), 0);

            if idx < 0 {
                return Err(JlrsError::NoSuchField(
                    symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                )
                .into());
            }

            if !jl_field_isptr(self.datatype().unwrap(Private), idx) {
                Err(JlrsError::NotAPointerField(idx as _))?;
            }

            Ok(ValueRef::wrap(jl_get_nth_field_noalloc(
                self.unwrap(Private),
                idx as _,
            )))
        }
    }

    /// Set the value of the field at `idx`. Returns an error if this value is immutable, `idx` is
    /// out of bounds, or if the type of `value` is not a subtype of the field type.
    pub fn set_nth_field(self, idx: usize, value: Value) -> JlrsResult<()> {
        if !self.is::<Mutable>() {
            Err(JlrsError::Immutable)?
        }

        if idx >= self.n_fields() {
            Err(JlrsError::OutOfBounds(idx, self.n_fields()))?
        }

        unsafe {
            let field_type =
                self.datatype().field_types().wrapper_unchecked().data()[idx].value_unchecked();
            let dt = value.datatype();

            if Value::subtype(dt.as_value(), field_type) {
                jl_set_nth_field(self.unwrap(Private), idx, value.unwrap(Private));
                jl_gc_wb(self.unwrap(Private), value.unwrap(Private));
                Ok(())
            } else {
                Err(JlrsError::NotSubtype)?
            }
        }
    }

    /// Set the value of the `field_name`. Returns an error if this value is immutable, the field
    /// doesn't exist, or if the type of `value` is not a subtype of the field type.
    pub fn set_field<N>(self, field_name: N, value: Value) -> JlrsResult<()>
    where
        N: TemporarySymbol,
    {
        if !self.is::<Mutable>() {
            Err(JlrsError::Immutable)?
        }

        unsafe {
            let symbol = field_name.temporary_symbol(Private);

            let jl_type = jl_typeof(self.unwrap(Private)).cast();
            let idx = jl_field_index(jl_type, symbol.unwrap(Private), 0);

            if idx < 0 {
                return Err(JlrsError::NoSuchField(
                    symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                )
                .into());
            }

            let field_type = self.datatype().field_types().wrapper_unchecked().data()[idx as usize]
                .value_unchecked();
            let dt = value.datatype();

            if Value::subtype(dt.as_value(), field_type) {
                jl_set_nth_field(self.unwrap(Private), idx as _, value.unwrap(Private));
                jl_gc_wb(self.unwrap(Private), value.unwrap(Private));
                Ok(())
            } else {
                Err(JlrsError::NotSubtype)?
            }
        }
    }
}

/// # Evaluate Julia code
///
/// The easiest way to call Julia from Rust is by evaluating some Julia code directly. This can be
/// used to call simple functions without any arguments provided from Rust and to execute
/// using-statements.
impl<'data> Value<'_, 'data> {
    /// Execute a Julia command `cmd`, for example `Value::eval_string(&mut *frame, "sqrt(2)")` or
    /// `Value::eval_string(&mut *frame, "using LinearAlgebra")`.
    pub fn eval_string<'frame, F, S>(
        frame: &mut F,
        cmd: S,
    ) -> JlrsResult<JuliaResult<'frame, 'static>>
    where
        F: Frame<'frame>,
        S: AsRef<str>,
    {
        unsafe {
            let cmd = cmd.as_ref();
            let cmd_cstring = CString::new(cmd).map_err(JlrsError::other)?;
            let cmd_ptr = cmd_cstring.as_ptr();
            let res = jl_eval_string(cmd_ptr);
            try_root(frame, res)
        }
    }

    /// Execute a Julia command `cmd`. This is equivalent to `Value::eval_string`, but uses a
    /// null-terminated string.
    pub fn eval_cstring<'frame, F, S>(
        frame: &mut F,
        cmd: S,
    ) -> JlrsResult<JuliaResult<'frame, 'static>>
    where
        F: Frame<'frame>,
        S: AsRef<CStr>,
    {
        unsafe {
            let cmd = cmd.as_ref();
            let cmd_ptr = cmd.as_ptr();
            let res = jl_eval_string(cmd_ptr);
            try_root(frame, res)
        }
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
    pub unsafe fn add_finalizer(self, f: Value) {
        jl_gc_add_finalizer(self.unwrap(Private), f.unwrap(Private))
    }

    /// Call all finalizers.
    pub unsafe fn finalize(self) {
        jl_finalize(self.unwrap(Private))
    }
}

/// # Constant values.
impl<'base> Value<'base, 'static> {
    /// `Core.Union{}`.
    pub fn bottom_type(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_bottom_type), Private) }
    }

    /// `Core.StackOverflowError`.
    pub fn stackovf_exception(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_stackovf_exception), Private) }
    }

    /// `Core.OutOfMemoryError`.
    pub fn memory_exception(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_memory_exception), Private) }
    }

    /// `Core.ReadOnlyMemoryError`.
    pub fn readonlymemory_exception(_: Global<'base>) -> Self {
        unsafe {
            Value::wrap_non_null(NonNull::new_unchecked(jl_readonlymemory_exception), Private)
        }
    }

    /// `Core.DivideError`.
    pub fn diverror_exception(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_diverror_exception), Private) }
    }

    /// `Core.UndefRefError`.
    pub fn undefref_exception(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_undefref_exception), Private) }
    }

    /// `Core.InterruptException`.
    pub fn interrupt_exception(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_interrupt_exception), Private) }
    }

    /// An empty `Core.Array{Any, 1}.
    pub fn an_empty_vec_any(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_an_empty_vec_any), Private) }
    }

    /// An empty immutable String, "".
    pub fn an_empty_string(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_an_empty_string), Private) }
    }

    /// `Core.Array{UInt8, 1}`
    pub fn array_uint8_type(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_uint8_type), Private) }
    }

    /// `Core.Array{Any, 1}`
    pub fn array_any_type(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_any_type), Private) }
    }

    /// `Core.Array{Symbol, 1}`
    pub fn array_symbol_type(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_symbol_type), Private) }
    }

    /// `Core.Array{Int32, 1}`
    pub fn array_int32_type(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_int32_type), Private) }
    }

    /// The empty tuple, `()`.
    pub fn emptytuple(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_emptytuple), Private) }
    }

    /// The instance of `true`.
    pub fn true_v(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_true), Private) }
    }

    /// The instance of `false`.
    pub fn false_v(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_false), Private) }
    }

    /// The instance of `Core.Nothing`, `nothing`.
    pub fn nothing(_: Global<'base>) -> Self {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_nothing), Private) }
    }
}

impl<'frame, 'data> Debug for Value<'frame, 'data> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Value").field(&self.type_name()).finish()
    }
}

unsafe impl<'frame, 'data> ValidLayout for Value<'frame, 'data> {
    unsafe fn valid_layout(v: Value) -> bool {
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

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Value<'scope, 'data> {
    type Internal = jl_value_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}

unsafe fn try_root<'frame, F>(
    frame: &mut F,
    res: *mut jl_value_t,
) -> JlrsResult<JuliaResult<'frame, 'static>>
where
    F: Frame<'frame>,
{
    let exc = jl_exception_occurred();

    if !exc.is_null() {
        match frame.push_root(NonNull::new_unchecked(exc), Private) {
            Ok(exc) => Ok(Err(exc)),
            Err(a) => Err(a.into()),
        }
    } else {
        if res.is_null() {
            Ok(Ok(Value::nothing(frame.global())))
        } else {
            match frame.push_root(NonNull::new_unchecked(res), Private) {
                Ok(v) => Ok(Ok(v)),
                Err(a) => Err(a.into()),
            }
        }
    }
}

unsafe fn small_dim_tuple<'frame, F>(
    frame: &mut F,
    dims: &Dimensions,
) -> JlrsResult<Value<'frame, 'static>>
where
    F: Frame<'frame>,
{
    let n = dims.n_dimensions();
    debug_assert!(n <= 8, "Too many dimensions for small_dim_tuple");
    let elem_types = JL_LONG_TYPE.with(|longs| longs.get());
    let tuple_type = jl_apply_tuple_type_v(elem_types.cast(), n);
    let tuple = jl_new_struct_uninit(tuple_type);
    let v = frame
        .push_root(NonNull::new_unchecked(tuple), Private)
        .map_err(JlrsError::alloc_error)?;

    let usize_ptr: *mut usize = v.unwrap(Private).cast();
    std::ptr::copy_nonoverlapping(dims.as_slice().as_ptr(), usize_ptr, n);

    Ok(v)
}

unsafe fn large_dim_tuple<'frame, F>(
    frame: &mut F,
    dims: &Dimensions,
) -> JlrsResult<Value<'frame, 'static>>
where
    F: Frame<'frame>,
{
    let n = dims.n_dimensions();
    let global = frame.global();
    let mut elem_types = vec![usize::julia_type(global); n];
    let tuple_type = jl_apply_tuple_type_v(elem_types.as_mut_ptr().cast(), n);
    let tuple = jl_new_struct_uninit(tuple_type);
    let v = frame
        .push_root(NonNull::new_unchecked(tuple), Private)
        .map_err(JlrsError::alloc_error)?;

    let usize_ptr: *mut usize = v.unwrap(Private).cast();
    std::ptr::copy_nonoverlapping(dims.as_slice().as_ptr(), usize_ptr, n);

    Ok(v)
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
    /// method can only be called inside a closure taken by one of the `frame`-methods.
    pub fn as_value<'base>(self, _: Global<'base>) -> Value<'base, 'static> {
        self.0
    }
}
