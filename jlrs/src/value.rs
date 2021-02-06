//! Julia values and functions.
//!
//! When using this crate Julia data will usually be returned as a [`Value`]. A [`Value`] is a
//! "generic" wrapper. Type information will generally be available allowing you to safely convert
//! a [`Value`] to its actual type. Data like arrays and modules can be returned as a [`Value`].
//! These, and other types with a custom implementation in the C API, can be found in the
//! submodules of this module.
//!
//! One special property of a [`Value`] is that it can always be called as a function; there's no
//! way to check if a [`Value`] is actually a function except trying to call it. Multiple
//! [`Value`]s can be created at the same time by using [`Values`].
//!
//! [`Value`]: struct.Value.html
//! [`Values`]: struct.Values.html

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
/// ```no_run
/// # use jlrs::prelude::*;
/// # fn main() {
/// let mut julia = unsafe { Julia::init().unwrap() };
/// // Three slots; two for the inputs and one for the output.
/// julia.frame(3, |global, frame| {
///     // Create the two arguments, each value requires one slot
///     let i = Value::new(&mut *frame, 2u64)?;
///     let j = Value::new(&mut *frame, 1u32)?;
///
///     let _nt = named_tuple!(&mut *frame, "i" => i, "j" => j);
///
///     Ok(())
/// }).unwrap();
/// # }
/// ```
#[macro_export]
macro_rules! named_tuple {
    ($frame:expr, $name:expr => $value:expr) => {
        $crate::value::Value::new_named_tuple($frame, &mut [$name], &mut [$value])
    };
    ($frame:expr, $name:expr => $value:expr, $($rest:tt)+) => {
        {
            let n = $crate::count!($($rest)+);
            let mut names = ::smallvec::SmallVec::<[_; $crate::value::MAX_SIZE]>::with_capacity(n);
            let mut values = ::smallvec::SmallVec::<[_; $crate::value::MAX_SIZE]>::with_capacity(n);

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
            $crate::value::Value::new_named_tuple($frame, $names, $values)
        }
    };
}

pub mod traits;

use self::array::{Array, Dimensions};
use self::datatype::DataType;
use self::module::Module;
use self::symbol::Symbol;
use self::type_var::TypeVar;
use self::union_all::UnionAll;
use crate::{convert::into_julia::IntoJulia, impl_julia_type};
use crate::memory::traits::{
    scope::Scope,
    frame::private::Frame as PNewFrame, frame::Frame,
    scope::private::Scope as PScope,
};
use crate::layout::{valid_layout::ValidLayout, julia_type::JuliaType, julia_typecheck::JuliaTypecheck};
use crate::convert::{cast::Cast, temporary_symbol::TemporarySymbol};
use crate::{
    error::{CallResult, JlrsError, JlrsResult},
};
use crate::memory::{global::Global, output::OutputScope};
use jl_sys::{
    jl_alloc_array_1d, jl_alloc_array_2d, jl_alloc_array_3d, jl_an_empty_string,
    jl_an_empty_vec_any, jl_any_type, jl_apply_array_type, jl_apply_tuple_type_v, jl_apply_type,
    jl_array_any_type, jl_array_int32_type, jl_array_symbol_type, jl_array_uint8_type,
    jl_bottom_type, jl_call, jl_call0, jl_call1, jl_call2, jl_call3, jl_datatype_t,
    jl_diverror_exception, jl_egal, jl_emptytuple, jl_eval_string, jl_exception_occurred, jl_false,
    jl_field_index, jl_field_isptr, jl_field_names, jl_fieldref, jl_fieldref_noalloc, jl_finalize,
    jl_gc_add_finalizer, jl_gc_wb, jl_get_kwsorter, jl_get_nth_field, jl_get_nth_field_noalloc,
    jl_interrupt_exception, jl_is_kind, jl_isa, jl_memory_exception, jl_new_array,
    jl_new_struct_uninit, jl_nfields, jl_nothing, jl_nothing_type, jl_object_id, jl_ptr_to_array,
    jl_ptr_to_array_1d, jl_readonlymemory_exception, jl_set_nth_field, jl_stackovf_exception,
    jl_subtype, jl_svec_data, jl_svec_len, jl_true, jl_type_union, jl_type_unionall, jl_typeof,
    jl_typeof_str, jl_undefref_exception, jl_value_t,
};
use traits::private::Internal;
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::ptr::null_mut;
use std::slice;
#[cfg(all(feature = "async", target_os = "linux"))]
use crate::memory::frame::AsyncGcFrame;

/// In some cases it's necessary to place one or more arguments in front of the arguments a
/// function is called with. Examples include `Value::asynccall` and `WithKeywords::call`. If
/// these functions are called with fewer than `MAX_SIZE` arguments (including the added
/// arguments), no heap allocation is required to store them.
pub const MAX_SIZE: usize = 8;

pub mod array;
pub mod code_instance;
pub mod datatype;
pub mod expr;
pub mod method;
pub mod method_instance;
pub mod method_table;
pub mod module;
pub mod simple_vector;
pub mod string;
pub mod symbol;
pub mod task;
pub mod tuple;
pub mod type_name;
pub mod type_var;
pub mod typemap_entry;
pub mod typemap_level;
pub mod union;
pub mod union_all;
pub mod weak_ref;

thread_local! {
    // Used as a pool to convert dimensions to tuples. Safe because a thread local is initialized
    // when `with` is first called, which happens after `Julia::init` has been called. The C API
    // requires a mutable pointer to this array so an `UnsafeCell` is used to store it.
    static JL_LONG_TYPE: UnsafeCell<[*mut jl_datatype_t; 8]> = unsafe {
        UnsafeCell::new([
            usize::julia_type(),
            usize::julia_type(),
            usize::julia_type(),
            usize::julia_type(),
            usize::julia_type(),
            usize::julia_type(),
            usize::julia_type(),
            usize::julia_type(),
        ])
    };
}

