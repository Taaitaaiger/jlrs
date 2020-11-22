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

use self::array::{Array, Dimensions};
use self::datatype::{Concrete, DataType};
use self::module::Module;
use self::symbol::Symbol;
use self::type_var::TypeVar;
use self::union_all::UnionAll;
use crate::error::{JlrsError, JlrsResult};
use crate::frame::Output;
use crate::global::Global;
use crate::impl_julia_type;
use crate::traits::{
    private::Internal, valid_layout::ValidLayout, Cast, Frame, IntoJulia, JuliaType,
    JuliaTypecheck, TemporarySymbol,
};
use jl_sys::{
    jl_alloc_array_1d, jl_alloc_array_2d, jl_alloc_array_3d, jl_an_empty_string,
    jl_an_empty_vec_any, jl_any_type, jl_apply_array_type, jl_apply_tuple_type_v, jl_apply_type,
    jl_array_any_type, jl_array_int32_type, jl_array_symbol_type, jl_array_uint8_type,
    jl_bottom_type, jl_call, jl_call0, jl_call1, jl_call2, jl_call3, jl_datatype_t,
    jl_diverror_exception, jl_egal, jl_emptytuple, jl_eval_string, jl_exception_occurred, jl_false,
    jl_field_index, jl_field_isptr, jl_field_names, jl_fieldref, jl_fieldref_noalloc, jl_finalize,
    jl_gc_add_finalizer, jl_get_kwsorter, jl_get_nth_field, jl_get_nth_field_noalloc,
    jl_interrupt_exception, jl_is_kind, jl_memory_exception, jl_new_array, jl_new_struct_uninit,
    jl_new_structv, jl_nfields, jl_nothing, jl_object_id, jl_ptr_to_array, jl_ptr_to_array_1d,
    jl_readonlymemory_exception, jl_set_nth_field, jl_stackovf_exception, jl_subtype, jl_svec_data,
    jl_svec_len, jl_true, jl_type_union, jl_type_unionall, jl_typeof, jl_typeof_str,
    jl_undefref_exception, jl_value_t,
};
use std::borrow::BorrowMut;
use std::ffi::{CStr, CString};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::ptr::null_mut;
use std::slice;

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
    static JL_LONG_TYPE: std::cell::UnsafeCell<[*mut jl_datatype_t; 8]> = unsafe {
        std::cell::UnsafeCell::new([
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

/// This type alias is used to encode the result of a function call: `Ok` indicates the call was
/// successful and contains the function's result, while `Err` indicates an exception was thrown
/// and contains said exception.
pub type CallResult<'frame, 'data, V = Value<'frame, 'data>> = Result<V, Value<'frame, 'data>>;

/// Several values that are allocated consecutively. This can be used in combination with
/// [`Value::call_values`] and [`WithOutput::call_values`].
///
/// [`Value::call_values`]: struct.Value.html#method.call_values
/// [`WithOutput::call_values`]: struct.WithOutput.html#method.call_values
#[derive(Copy, Clone, Debug)]
pub struct Values<'frame>(*mut *mut jl_value_t, usize, PhantomData<&'frame ()>);

impl<'frame> Values<'frame> {
    pub(crate) unsafe fn wrap(ptr: *mut *mut jl_value_t, n: usize) -> Self {
        Values(ptr, n, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut *mut jl_value_t {
        self.0
    }

    /// Returns the number of `Value`s in this group.
    pub fn len(self) -> usize {
        self.1
    }

    /// Get a specific `Value` in this group. Returns an error if the index is out of bounds.
    pub fn value(self, index: usize) -> JlrsResult<Value<'frame, 'static>> {
        if index >= self.len() {
            return Err(JlrsError::OutOfBounds(index, self.len()).into());
        }

        unsafe { Ok(Value(*(self.ptr().add(index)), PhantomData, PhantomData)) }
    }

    /// Allocate several values of the same type, this type must implement [`IntoJulia`]. The
    /// values will be protected from garbage collection inside the frame used to create them.
    /// This takes as many slots on the GC stack as values that are allocated.
    ///
    /// Returns an error if there is not enough space on the stack.
    ///
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    pub fn new<T, V, F>(frame: &mut F, data: V) -> JlrsResult<Self>
    where
        T: IntoJulia,
        V: AsRef<[T]>,
        F: Frame<'frame>,
    {
        frame
            .create_many(data.as_ref(), Internal)
            .map_err(Into::into)
    }

    /// Allocate several values of possibly different types, these types must implement
    /// [`IntoJulia`]. The values will be protected from garbage collection inside the frame used
    /// to create them. This takes as many slots on the GC stack as values that are allocated.
    ///
    /// Returns an error if there is not enough space on the stack.
    ///
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    pub fn new_dyn<'v, V, F>(frame: &mut F, data: V) -> JlrsResult<Self>
    where
        V: AsRef<[&'v dyn IntoJulia]>,
        F: Frame<'frame>,
    {
        frame
            .create_many_dyn(data.as_ref(), Internal)
            .map_err(Into::into)
    }
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
    pub fn new<V, F>(frame: &mut F, value: V) -> JlrsResult<Value<'frame, 'static>>
    where
        V: IntoJulia,
        F: Frame<'frame>,
    {
        unsafe {
            frame
                .protect(value.into_julia(), Internal)
                .map_err(Into::into)
        }
    }

    /// Create a new Julia value using the output to protect it from garbage collection, any type
    /// that implements [`IntoJulia`] can be converted using this function. The value will be
    /// protected from garbage collection until the frame the output belongs to goes out of scope.
    ///
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    pub fn new_output<'output, V, F>(
        frame: &mut F,
        output: Output<'output>,
        value: V,
    ) -> Value<'output, 'static>
    where
        V: IntoJulia,
        F: Frame<'frame>,
    {
        unsafe { frame.assign_output(output, value.into_julia(), Internal) }
    }

    /// Create a new instance of a value with `DataType` `ty`, using `values` to set the fields.
    /// This is essentially a more powerful version of [`Value::new`] and can instantiate
    /// arbitrary concrete `DataType`s, at the cost that each of its fields must have already been
    /// allocated as a `Value`. This functions returns an error if the given `DataType` is not
    /// concrete. One free slot on the GC stack is required for this function to succeed, returns
    /// an error if no slot is available.
    pub fn instantiate<'value, 'borrow, F, V>(
        frame: &mut F,
        ty: DataType,
        values: &mut V,
    ) -> JlrsResult<Value<'frame, 'borrow>>
    where
        F: Frame<'frame>,
        V: AsMut<[Value<'value, 'borrow>]>,
    {
        unsafe {
            if !ty.is::<Concrete>() {
                Err(JlrsError::NotConcrete(ty.name().into()))?;
            }

            let values = values.as_mut();
            let value = jl_new_structv(ty.ptr(), values.as_mut_ptr().cast(), values.len() as _);
            frame.protect(value, Internal).map_err(Into::into)
        }
    }

    /// Create a new instance of a value with `DataType` `ty`, using `values` to set the fields.
    /// This is essentially a more powerful version of [`Value::new`] and can instantiate
    /// arbitrary concrete `DataType`s, at the cost that each of its fields must have already been
    /// allocated as a `Value`. This functions returns an error if the given `DataType` is not
    /// concrete. One free slot on the GC stack is required for this function to succeed, returns
    /// an error if no slot is available.
    pub fn instantiate_output<'output, 'value, 'borrow, F, V>(
        frame: &mut F,
        output: Output<'output>,
        ty: DataType,
        values: &mut V,
    ) -> JlrsResult<Value<'output, 'borrow>>
    where
        F: Frame<'frame>,
        V: AsMut<[Value<'value, 'borrow>]>,
    {
        unsafe {
            if !ty.is::<Concrete>() {
                Err(JlrsError::NotConcrete(ty.name().into()))?;
            }

            let values = values.as_mut();
            let value = jl_new_structv(ty.ptr(), values.as_mut_ptr().cast(), values.len() as _);
            Ok(frame.assign_output(output, value, Internal))
        }
    }

    /// Allocates a new n-dimensional array in Julia.
    ///
    /// Creating an an array with 1, 2 or 3 dimensions requires one slot on the GC stack. If you
    /// create an array with more dimensions an extra frame is created with a single slot,
    /// temporarily taking 3 additional slots.
    ///
    /// This function returns an error if there are not enough slots available.
    pub fn new_array<T, D, F>(frame: &mut F, dimensions: D) -> JlrsResult<Value<'frame, 'static>>
    where
        T: IntoJulia + JuliaType,
        D: Into<Dimensions>,
        F: Frame<'frame>,
    {
        unsafe {
            let array = new_array::<T, _, _>(frame, dimensions)?;
            frame.protect(array, Internal).map_err(Into::into)
        }
    }

    /// Allocates a new n-dimensional array in Julia using an `Output`.
    ///
    /// Because an `Output` is used, no additional slot in the current frame is used if you create
    /// an array with 1, 2 or 3 dimensions. If you create an array with more dimensions an extra
    // frame is created with a single slot, temporarily taking 3 additional slots.
    ///
    /// This function returns an error if there are not enough slots available.
    pub fn new_array_output<'output, T, D, F>(
        frame: &mut F,
        output: Output<'output>,
        dimensions: D,
    ) -> JlrsResult<Value<'output, 'static>>
    where
        T: IntoJulia + JuliaType,
        D: Into<Dimensions>,
        F: Frame<'frame>,
    {
        unsafe {
            let array = new_array::<T, _, _>(frame, dimensions)?;
            Ok(frame.assign_output(output, array, Internal))
        }
    }

    /// Borrows an n-dimensional array from Rust for use in Julia.
    ///
    /// Borrowing an array with one dimension requires one slot on the GC stack. If you borrow an
    /// array with more dimensions, an extra frame is created with a single slot slot, temporarily
    /// taking 3 additional slots.
    ///
    /// This function returns an error if there are not enough slots available.
    pub fn borrow_array<T, D, V, F>(
        frame: &mut F,
        data: &'data mut V,
        dimensions: D,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        T: IntoJulia + JuliaType,
        D: Into<Dimensions>,
        V: BorrowMut<[T]>,
        F: Frame<'frame>,
    {
        unsafe {
            let array = borrow_array(frame, data, dimensions)?;
            frame.protect(array, Internal).map_err(Into::into)
        }
    }

    /// Borrows an n-dimensional array from Rust for use in Julia using an `Output`.
    ///
    /// Because an `Output` is used, no additional slot in the current frame is used for the array
    /// itself. If you borrow an array with more than 1 dimension an extra frame is created with a
    /// single slot, temporarily taking 3 additional slots.
    ///
    /// This function returns an error if there are not enough slots available.
    pub fn borrow_array_output<'output, 'borrow, T, D, V, F>(
        frame: &mut F,
        output: Output<'output>,
        data: &'borrow mut V,
        dimensions: D,
    ) -> JlrsResult<Value<'output, 'borrow>>
    where
        'borrow: 'output,
        T: IntoJulia + JuliaType,
        D: Into<Dimensions>,
        V: BorrowMut<[T]>,
        F: Frame<'frame>,
    {
        unsafe {
            let array = borrow_array(frame, data, dimensions)?;
            Ok(frame.assign_output(output, array, Internal))
        }
    }

    /// Moves an n-dimensional array from Rust to Julia.
    ///
    /// Moving an array with one dimension requires one slot on the GC stack. If you move an array
    /// with more dimensions, an extra frame is created with a single slot slot, temporarily
    /// taking 3 additional slots.
    ///
    /// This function returns an error if there are not enough slots available.
    pub fn move_array<T, D, F>(
        frame: &mut F,
        data: Vec<T>,
        dimensions: D,
    ) -> JlrsResult<Value<'frame, 'static>>
    where
        T: IntoJulia + JuliaType,
        D: Into<Dimensions>,
        F: Frame<'frame>,
    {
        unsafe {
            let array = move_array(frame, data, dimensions)?;
            frame
                .protect(array, Internal)
                .map(|v| {
                    let g = Global::new();
                    v.add_finalizer(
                        Module::main(g)
                            .submodule("Jlrs")
                            .unwrap()
                            .function("clean")
                            .unwrap(),
                    );
                    v
                })
                .map_err(Into::into)
        }
    }

    /// Moves an n-dimensional array from Rust to Julia using an output.
    ///
    /// Because an `Output` is used, no additional slot in the current frame is used for the array
    /// itself. If you move an array with more dimensions, an extra frame is created with a single
    /// slot slot, temporarily taking 3 additional slots.
    ///
    /// This function returns an error if there are not enough slots available.
    pub fn move_array_output<'output, T, D, F>(
        frame: &mut F,
        output: Output<'output>,
        data: Vec<T>,
        dimensions: D,
    ) -> JlrsResult<Value<'output, 'static>>
    where
        T: IntoJulia + JuliaType,
        D: Into<Dimensions>,
        F: Frame<'frame>,
    {
        unsafe {
            let array = move_array(frame, data, dimensions)?;
            let v = frame.assign_output(output, array, Internal);
            let g = Global::new();
            v.add_finalizer(
                Module::main(g)
                    .submodule("Jlrs")
                    .unwrap()
                    .function("clean")
                    .unwrap(),
            );
            Ok(v)
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
    pub fn new_union<F>(frame: &mut F, types: &mut [Value]) -> JlrsResult<Self>
    where
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
            frame.protect(un, Internal).map_err(Into::into)
        }
    }

    /// Create a new `UnionAll`. One free slot on the GC stack is required for this function to
    /// succeed, returns an error if no slot is available.
    pub fn new_unionall<F>(frame: &mut F, tvar: TypeVar, body: Value) -> JlrsResult<Self>
    where
        F: Frame<'frame>,
    {
        if !body.is_type() && !body.is::<TypeVar>() {
            Err(JlrsError::InvalidBody(body.type_name().into()))?;
        }

        unsafe {
            let ua = jl_type_unionall(tvar.ptr(), body.ptr());
            frame.protect(ua, Internal).map_err(Into::into)
        }
    }

    pub fn new_named_tuple<'value, 'borrow, F, S, T, V>(
        frame: &mut F,
        field_names: &mut S,
        values: &mut V,
    ) -> JlrsResult<Value<'frame, 'borrow>>
    where
        F: Frame<'frame>,
        S: AsMut<[T]>,
        T: TemporarySymbol,
        V: AsMut<[Value<'value, 'borrow>]>,
    {
        let output = frame.output()?;
        frame.frame(4, |frame| unsafe {
            let global = Global::new();
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
                .apply_type(frame, &mut symbol_type_vec)?
                .cast::<DataType>()?
                .instantiate(frame, &mut field_names_vec)?;

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
                .apply_type(frame, &mut field_types_vec)?;

            UnionAll::namedtuple_type(global)
                .as_value()
                .apply_type(frame, &mut [names, field_type_tup])?
                .cast::<DataType>()?
                .instantiate_output(frame, output, values)
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
    pub fn apply_type<'value, 'borrow, F, V>(self, frame: &mut F, types: &mut V) -> JlrsResult<Self>
    where
        F: Frame<'frame>,
        V: AsMut<[Value<'value, 'borrow>]>,
    {
        unsafe {
            let types = types.as_mut();
            let applied = jl_apply_type(self.ptr(), types.as_mut_ptr().cast(), types.len());
            frame.protect(applied, Internal).map_err(Into::into)
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
}

/// # Type checking
impl<'frame, 'data> Value<'frame, 'data> {
    /// Returns true if the value is `nothing`. Note that the Julia C API often returns a null
    /// pointer instead of `nothing`, this method return false if the given value is a null
    /// pointer.
    pub fn is_nothing(self) -> bool {
        unsafe { !self.is_null() && jl_typeof(self.ptr()) == jl_sys::jl_nothing_type.cast() }
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
    /// # let mut julia = unsafe { Julia::init(16).unwrap() };
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
    /// # let mut julia = unsafe { Julia::init(16).unwrap() };
    /// julia.frame(1, |_global, frame| {
    ///     let arr = Value::new_array::<f64, _, _>(frame, (3, 3))?;
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

    /// Extend the `Value`'s lifetime to the `Output's lifetime. The original value will still be
    /// valid after calling this method, the data will be protected from garbage collection until
    /// the `Output`'s frame goes out of scope.
    pub fn extend<'output, F>(self, frame: &mut F, output: Output<'output>) -> Value<'output, 'data>
    where
        F: Frame<'frame>,
    {
        unsafe { frame.assign_output(output, self.ptr().cast(), Internal) }
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
    pub fn get_nth_field<'fr, F>(self, frame: &mut F, idx: usize) -> JlrsResult<Value<'fr, 'data>>
    where
        F: Frame<'fr>,
    {
        unsafe {
            if idx >= self.n_fields() {
                return Err(JlrsError::OutOfBounds(idx, self.n_fields()).into());
            }

            frame
                .protect(jl_fieldref(self.ptr(), idx), Internal)
                .map_err(Into::into)
        }
    }

    /// Returns the field at index `idx` if it exists. If it does not exist
    /// `JlrsError::OutOfBounds` is returned. This function assumes the field must be protected
    /// from garbage collection and uses the provided output to do so.
    pub fn get_nth_field_output<'output, 'fr, F>(
        self,
        frame: &mut F,
        output: Output<'output>,
        idx: usize,
    ) -> JlrsResult<Value<'output, 'data>>
    where
        F: Frame<'fr>,
    {
        unsafe {
            if idx >= self.n_fields() {
                return Err(JlrsError::OutOfBounds(idx, self.n_fields()).into());
            }

            Ok(frame.assign_output(output, jl_fieldref(self.ptr(), idx), Internal))
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
    pub fn get_field<'fr, N, F>(self, frame: &mut F, field_name: N) -> JlrsResult<Value<'fr, 'data>>
    where
        N: TemporarySymbol,
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

            frame
                .protect(jl_get_nth_field(self.ptr(), idx as _), Internal)
                .map_err(Into::into)
        }
    }

    /// Returns the field with the name `field_name` if it exists. If it does not exist
    /// `JlrsError::NoSuchField` is returned. This function assumes the field must be protected
    /// from garbage collection and uses the provided output to do so.
    pub fn get_field_output<'output, 'fr, N, F>(
        self,
        frame: &mut F,
        output: Output<'output>,
        field_name: N,
    ) -> JlrsResult<Value<'output, 'data>>
    where
        N: TemporarySymbol,
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

            Ok(frame.assign_output(output, jl_get_nth_field(self.ptr(), idx as _), Internal))
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
impl<'data> Value<'_, 'data> {
    /// Wraps a `Value` so that a function call will not require a slot in the current frame but
    /// uses the one that was allocated for the output.
    pub fn with_output<'output>(self, output: Output<'output>) -> WithOutput<'output, Self> {
        WithOutput {
            value: self,
            output,
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
            try_protect(frame, res)
        }
    }

    /// Execute a Julia command `cmd`.
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
            try_protect(frame, res)
        }
    }

    /// Call this value as a function that takes zero arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call0<'frame, F>(self, frame: &mut F) -> JlrsResult<CallResult<'frame, 'static>>
    where
        F: Frame<'frame>,
    {
        unsafe {
            let res = jl_call0(self.ptr());
            try_protect(frame, res)
        }
    }

    /// Call this value as a function that takes zero arguments and don't protect the result from
    /// garbage collection. This is safe if you won't use the result or if you can guarantee it's
    /// a global value in Julia, e.g. `nothing` or a [`Module`].
    pub unsafe fn call0_unprotected<'base>(self, _: Global<'base>) -> CallResult<'base, 'static> {
        let res = jl_call0(self.ptr());
        let exc = jl_sys::jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
    }

    /// Call this value as a function that takes one argument, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call1<'frame, 'borrow, F>(
        self,
        frame: &mut F,
        arg: Value<'_, 'borrow>,
    ) -> JlrsResult<CallResult<'frame, 'borrow>>
    where
        F: Frame<'frame>,
    {
        unsafe {
            let res = jl_call1(self.ptr().cast(), arg.ptr());
            try_protect(frame, res)
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
        let exc = jl_sys::jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
    }

    /// Call this value as a function that takes two arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call2<'frame, 'borrow, F>(
        self,
        frame: &mut F,
        arg0: Value<'_, 'borrow>,
        arg1: Value<'_, 'borrow>,
    ) -> JlrsResult<CallResult<'frame, 'borrow>>
    where
        F: Frame<'frame>,
    {
        unsafe {
            let res = jl_call2(self.ptr().cast(), arg0.ptr(), arg1.ptr());
            try_protect(frame, res)
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
        let exc = jl_sys::jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
    }

    /// Call this value as a function that takes three arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call3<'frame, 'borrow, F>(
        self,
        frame: &mut F,
        arg0: Value<'_, 'borrow>,
        arg1: Value<'_, 'borrow>,
        arg2: Value<'_, 'borrow>,
    ) -> JlrsResult<CallResult<'frame, 'borrow>>
    where
        F: Frame<'frame>,
    {
        unsafe {
            let res = jl_call3(self.ptr().cast(), arg0.ptr(), arg1.ptr(), arg2.ptr());
            try_protect(frame, res)
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
        let exc = jl_sys::jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
    }

    /// Call this value as a function that takes several arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call<'frame, 'value, 'borrow, V, F>(
        self,
        frame: &mut F,
        args: &mut V,
    ) -> JlrsResult<CallResult<'frame, 'borrow>>
    where
        V: AsMut<[Value<'value, 'borrow>]>,
        F: Frame<'frame>,
    {
        unsafe {
            let args = args.as_mut();
            let n = args.len();
            let res = jl_call(self.ptr().cast(), args.as_mut_ptr().cast(), n as _);
            try_protect(frame, res)
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
        let exc = jl_sys::jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
    }

    /// Call this value as a function that takes keyword arguments and any number of positional
    /// arguments.
    ///
    /// Functions that can take keyword arguments can be called in two major ways, either with or
    /// without keyword arguments. The normal call-methods take care of the frst case, this one
    /// takes care of the second. In order to successfully call this function the first argument
    /// in `args` must be a `NamedTuple` which contains all the keyword arguments, the second must
    /// be the function you want to call (ie `self`), and then all positional arguments.
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
    ///       let a_value = Value::new(frame, 1isize)?;
    ///       let b_value = Value::new(frame, 10isize)?;
    ///       // `funcwithkw` takes a single positional argument of type `Int`, one keyword
    ///       // argument named `b` of the same type, and returns `a` + `b`.
    ///       let func = Module::main(global)
    ///           .submodule("JlrsTests")?
    ///           .function("funcwithkw")?;
    ///
    ///       let kw = named_tuple!(frame, "b" => b_value)?;
    ///       let res = func.call_keywords(frame, &mut [kw, func, a_value])?
    ///           .unwrap()
    ///           .cast::<isize>()?;
    ///  
    ///       assert_eq!(res, 11);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    pub fn call_keywords<'frame, 'value, 'borrow, V, F>(
        self,
        frame: &mut F,
        args: &mut V,
    ) -> JlrsResult<CallResult<'frame, 'borrow>>
    where
        F: Frame<'frame>,
        V: AsMut<[Value<'value, 'borrow>]>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.datatype().expect("").ptr().cast());
            let args = args.as_mut();
            let n = args.len();

            let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
            try_protect(frame, res)
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
        let exc = jl_sys::jl_exception_occurred();

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
    /// This function can only be called with an `AsyncFrame`, while you're waiting for this
    /// function to complete, other tasks are able to progress.
    #[cfg(all(feature = "async", target_os = "linux"))]
    pub async fn call_async<'frame, 'value, 'borrow, V>(
        self,
        frame: &mut crate::frame::AsyncFrame<'frame>,
        args: &mut V,
    ) -> JlrsResult<CallResult<'frame, 'borrow>>
    where
        V: AsMut<[Value<'value, 'borrow>]>,
    {
        unsafe { Ok(crate::julia_future::JuliaFuture::new(frame, self, args)?.await) }
    }

    /// Call this value as a function that takes several arguments in a single `Values`, this
    /// takes one slot on the GC stack. Returns the result of this function call if no exception
    /// is thrown, the exception if one is, or an error if no space is left on the stack.
    pub fn call_values<'frame, F>(
        self,
        frame: &mut F,
        args: Values,
    ) -> JlrsResult<CallResult<'frame, 'static>>
    where
        F: Frame<'frame>,
    {
        unsafe {
            let res = jl_call(self.ptr().cast(), args.ptr(), args.len() as _);
            try_protect(frame, res)
        }
    }

    /// Call this value as a function that takes several arguments in a single `Values` and don't
    /// protect the result from garbage collection. This is safe if you won't use the result or if
    /// you can guarantee it's a global value in Julia, e.g. `nothing` or a [`Module`].
    pub unsafe fn call_values_unprotected<'base>(
        self,
        _: Global<'base>,
        args: Values,
    ) -> CallResult<'base, 'static> {
        let res = jl_call(self.ptr().cast(), args.ptr(), args.len() as _);
        let exc = jl_sys::jl_exception_occurred();

        if exc.is_null() {
            Ok(Value::wrap(res))
        } else {
            Err(Value::wrap(exc))
        }
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
            let global = Global::new();
            let func = Module::main(global)
                .submodule("Jlrs")?
                .function("tracingcall")?;
            let res = jl_call1(func.ptr(), self.ptr());
            try_protect(frame, res)
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
            let global = Global::new();
            let func = Module::main(global)
                .submodule("Jlrs")?
                .function("attachstacktrace")?;
            let res = jl_call1(func.ptr(), self.ptr());
            try_protect(frame, res)
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

/// A wrapper that will let you call a `Value` as a function and store the result using an
/// `Output`. The function call will not require a slot in the current frame but uses the one
/// that was allocated for the output. You can create this by calling [`Value::with_output`].
///
/// Because the result of a function call is stored in an already allocated slot, calling a
/// function usually returns the `CallResult` directly rather than wrapping it in a `JlrsResult`.
///
/// [`Value::with_output`]: Value.html#method.with_output
pub struct WithOutput<'output, V> {
    value: V,
    output: Output<'output>,
}