/// When working with the Julia C API most data is returned as a raw pointer to a `jl_value_t`.
/// This pointer is similar to a void pointer in the sense that this pointer can point to data of
/// any type. It's up to the user to determine the correct type and cast the pointer. In order to
/// make this possible, data pointed to by a `jl_value_t`-pointer is guaranteed to be preceded in
/// memory by a fixed-size header that contains its type and layout-information.
///
/// A `Value` is a wrapper around the raw pointer to a `jl_value_t` that adds two lifetimes,
/// `'frame` and `'data`. The first is inherited from the frame used to create the `Value`; frames
/// ensure a `Value` is protected from garbage collection as long as the frame used to protect it
/// has not been dropped. As a result, a `Value` can only be used when it can be guaranteed that
/// the garbage collector won't drop it. The second indicates the lifetime of its contents; it's
/// usually `'static`, but if you create a `Value` that borrows array data from Rust it's the
/// lifetime of the borrow. If you call a Julia function the returned `Value` will inherit the
/// `'data`-lifetime of the `Value`s used as arguments. This ensures that a `Value` that
/// (possibly) borrows data from Rust can't be used after that borrow ends. If this restriction is
/// too strict you can forget the second lifetime by calling [`Value::assume_owned`].
///
/// ### Creating new values
///
/// New `Value`s can be created from Rust in several ways. Types that implement [`IntoJulia`] can
/// be converted to a `Value` by calling [`Value::new`]. This trait is implemented by primitive
/// types like `bool`, `char`, `i16`, and `usize`; string types like `String`, `&str`, and `Cow`;
/// [`tuples`]; and you can derive it for your own types by deriving [`IntoJulia`]. You should
/// use `JlrsReflect.jl` rather than doing this manually.
///
/// [`Value`] also has several methods to create an n-dimensional array if the element type
/// implements [`IntoJulia`], this includes primitive types, strings. It is also implemented for
/// bits types with no type parameters when these bindings are generated with `JlrsReflect.jl`. A
/// new array whose data is completely managed by Julia can be created by calling
/// [`Value::new_array`]. You can also transfer the ownership of some `Vec` to Julia and treat it
/// as an n-dimensional array with [`Value::move_array`]. Finally, you can borrow anything that
/// can be borrowed as a mutable slice with [`Value::borrow_array`].
///
/// Functions and other global values defined in a module can be accessed through that module.
/// Please see the documentation for [`Module`] for more information.
///
/// ### Casting values
///
/// A `Value`'s type information can be accessed by calling [`Value::datatype`], this is usually
/// not necessary to determine what kind of data it contains; you can use [`Value::is`] to query
/// properties of the value's type. You can use [`Value::cast`] to convert the value to the
/// appropriate type. If a type implements both [`JuliaTypecheck`] and [`Cast`], which are used by
/// [`Value::is`] and [`Value::cast`] respectively, the former returning `true` when called with
/// that type as generic parameter indicates that the latter will succeed. For example,
/// `value.is::<u8>()` returning true means `value.cast::<u8>()` will succeed. You can derive
/// these traits for custom structs by deriving [`JuliaStruct`].
///
/// The methods that create a new `Value` come in two varieties: `<method>` and `<method>_output`.
/// The first will use a slot in the current frame to protect the value from garbage collection,
/// while the latter uses a slot in another active frame.
///
/// [`Value::assume_owned`]: struct.Value.html#method.assume_owned
/// [`Value`]: struct.Value.html
/// [`Value::move_array`]: struct.Value.html#method.move_array
/// [`Value::new_array`]: struct.Value.html#method.new_array
/// [`Value::borrow_array`]: struct.Value.html#method.borrow_array
/// [`IntoJulia`]: ../traits/trait.IntoJulia.html
/// [`JuliaType`]: ../traits/trait.JuliaType.html
/// [`Value::new`]: struct.Value.html#method.new
/// [`Value::datatype`]: struct.Value.html#method.datatype
/// [`JuliaStruct`]: ../traits/trait.JuliaStruct.html
/// [`tuples`]: ./tuple/index.html
/// [`Module`]: ./module/struct.Module.html
/// [`Value::datatype`]: struct.Value.html#method.datatype
/// [`Value::is`]: struct.Value.html#method.is
/// [`Value::cast`]: struct.Value.html#method.cast
/// [`JuliaTypecheck`]: ../traits/trait.JuliaTypecheck.html
/// [`Cast`]: ../traits/trait.Cast.html
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Value<'frame, 'data>(
    *mut jl_value_t,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
);

impl<'frame, 'data> Value<'frame, 'data> {
    pub(crate) unsafe fn wrap(ptr: *mut jl_value_t) -> Value<'frame, 'static> {
        Value(ptr, PhantomData, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_value_t {
        self.0
    }
}

/// # Create new `Value`s
impl<'frame, 'data> Value<'frame, 'data> {
    /// Create a new Julia value, any type that implements [`IntoJulia`] can be converted using
    /// this function. The value will be protected from garbage collection inside the frame used
    /// to create it. One free slot on the GC stack is required for this function to succeed,
    /// returns an error if no slot is available.
    ///
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    pub fn new<'scope, V, S, F>(scope: S, value: V) -> JlrsResult<S::Value>
    where
        V: IntoJulia,
        S: Scope<'scope, 'frame, 'static, F>,
        F: Frame<'frame>,
    {
        unsafe { scope.value(value.into_julia(), Internal) }
    }

    /// Create a new instance of a value with `DataType` `ty`, using `values` to set the fields.
    /// This is essentially a more powerful version of [`Value::new`] and can instantiate
    /// arbitrary concrete `DataType`s, at the cost that each of its fields must have already been
    /// allocated as a `Value`. This functions returns an error if the given `DataType` is not
    /// concrete. One free slot on the GC stack is required for this function to succeed, returns
    /// an error if no slot is available.
    pub fn instantiate<'scope, 'value, 'borrow, V, S, F>(
        scope: S,
        ty: DataType,
        values: &mut V,
    ) -> JlrsResult<S::Value>
    where
        V: AsMut<[Value<'value, 'borrow>]>,
        S: Scope<'scope, 'frame, 'borrow, F>,
        F: Frame<'frame>,
    {
        ty.instantiate(scope, values)
    }

    /// Allocates a new n-dimensional array in Julia.
    ///
    /// Creating an an array with 1, 2 or 3 dimensions requires one slot on the GC stack. If you
    /// create an array with more dimensions an extra frame is created with a single slot,
    /// temporarily taking 3 additional slots.
    ///
    /// This function returns an error if there are not enough slots available.
    pub fn new_array<'scope, T, D, S, F>(scope: S, dimensions: D) -> JlrsResult<S::Value>
    where
        T: IntoJulia + JuliaType,
        D: Into<Dimensions>,
        S: Scope<'scope, 'frame, 'static, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let dims = dimensions.into();
            let array_type = jl_apply_array_type(T::julia_type().cast(), dims.n_dimensions());

            match dims.n_dimensions() {
                1 => scope.value(
                    jl_alloc_array_1d(array_type, dims.n_elements(0)).cast(),
                    Internal,
                ),
                2 => scope.value(
                    jl_alloc_array_2d(array_type, dims.n_elements(0), dims.n_elements(1)).cast(),
                    Internal,
                ),
                3 => scope.value(
                    jl_alloc_array_3d(
                        array_type,
                        dims.n_elements(0),
                        dims.n_elements(1),
                        dims.n_elements(2),
                    )
                    .cast(),
                    Internal,
                ),
                n if n <= 8 => scope.value_frame_with_slots(1, |output, frame| {
                    let tuple = small_dim_tuple(frame, &dims)?;
                    output
                        .into_scope(frame)
                        .value(jl_new_array(array_type, tuple.ptr()).cast(), Internal)
                }),
                _ => scope.value_frame_with_slots(1, |output, frame| {
                    let tuple = large_dim_tuple(frame, &dims)?;
                    output
                        .into_scope(frame)
                        .value(jl_new_array(array_type, tuple.ptr()).cast(), Internal)
                }),
            }
        }
    }

    /// Borrows an n-dimensional array from Rust for use in Julia.
    ///
    /// Borrowing an array with one dimension requires one slot on the GC stack. If you borrow an
    /// array with more dimensions, an extra frame is created with a single slot slot, temporarily
    /// taking 3 additional slots.
    ///
    /// This function returns an error if there are not enough slots available.
    pub fn borrow_array<'scope, T, D, V, S, F>(
        scope: S,
        data: &'data mut V,
        dimensions: D,
    ) -> JlrsResult<S::Value>
    where
        T: IntoJulia + JuliaType,
        D: Into<Dimensions>,
        V: AsMut<[T]>,
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let dims = dimensions.into();
            let array_type = jl_apply_array_type(T::julia_type().cast(), dims.n_dimensions());

            match dims.n_dimensions() {
                1 => scope.value(
                    jl_ptr_to_array_1d(
                        array_type,
                        data.as_mut().as_mut_ptr().cast(),
                        dims.n_elements(0),
                        0,
                    )
                    .cast(),
                    Internal,
                ),
                n if n <= 8 => scope.value_frame_with_slots(1, |output, frame| {
                    let tuple = small_dim_tuple(frame, &dims)?;
                    output.into_scope(frame).value(
                        jl_ptr_to_array(
                            array_type,
                            data.as_mut().as_mut_ptr().cast(),
                            tuple.ptr(),
                            0,
                        )
                        .cast(),
                        Internal,
                    )
                }),
                _ => scope.value_frame_with_slots(1, |output, frame| {
                    let tuple = large_dim_tuple(frame, &dims)?;
                    output.into_scope(frame).value(
                        jl_ptr_to_array(
                            array_type,
                            data.as_mut().as_mut_ptr().cast(),
                            tuple.ptr(),
                            0,
                        )
                        .cast(),
                        Internal,
                    )
                }),
            }
        }
    }

    /// Moves an n-dimensional array from Rust to Julia.
    ///
    /// Moving an array with one dimension requires one slot on the GC stack. If you move an array
    /// with more dimensions, an extra frame is created with a single slot slot, temporarily
    /// taking 3 additional slots.
    ///
    /// This function returns an error if there are not enough slots available.
    pub fn move_array<'scope, T, D, S, F>(
        scope: S,
        data: Vec<T>,
        dimensions: D,
    ) -> JlrsResult<S::Value>
    where
        T: IntoJulia + JuliaType,
        D: Into<Dimensions>,
        S: Scope<'scope, 'frame, 'static, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let dims = dimensions.into();
            let global = scope.global();
            let finalizer = Module::main(global).submodule("Jlrs")?.function("clean")?;

            scope.value_frame_with_slots(2, |output, frame| {
                let array_type = jl_apply_array_type(T::julia_type().cast(), dims.n_dimensions());
                let _ = frame
                    .push_root(array_type, Internal)
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

                        jl_gc_add_finalizer(array, finalizer.ptr());
                        output.into_scope(frame).value(array, Internal)
                    }
                    n if n <= 8 => {
                        let tuple = small_dim_tuple(frame, &dims)?;
                        let array = jl_ptr_to_array(
                            array_type,
                            Box::into_raw(data.into_boxed_slice()).cast(),
                            tuple.ptr(),
                            0,
                        )
                        .cast();

                        jl_gc_add_finalizer(array, finalizer.ptr());
                        output.into_scope(frame).value(array, Internal)
                    }
                    _ => {
                        let tuple = large_dim_tuple(frame, &dims)?;
                        let array = jl_ptr_to_array(
                            array_type,
                            Box::into_raw(data.into_boxed_slice()).cast(),
                            tuple.ptr(),
                            0,
                        )
                        .cast();

                        jl_gc_add_finalizer(array, finalizer.ptr());
                        output.into_scope(frame).value(array, Internal)
                    }
                }
            })
        }
    }

    /// Returns the union of all types in `types`. For each of these types, [`Value::is_kind`]
    /// must return `true`. TNote that the result is not necessarily a [`Union`], for example the
    /// union of a single [`DataType`] is that type, not a `Union` with a single variant. One free
    /// slot on the GC stack is required for this function to succeed, returns an error if no slot is available.
    ///
    /// [`Value::is_kind`]: struct.Value.html#method.is_kind
    /// [`Union`]: union/struct.Union.html
    /// [`DataType`]: datatype/struct.DataType.html
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
                Err(JlrsError::NotAKind(v.type_name().into()))?;
            }

            let un = jl_type_union(types.as_mut_ptr().cast(), types.len());
            scope.value(un, Internal)
        }
    }

    /// Create a new `UnionAll`. One free slot on the GC stack is required for this function to
    /// succeed, returns an error if no slot is available.
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
            Err(JlrsError::InvalidBody(body.type_name().into()))?;
        }

        unsafe {
            let ua = jl_type_unionall(tvar.ptr(), body.ptr());
            scope.value(ua, Internal)
        }
    }

    /// Create a new named tuple, you can use the `named_tuple` macro instead of this method.
    pub fn new_named_tuple<'scope, 'value, S, F, N, T, V>(
        scope: S,
        field_names: &mut N,
        values: &mut V,
    ) -> JlrsResult<S::Value>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
        N: AsMut<[T]>,
        T: TemporarySymbol,
        V: AsMut<[Value<'value, 'data>]>,
    {
        scope.value_frame_with_slots(4, |output, frame| unsafe {
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
                .map(|name| name.temporary_symbol(Internal).as_value())
                .collect::<Vec<_>>();

            let names = DataType::anytuple_type(global)
                .as_value()
                .apply_type(&mut *frame, &mut symbol_type_vec)?
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut field_names_vec)?;

            let mut field_types_vec = values_m
                .iter()
                .copied()
                .map(|val| {
                    val.datatype()
                        .unwrap_or(DataType::nothing_type(global))
                        .as_value()
                })
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

    /// Apply the given types to `self`.
    ///
    /// If `self` is the [`DataType`] `anytuple_type`, calling this function will return a new
    /// tuple type with the given types as its field types. If it is the [`DataType`]
    /// `uniontype_type`, calling this function is equivalent to calling [`Value::new_union`]. If
    /// the value is a `UnionAll`, the given types will be applied and the resulting type is
    /// returned.
    ///
    /// If the types cannot be applied to `self` your program will abort.
    ///
    /// One free slot on the GC stack is required for this function to succeed, returns an error
    /// if no slot is available.
    pub fn apply_type<'scope, 'fr, 'value, 'borrow, S, F, V>(
        self,
        scope: S,
        types: &mut V,
    ) -> JlrsResult<S::Value>
    where
        S: Scope<'scope, 'fr, 'borrow, F>,
        F: Frame<'fr>,
        V: AsMut<[Value<'value, 'borrow>]>,
    {
        unsafe {
            let types = types.as_mut();
            let applied = jl_apply_type(self.ptr(), types.as_mut_ptr().cast(), types.len());
            scope.value(applied, Internal)
        }
    }
}

/// # Properties
impl<'frame, 'data> Value<'frame, 'data> {
    /// Returns the `DataType` of this value, or `None` if the value is a null pointer.
    pub fn datatype(self) -> Option<DataType<'frame>> {
        unsafe {
            if self.is_null() {
                return None;
            }

            Some(DataType::wrap(jl_typeof(self.ptr()).cast()))
        }
    }

    /// Returns the type name of this value.
    pub fn type_name(self) -> &'frame str {
        unsafe {
            if self.ptr().is_null() {
                return "null";
            }
            let type_name = jl_typeof_str(self.ptr());
            let type_name_ref = CStr::from_ptr(type_name);
            type_name_ref.to_str().unwrap()
        }
    }

    /// Returns the object id of this value.
    pub fn object_id(self) -> usize {
        unsafe { jl_object_id(self.ptr()) }
    }

    pub fn isa(self, other: Value) -> bool {
        unsafe { jl_isa(self.ptr(), other.ptr()) != 0 }
    }
}