impl<'output, 'frame, 'data> WithOutput<'output, Value<'frame, 'data>> {
    /// Call the value as a function that takes zero arguments and use the `Output` to extend the
    /// result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call0<'fr, F>(self, frame: &mut F) -> CallResult<'output, 'static>
    where
        F: Frame<'fr>,
    {
        unsafe {
            let res = jl_call0(self.value.ptr());
            assign(frame, self.output, res)
        }
    }

    /// Call the value as a function that takes one argument and use the `Output` to extend the
    /// result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call1<'borrow, 'fr, F>(
        self,
        frame: &mut F,
        arg: Value<'_, 'borrow>,
    ) -> CallResult<'output, 'borrow>
    where
        'borrow: 'output,
        F: Frame<'fr>,
    {
        unsafe {
            let res = jl_call1(self.value.ptr().cast(), arg.ptr());
            assign(frame, self.output, res)
        }
    }

    /// Call the value as a function that takes two arguments and use the `Output` to extend the
    /// result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call2<'borrow, 'fr, F>(
        self,
        frame: &mut F,
        arg0: Value<'_, 'borrow>,
        arg1: Value<'_, 'borrow>,
    ) -> CallResult<'output, 'borrow>
    where
        'borrow: 'output,
        F: Frame<'fr>,
    {
        unsafe {
            let res = jl_call2(self.value.ptr().cast(), arg0.ptr(), arg1.ptr());
            assign(frame, self.output, res)
        }
    }

    /// Call the value as a function that takes three arguments and use the `Output` to extend
    /// the result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call3<'borrow, 'fr, F>(
        self,
        frame: &mut F,
        arg0: Value<'_, 'borrow>,
        arg1: Value<'_, 'borrow>,
        arg2: Value<'_, 'borrow>,
    ) -> CallResult<'output, 'borrow>
    where
        'borrow: 'output,
        F: Frame<'fr>,
    {
        unsafe {
            let res = jl_call3(self.value.ptr().cast(), arg0.ptr(), arg1.ptr(), arg2.ptr());
            assign(frame, self.output, res)
        }
    }

    /// Call the value as a function that takes several arguments and use the `Output` to extend
    /// the result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call<'value, 'borrow, 'fr, V, F>(
        self,
        frame: &mut F,
        args: &mut V,
    ) -> CallResult<'output, 'borrow>
    where
        'borrow: 'output,
        V: AsMut<[Value<'value, 'borrow>]>,
        F: Frame<'fr>,
    {
        unsafe {
            let args = args.as_mut();
            let n = args.len();
            let res = jl_call(self.value.ptr().cast(), args.as_mut_ptr().cast(), n as _);
            assign(frame, self.output, res)
        }
    }

    /// Call the value as a function that takes several arguments in a single `Values` and use
    /// the `Output` to extend the result's lifetime. This takes no space on the GC stack. Returns
    /// the result of this function call if no exception is thrown or the exception if one is.
    pub fn call_values<'fr, F>(self, frame: &mut F, args: Values) -> CallResult<'output, 'static>
    where
        F: Frame<'fr>,
    {
        unsafe {
            let res = jl_call(self.value.ptr().cast(), args.ptr(), args.len() as _);
            assign(frame, self.output, res)
        }
    }

    /// Returns an anonymous function that wraps the value in a try-catch block. Calling this
    /// anonymous function with some arguments will call the value as a function with those
    /// arguments and return its result, or catch the exception, print the stackstrace, and
    /// rethrow that exception. The output is used to protect the result.
    pub fn tracing_call<'fr, F>(self, frame: &mut F) -> JlrsResult<CallResult<'output, 'data>>
    where
        F: Frame<'fr>,
    {
        unsafe {
            let global = Global::new();
            let func = Module::main(global)
                .submodule("Jlrs")?
                .function("tracingcall")?;
            let res = jl_call1(func.ptr(), self.value.ptr());
            Ok(assign(frame, self.output, res))
        }
    }

    /// Returns an anonymous function that wraps the value in a try-catch block. Calling this
    /// anonymous function with some arguments will call the value as a function with those
    /// arguments and return its result, or catch the exception and throw a new one with two
    /// fields, `exc` and `stacktrace`, containing the original exception and the stacktrace
    /// respectively. The output is used to protect the result.
    pub fn attach_stacktrace<'fr, F>(self, frame: &mut F) -> JlrsResult<CallResult<'output, 'data>>
    where
        F: Frame<'fr>,
    {
        unsafe {
            let global = Global::new();
            let func = Module::main(global)
                .submodule("Jlrs")?
                .function("attachstacktrace")?;
            let res = jl_call1(func.ptr(), self.value.ptr());
            Ok(assign(frame, self.output, res))
        }
    }
}