/// # Type checking
impl<'frame, 'data> Value<'frame, 'data> {
    /// Returns true if the value is `nothing`. Note that the Julia C API often returns a null
    /// pointer instead of `nothing`, this method return false if the given value is a null
    /// pointer.
    pub fn is_nothing(self) -> bool {
        unsafe { !self.is_null() && jl_typeof(self.ptr()) == jl_nothing_type.cast() }
    }

    /// Returns true if the value is a null pointer.
    pub fn is_null(self) -> bool {
        unsafe { self.ptr() == null_mut() }
    }

    /// Performs the given type check. For types that represent Julia data, this check comes down
    /// to checking if the data has that type. This works for primitive types, for example:
    ///
    /// ```no_run
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = unsafe { Julia::init().unwrap() };
    /// julia.frame(1, |_global, frame| {
    ///     let i = Value::new(frame, 2u64)?;
    ///     assert!(i.is::<u64>());
    ///     Ok(())
    /// }).unwrap();
    /// # }
    /// ```
    ///
    /// "Special" types in Julia that are defined in C, like [`Array`], [`Module`] and
    /// [`DataType`], are also supported:
    ///
    /// ```no_run
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = unsafe { Julia::init().unwrap() };
    /// julia.frame(1, |_global, frame| {
    ///     let arr = Value::new_array::<f64, _, _, _>(&mut *frame, (3, 3))?;
    ///     assert!(arr.is::<Array>());
    ///     Ok(())
    /// }).unwrap();
    /// # }
    /// ```
    ///
    /// If you derive [`JuliaStruct`] for some type, that type will be supported by this method. A
    /// full list of supported checks can be found [here].
    ///
    /// [`Array`]: array/struct.Array.html
    /// [`DataType`]: datatype/struct.DataType.html
    /// [`Module`]: module/struct.Module.html
    /// [`Symbol`]: symbol/struct.Symbol.html
    /// [`JuliaStruct`]: ../traits/trait.JuliaStruct.html
    /// [here]: ../traits/trait.JuliaTypecheck.html#implementors
    pub fn is<T: JuliaTypecheck>(self) -> bool {
        if self.is_nothing() {
            return false;
        }

        self.datatype().unwrap().is::<T>()
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
        unsafe { jl_subtype(self.ptr(), sup.ptr()) != 0 }
    }

    /// Returns true if `self` is the type of a `DataType`, `UnionAll`, `Union`, or `Union{}` (the
    /// bottom type).
    pub fn is_kind(self) -> bool {
        unsafe { jl_is_kind(self.ptr()) }
    }

    /// Returns true if the value is a type, ie a `DataType`, `UnionAll`, `Union`, or `Union{}`
    /// (the bottom type).
    pub fn is_type(self) -> bool {
        if let Some(dt) = self.datatype() {
            Value::is_kind(dt.into())
        } else {
            false
        }
    }
}

/// # Lifetime management
impl<'frame, 'data> Value<'frame, 'data> {
    /// If you call a function with one or more borrowed arrays as arguments, its result can only
    /// be used when all the borrows are active. If this result doesn't reference any borrowed
    /// data this function can be used to relax its second lifetime to `'static`.
    ///
    /// Safety: The value must not contain a reference any borrowed data.
    pub unsafe fn assume_owned(self) -> Value<'frame, 'static> {
        Value::wrap(self.ptr())
    }

    pub fn as_unrooted<'scope, 'borrow, F: Frame<'frame>>(
        self,
        _output: OutputScope<'scope, 'frame, 'borrow, F>,
    ) -> UnrootedValue<'scope, 'data, 'borrow> {
        unsafe { UnrootedValue::new(self.ptr()) }
    }
}

/// # Casting to Rust
impl<'frame, 'data> Value<'frame, 'data> {
    /// Cast the contents of this value into a compatible Rust type. Any type which implements
    /// `Cast` can be used as a target, by default this includes primitive types like `u8`, `f32`
    /// and `bool`, and builtin types like [`Array`], [`JuliaString`] and [`Symbol`]. You can
    /// implement this trait for custom types by deriving [`JuliaStruct`].
    ///
    /// [`Array`]: array/struct.Array.html
    /// [`JuliaString`]: string/struct.JuliaString.html
    /// [`Symbol`]: symbol/struct.Symbol.html
    /// [`JuliaStruct`]: ../traits/trait.JuliaStruct.html
    pub fn cast<T: Cast<'frame, 'data>>(self) -> JlrsResult<<T as Cast<'frame, 'data>>::Output> {
        T::cast(self)
    }

    /// Cast the contents of this value into a compatible Rust type without checking if the layout is valid.
    ///
    /// Safety:
    ///
    /// You must guarantee `self.is::<T>()` would have returned `true`.
    pub unsafe fn cast_unchecked<T: Cast<'frame, 'data>>(
        self,
    ) -> <T as Cast<'frame, 'data>>::Output {
        T::cast_unchecked(self)
    }
}

/// # Fields
impl<'frame, 'data> Value<'frame, 'data> {
    /// Returns the field names of this value as a slice of `Symbol`s. These symbols can be used
    /// to access their fields with [`Value::get_field`].
    ///
    /// [`Value::get_field`]: struct.Value.html#method.get_field
    pub fn field_names(self) -> &'frame [Symbol<'frame>] {
        if self.is_nothing() {
            return &[];
        }

        unsafe {
            let tp = jl_typeof(self.ptr());
            let field_names = jl_field_names(tp.cast());
            let len = jl_svec_len(field_names);
            let items: *mut Symbol = jl_svec_data(field_names).cast();
            slice::from_raw_parts(items.cast(), len)
        }
    }

    /// Returns the number of fields the underlying Julia value has. These fields can be accessed
    /// with [`Value::get_field_n`].
    ///
    /// [`Value::get_field_n`]: struct.Value.html#method.get_field_n
    pub fn n_fields(self) -> usize {
        if self.is_nothing() {
            return 0;
        }

        unsafe { jl_nfields(self.ptr()) as _ }
    }

    /// Returns the field at index `idx` if it exists. If it does not exist
    /// `JlrsError::OutOfBounds` is returned. This function assumes the field must be protected
    /// from garbage collection, so calling this function will take a single slot on the GC stack.
    /// If there is no slot available `JlrsError::AllocError` is returned.
    pub fn get_nth_field<'scope, 'fr, S, F>(self, scope: S, idx: usize) -> JlrsResult<S::Value>
    where
        S: Scope<'scope, 'fr, 'data, F>,
        F: Frame<'fr>,
    {
        unsafe {
            if idx >= self.n_fields() {
                return Err(JlrsError::OutOfBounds(idx, self.n_fields()).into());
            }

            scope.value(jl_fieldref(self.ptr(), idx), Internal)
        }
    }

    /// Returns the field at index `idx` if it exists and no allocation is required to return it.
    /// Allocation is not required if the field is a pointer to another value.
    ///
    /// If the field does not exist `JlrsError::NoSuchField` is returned. If allocating is
    /// required to return the field, `JlrsError::NotAPointerField` is returned.
    ///
    /// This function is unsafe because the value returned as a result will only be valid as long
    /// as the field is not changed.
    pub unsafe fn get_nth_field_noalloc(self, idx: usize) -> JlrsResult<Value<'frame, 'data>> {
        if self.is_nothing() {
            Err(JlrsError::Nothing)?;
        }

        if idx >= self.n_fields() {
            Err(JlrsError::OutOfBounds(idx, self.n_fields()))?
        }

        if !jl_field_isptr(self.datatype().unwrap().ptr(), idx as _) {
            Err(JlrsError::NotAPointerField(idx))?;
        }

        Ok(Value::wrap(jl_fieldref_noalloc(self.ptr(), idx)))
    }

    /// Returns the field with the name `field_name` if it exists. If it does not exist
    /// `JlrsError::NoSuchField` is returned. This function assumes the field must be protected
    /// from garbage collection, so calling this function will take a single slot on the GC stack.
    /// If there is no slot available `JlrsError::AllocError` is returned.
    pub fn get_field<'scope, 'fr, N, S, F>(self, scope: S, field_name: N) -> JlrsResult<S::Value>
    where
        N: TemporarySymbol,
        S: Scope<'scope, 'fr, 'data, F>,
        F: Frame<'fr>,
    {
        unsafe {
            let symbol = field_name.temporary_symbol(Internal);

            if self.is_nothing() {
                Err(JlrsError::Nothing)?;
            }

            let jl_type = jl_typeof(self.ptr()).cast();
            let idx = jl_field_index(jl_type, symbol.ptr(), 0);

            if idx < 0 {
                return Err(JlrsError::NoSuchField(symbol.into()).into());
            }

            scope.value(jl_get_nth_field(self.ptr(), idx as _), Internal)
        }
    }

    /// Returns the field with the name `field_name` if it exists and no allocation is required
    /// to return it. Allocation is not required if the field is a pointer to another value.
    ///
    /// If the field does not exist `JlrsError::NoSuchField` is returned. If allocating is
    /// required to return the field, `JlrsError::NotAPointerField` is returned.
    ///
    /// This function is unsafe because the value returned as a result will only be valid as long
    /// as the field is not changed.
    pub unsafe fn get_field_noalloc<N>(self, field_name: N) -> JlrsResult<Value<'frame, 'data>>
    where
        N: TemporarySymbol,
    {
        let symbol = field_name.temporary_symbol(Internal);

        if self.is_nothing() {
            Err(JlrsError::Nothing)?;
        }

        let jl_type = jl_typeof(self.ptr()).cast();
        let idx = jl_field_index(jl_type, symbol.ptr(), 0);

        if idx < 0 {
            return Err(JlrsError::NoSuchField(symbol.into()).into());
        }

        if !jl_field_isptr(self.datatype().unwrap().ptr(), idx) {
            Err(JlrsError::NotAPointerField(idx as _))?;
        }

        Ok(Value::wrap(jl_get_nth_field_noalloc(self.ptr(), idx as _)))
    }

    /// Set the value of the field at `idx`. Returns an error if this value is immutable or if the
    /// type of `value` is not a subtype of the field type. This is unsafe because the previous
    /// value of this field can become unrooted if you're directly using it from Rust.
    pub unsafe fn set_nth_field(self, idx: usize, value: Value) -> JlrsResult<()> {
        if !self.is::<datatype::Mutable>() {
            Err(JlrsError::Immutable)?
        }

        let field_type = self.datatype().unwrap().field_types()[idx];
        if let Some(dt) = value.datatype() {
            if Value::subtype(dt.into(), field_type) {
                jl_set_nth_field(self.ptr(), idx, value.ptr());
                jl_gc_wb(self.ptr(), value.ptr());
                return Ok(());
            } else {
                Err(JlrsError::NotSubtype)?
            }
        }

        Err(JlrsError::Nothing)?
    }
}