unsafe fn new_array<'frame, T, D, F>(frame: &mut F, dimensions: D) -> JlrsResult<*mut jl_value_t>
where
    T: IntoJulia + JuliaType,
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    let dims = dimensions.into();
    let array_type = jl_apply_array_type(T::julia_type().cast(), dims.n_dimensions());

    match dims.n_dimensions() {
        1 => Ok(jl_alloc_array_1d(array_type, dims.n_elements(0)).cast()),
        2 => Ok(jl_alloc_array_2d(array_type, dims.n_elements(0), dims.n_elements(1)).cast()),
        3 => Ok(jl_alloc_array_3d(
            array_type,
            dims.n_elements(0),
            dims.n_elements(1),
            dims.n_elements(2),
        )
        .cast()),
        n if n <= 8 => frame.frame(1, |frame| {
            let tuple = small_dim_tuple(frame, &dims)?;
            Ok(jl_new_array(array_type, tuple.ptr()).cast())
        }),
        _ => frame.frame(1, |frame| {
            let tuple = large_dim_tuple(frame, &dims)?;
            Ok(jl_new_array(array_type, tuple.ptr()).cast())
        }),
    }
}

unsafe fn borrow_array<'data, 'frame, T, D, V, F>(
    frame: &mut F,
    data: &'data mut V,
    dimensions: D,
) -> JlrsResult<*mut jl_value_t>
where
    T: IntoJulia + JuliaType,
    D: Into<Dimensions>,
    V: BorrowMut<[T]>,
    F: Frame<'frame>,
{
    let dims = dimensions.into();
    let array_type = jl_apply_array_type(T::julia_type().cast(), dims.n_dimensions());

    match dims.n_dimensions() {
        1 => Ok(jl_ptr_to_array_1d(
            array_type,
            data.borrow_mut().as_mut_ptr().cast(),
            dims.n_elements(0),
            0,
        )
        .cast()),
        n if n <= 8 => frame.frame(1, |frame| {
            let tuple = small_dim_tuple(frame, &dims)?;

            Ok(jl_ptr_to_array(
                array_type,
                data.borrow_mut().as_mut_ptr().cast(),
                tuple.ptr(),
                0,
            )
            .cast())
        }),
        _ => frame.frame(1, |frame| {
            let tuple = large_dim_tuple(frame, &dims)?;

            Ok(jl_ptr_to_array(
                array_type,
                data.borrow_mut().as_mut_ptr().cast(),
                tuple.ptr(),
                0,
            )
            .cast())
        }),
    }
}