/// # Call Julia.
///
/// Several methods are available to call Julia. Raw commands can be executed with `eval_string`
/// and `eval_cstring`, but these can't take any arguments. In order to call functions that take
/// arguments, you must use one of the `call` methods which will call that value as a function
/// with any number of arguments. One of these, `call_keywords`, lets you call functions with
/// keyword arguments.
impl<'fr, 'data> Value<'fr, 'data> {
    /// Provide keywords to this function.
    ///
    /// Functions that can take keyword arguments can be called in two major ways, either with or
    /// without keyword arguments. The normal call-methods take care of the frst case, this one
    /// takes care of the second.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(4, |global, frame| {
    ///       let a_value = Value::new(&mut *frame, 1isize)?;
    ///       let b_value = Value::new(&mut *frame, 10isize)?;
    ///       // `funcwithkw` takes a single positional argument of type `Int`, one keyword
    ///       // argument named `b` of the same type, and returns `a` + `b`.
    ///       let func = Module::main(global)
    ///           .submodule("JlrsTests")?
    ///           .function("funcwithkw")?;
    ///
    ///       let kw = named_tuple!(&mut *frame, "b" => b_value)?;
    ///       let res = func.with_keywords(kw)
    ///           .call1(&mut *frame, a_value)?
    ///           .unwrap()
    ///           .cast::<isize>()?;
    ///  
    ///       assert_eq!(res, 11);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    pub fn with_keywords<'kws>(
        self,
        keywords: Value<'kws, 'data>,
    ) -> WithKeywords<'fr, 'kws, 'data> {
        WithKeywords {
            func: self,
            kws: keywords,
        }
    }

    /// Execute a Julia command `cmd`, for example
    ///
    /// `Value::eval_string(frame, "sqrt(2)")`.
    pub fn eval_string<'frame, F, S>(
        frame: &mut F,
        cmd: S,
    ) -> JlrsResult<CallResult<'frame, 'static>>
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
    ) -> JlrsResult<CallResult<'frame, 'static>>
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

    /// Call this value as a function that takes zero arguments and don't protect the result from
    /// garbage collection. This is safe if you won't use the result or if you can guarantee it's
    /// a global value in Julia, e.g. `nothing` or a [`Module`].
    pub unsafe fn call0_unprotected<'base>(self, _: Global<'base>) -> CallResult<'base, 'static> {
        let res = jl_call0(self.ptr());
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
    }

    /// Call this value as a function that takes one argument and don't protect the result from
    /// garbage collection. This is safe if you won't use the result or if you can guarantee it's
    /// a global value in Julia, e.g. `nothing` or a [`Module`].
    pub unsafe fn call1_unprotected<'base, 'borrow>(
        self,
        _: Global<'base>,
        arg: Value<'_, 'borrow>,
    ) -> CallResult<'base, 'borrow> {
        let res = jl_call1(self.ptr().cast(), arg.ptr());
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
    }

    /// Call this value as a function that takes two arguments and don't protect the result from
    /// garbage collection. This is safe if you won't use the result or if you can guarantee it's
    /// a global value in Julia, e.g. `nothing` or a [`Module`].
    pub unsafe fn call2_unprotected<'base, 'borrow>(
        self,
        _: Global<'base>,
        arg0: Value<'_, 'borrow>,
        arg1: Value<'_, 'borrow>,
    ) -> CallResult<'base, 'borrow> {
        let res = jl_call2(self.ptr().cast(), arg0.ptr(), arg1.ptr());
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
    }

    /// Call this value as a function that takes three arguments and don't protect the result from
    /// garbage collection. This is safe if you won't use the result or if you can guarantee it's
    /// a global value in Julia, e.g. `nothing` or a [`Module`].
    pub unsafe fn call3_unprotected<'base, 'borrow>(
        self,
        _: Global<'base>,
        arg0: Value<'_, 'borrow>,
        arg1: Value<'_, 'borrow>,
        arg2: Value<'_, 'borrow>,
    ) -> CallResult<'base, 'borrow> {
        let res = jl_call3(self.ptr().cast(), arg0.ptr(), arg1.ptr(), arg2.ptr());
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
    }

    /// Call this value as a function that takes several arguments and don't protect the result
    /// from garbage collection. This is safe if you won't use the result or if you can guarantee
    /// it's a global value in Julia, e.g. `nothing` or a [`Module`].
    pub unsafe fn call_unprotected<'base, 'value, 'borrow, V, F>(
        self,
        _: Global<'base>,
        args: &mut V,
    ) -> CallResult<'base, 'borrow>
    where
        V: AsMut<[Value<'value, 'borrow>]>,
    {
        let args = args.as_mut();
        let n = args.len();
        let res = jl_call(self.ptr().cast(), args.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
    }

    /// Call this value as a function that takes keyword arguments, any number of positional
    /// arguments and don't protect the result from garbage collection. This is safe if you won't
    /// use the result or if you can guarantee it's a global value in Julia, e.g. `nothing` or a
    /// [`Module`].
    pub unsafe fn call_keywords_unprotected<'base, 'value, 'borrow, V, F>(
        self,
        _: Global<'base>,
        args: &mut V,
    ) -> CallResult<'base, 'borrow>
    where
        V: AsMut<[Value<'value, 'borrow>]>,
    {
        let func = jl_get_kwsorter(self.datatype().expect("").ptr().cast());
        let args = args.as_mut();
        let n = args.len();

        let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
    }

    /// Call this value as a function that takes several arguments and execute it on another
    /// thread in Julia created with `Base.@spawn`, this takes two slots on the GC stack. Returns
    /// the result of this function call if no exception is thrown, the exception if one is, or an
    /// error if no space is left on the stack.
    ///
    /// This function can only be called with an `AsyncDynamicFrame`, while you're waiting for this
    /// function to complete, other tasks are able to progress.
    #[cfg(all(feature = "async", target_os = "linux"))]
    pub async fn call_async<'frame, 'value, 'borrow, V>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
        args: &mut V,
    ) -> JlrsResult<CallResult<'frame, 'borrow>>
    where
        V: AsMut<[Value<'value, 'borrow>]>,
    {
        unsafe { Ok(crate::multitask::julia_future::JuliaFuture::new(frame, self, args)?.await) }
    }

    /// Returns an anonymous function that wraps this value in a try-catch block. Calling this
    /// anonymous function with some arguments will call the value as a function with those
    /// arguments and return its result, or catch the exception, print the stackstrace, and
    /// rethrow that exception. This takes one slot on the GC stack.
    pub fn tracing_call<'frame, F>(self, frame: &mut F) -> JlrsResult<CallResult<'frame, 'data>>
    where
        F: Frame<'frame>,
    {
        unsafe {
            let global = frame.global();
            let func = Module::main(global)
                .submodule("Jlrs")?
                .function("tracingcall")?;
            let res = jl_call1(func.ptr(), self.ptr());
            try_root(frame, res)
        }
    }

    /// Returns an anonymous function that wraps this value in a try-catch block. Calling this
    /// anonymous function with some arguments will call the value as a function with those
    /// arguments and return its result, or catch the exception and throw a new one with two
    /// fields, `exc` and `stacktrace`, containing the original exception and the stacktrace
    /// respectively. This takes one slot on the GC stack.
    pub fn attach_stacktrace<'frame, F>(
        self,
        frame: &mut F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        F: Frame<'frame>,
    {
        unsafe {
            let global = frame.global();
            let func = Module::main(global)
                .submodule("Jlrs")?
                .function("attachstacktrace")?;
            let res = jl_call1(func.ptr(), self.ptr());
            try_root(frame, res)
        }
    }
}