unsafe fn move_array<'frame, T, D, F>(
    frame: &mut F,
    data: Vec<T>,
    dimensions: D,
) -> JlrsResult<*mut jl_value_t>
where
    T: IntoJulia + JuliaType,
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    let dims = dimensions.into();
    let array_type = jl_apply_array_type(T::julia_type().cast(), dims.n_dimensions());

    match dims.n_dimensions() {
        1 => Ok(jl_ptr_to_array_1d(
            array_type,
            Box::into_raw(data.into_boxed_slice()).cast(),
            dims.n_elements(0),
            1,
        )
        .cast()),
        n if n <= 8 => frame.frame(1, |frame| {
            let tuple = small_dim_tuple(frame, &dims)?;

            Ok(jl_ptr_to_array(
                array_type,
                Box::into_raw(data.into_boxed_slice()).cast(),
                tuple.ptr(),
                1,
            )
            .cast())
        }),
        _ => frame.frame(1, |frame| {
            let tuple = large_dim_tuple(frame, &dims)?;

            Ok(jl_ptr_to_array(
                array_type,
                Box::into_raw(data.into_boxed_slice()).cast(),
                tuple.ptr(),
                1,
            )
            .cast())
        }),
    }
}

unsafe fn try_protect<'frame, F>(
    frame: &mut F,
    res: *mut jl_value_t,
) -> JlrsResult<CallResult<'frame, 'static>>
where
    F: Frame<'frame>,
{
    let exc = jl_sys::jl_exception_occurred();

    if !exc.is_null() {
        match frame.protect(exc, Internal) {
            Ok(exc) => Ok(Err(exc)),
            Err(a) => Err(a.into()),
        }
    } else {
        match frame.protect(res, Internal) {
            Ok(v) => Ok(Ok(v)),
            Err(a) => Err(a.into()),
        }
    }
}

unsafe fn assign<'output, 'frame, F>(
    frame: &mut F,
    output: Output<'output>,
    res: *mut jl_value_t,
) -> CallResult<'output, 'static>
where
    F: Frame<'frame>,
{
    let exc = jl_exception_occurred();

    if !exc.is_null() {
        Err(frame.assign_output(output, exc, Internal))
    } else {
        Ok(frame.assign_output(output, res, Internal))
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
    let v = try_protect(frame, tuple)?.unwrap();

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
    let v = try_protect(frame, tuple)?.unwrap();

    let usize_ptr: *mut usize = v.ptr().cast();
    std::ptr::copy_nonoverlapping(dims.as_slice().as_ptr(), usize_ptr, n);

    Ok(v)
}