/// # Equality
impl Value<'_, '_> {
    /// Returns true if `self` and `other` are equal.
    pub fn egal(self, other: Value) -> bool {
        unsafe { jl_egal(self.ptr(), other.ptr()) != 0 }
    }
}

/// # Finalization
impl Value<'_, '_> {
    /// Add a finalizer `f` to this value. The finalizer must be a Julia function, it will be
    /// called when this value is about to be freed by the garbage collector.
    pub unsafe fn add_finalizer(self, f: Value) {
        jl_gc_add_finalizer(self.ptr(), f.ptr())
    }

    /// Call all finalizers.
    pub unsafe fn finalize(self) {
        jl_finalize(self.ptr())
    }
}

/// Constant values.
impl<'base> Value<'base, 'static> {
    /// `Core.Union{}`.
    pub fn bottom_type(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_bottom_type) }
    }

    /// `Core.StackOverflowError`.
    pub fn stackovf_exception(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_stackovf_exception) }
    }

    /// `Core.OutOfMemoryError`.
    pub fn memory_exception(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_memory_exception) }
    }

    /// `Core.ReadOnlyMemoryError`.
    pub fn readonlymemory_exception(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_readonlymemory_exception) }
    }

    /// `Core.DivideError`.
    pub fn diverror_exception(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_diverror_exception) }
    }

    /// `Core.UndefRefError`.
    pub fn undefref_exception(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_undefref_exception) }
    }

    /// `Core.InterruptException`.
    pub fn interrupt_exception(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_interrupt_exception) }
    }

    /// An empty `Core.Array{Any, 1}.
    ///
    /// Safety: never mutate this vec.
    pub unsafe fn an_empty_vec_any(_: Global<'base>) -> Self {
        Value::wrap(jl_an_empty_vec_any)
    }

    /// An empty immutable String, "".
    pub fn an_empty_string(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_an_empty_string) }
    }

    /// `Core.Array{UInt8, 1}`
    pub fn array_uint8_type(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_array_uint8_type) }
    }

    /// `Core.Array{Any, 1}`
    pub fn array_any_type(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_array_any_type) }
    }

    /// `Core.Array{Symbol, 1}`
    pub fn array_symbol_type(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_array_symbol_type) }
    }

    /// `Core.Array{Int32, 1}`
    pub fn array_int32_type(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_array_int32_type) }
    }

    /// The empty tuple, `()`.
    pub fn emptytuple(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_emptytuple) }
    }

    /// The instance of `true`.
    pub fn true_v(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_true) }
    }

    /// The instance of `false`.
    pub fn false_v(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_false) }
    }

    /// The instance of `Core.Nothing`, `nothing`.
    pub fn nothing(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_nothing) }
    }
}

impl<'frame, 'data> Debug for Value<'frame, 'data> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Value").field(&self.type_name()).finish()
    }
}

impl_julia_type!(Value<'frame, 'data>, jl_any_type, 'frame, 'data);

unsafe impl<'frame, 'data> ValidLayout for Value<'frame, 'data> {
    unsafe fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            !dt.isinlinealloc()
        } else if v.cast::<union_all::UnionAll>().is_ok() {
            true
        } else if let Ok(u) = v.cast::<union::Union>() {
            !u.isbitsunion()
        } else {
            false
        }
    }
}

/// A function with keyword arguments
pub struct WithKeywords<'func, 'kw, 'data> {
    pub(crate) func: Value<'func, 'data>,
    pub(crate) kws: Value<'kw, 'data>,
}

unsafe fn try_root<'frame, F>(
    frame: &mut F,
    res: *mut jl_value_t,
) -> JlrsResult<CallResult<'frame, 'static>>
where
    F: Frame<'frame>,
{
    let exc = jl_exception_occurred();

    if !exc.is_null() {
        match frame.push_root(exc, Internal) {
            Ok(exc) => Ok(Err(exc)),
            Err(a) => Err(a.into()),
        }
    } else {
        match frame.push_root(res, Internal) {
            Ok(v) => Ok(Ok(v)),
            Err(a) => Err(a.into()),
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
    assert!(n <= 8);
    let elem_types = JL_LONG_TYPE.with(|longs| longs.get());
    let tuple_type = jl_apply_tuple_type_v(elem_types.cast(), n);
    let tuple = jl_new_struct_uninit(tuple_type);
    let v = frame
        .push_root(tuple, Internal)
        .map_err(JlrsError::alloc_error)?;

    let usize_ptr: *mut usize = v.ptr().cast();
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
    let mut elem_types = vec![usize::julia_type(); n];
    let tuple_type = jl_apply_tuple_type_v(elem_types.as_mut_ptr().cast(), n);
    let tuple = jl_new_struct_uninit(tuple_type);
    let v = frame
        .push_root(tuple, Internal)
        .map_err(JlrsError::alloc_error)?;

    let usize_ptr: *mut usize = v.ptr().cast();
    std::ptr::copy_nonoverlapping(dims.as_slice().as_ptr(), usize_ptr, n);

    Ok(v)
}

#[repr(transparent)]
pub(crate) struct PendingValue<'frame, 'data>(
    *mut jl_value_t,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
);

impl<'frame, 'data> PendingValue<'frame, 'data> {
    pub(crate) fn inner(self) -> *mut jl_value_t {
        self.0
    }

    pub(crate) fn new(contents: *mut jl_value_t) -> Self {
        PendingValue(contents, PhantomData, PhantomData)
    }
}

/// A `Value` that has not yet been rooted.
#[repr(transparent)]
pub struct UnrootedValue<'frame, 'data, 'borrow>(
    pub(crate) *mut jl_value_t,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
    PhantomData<&'borrow ()>,
);

impl<'frame, 'data, 'borrow> UnrootedValue<'frame, 'data, 'borrow> {
    pub(crate) fn into_pending(self) -> PendingValue<'frame, 'data> {
        PendingValue::new(self.0)
    }

    pub(crate) fn ptr(self) -> *mut jl_value_t {
        self.0
    }

    pub(crate) fn new(contents: *mut jl_value_t) -> Self {
        UnrootedValue(contents, PhantomData, PhantomData, PhantomData)
    }
}

pub(crate) type PendingCallResult<'frame, 'data> =
    Result<PendingValue<'frame, 'data>, PendingValue<'frame, 'data>>;

/// A `CallResult` that has not yet been rooted.
pub enum UnrootedCallResult<'frame, 'data, 'inner> {
    Ok(UnrootedValue<'frame, 'data, 'inner>),
    Err(UnrootedValue<'frame, 'data, 'inner>),
}

impl<'frame, 'data, 'inner> UnrootedCallResult<'frame, 'data, 'inner> {
    pub(crate) fn into_pending(self) -> PendingCallResult<'frame, 'data> {
        match self {
            Self::Ok(pov) => Ok(pov.into_pending()),
            Self::Err(pov) => Err(pov.into_pending()),
        }
    }

    #[cfg(all(feature = "async", target_os = "linux"))]
    pub(crate) fn is_exception(&self) -> bool {
        match self {
            Self::Ok(_) => true,
            Self::Err(_) => false,
        }
    }

    #[cfg(all(feature = "async", target_os = "linux"))]
    pub(crate) fn ptr(self) -> *mut jl_value_t {
        match self {
            Self::Ok(pov) => pov.ptr(),
            Self::Err(pov) => pov.ptr(),
        }
    }
}
