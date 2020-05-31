//! Julia values and functions.
//!
//! When using this crate Julia data will generally be returned as a [`Value`]. A [`Value`] is a
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
use self::datatype::DataType;
use self::module::Module;
use self::symbol::Symbol;
use crate::error::{JlrsError, JlrsResult};
use crate::frame::Output;
use crate::global::Global;
use crate::traits::{
    private::Internal, ArrayDataType, Cast, Frame, IntoJulia, JuliaType, JuliaTypecheck,
    TemporarySymbol,
};
use jl_sys::{
    jl_alloc_array_1d, jl_alloc_array_2d, jl_alloc_array_3d, jl_apply_array_type,
    jl_apply_tuple_type_v, jl_call, jl_call0, jl_call1, jl_call2, jl_call3, jl_datatype_t,
    jl_exception_occurred, jl_field_index, jl_field_isptr, jl_field_names, jl_fieldref,
    jl_fieldref_noalloc, jl_get_nth_field, jl_get_nth_field_noalloc, jl_new_array,
    jl_new_struct_uninit, jl_nfields, jl_ptr_to_array, jl_ptr_to_array_1d, jl_svec_data,
    jl_svec_len, jl_typeof, jl_typeof_str, jl_value_t,
};
use std::borrow::BorrowMut;
use std::ffi::CStr;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::ptr::null_mut;
use std::slice;

pub mod array;
pub mod datatype;
pub mod module;
pub mod symbol;

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
pub type CallResult<'frame, 'data> = Result<Value<'frame, 'data>, Value<'frame, 'data>>;

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
/// and you can derive it for your own types by deriving [`JuliaStruct`] and [`JuliaTuple`].
///
/// [`Value`] also has several methods to create an n-dimensional array if the element type
/// implements [`ArrayDataType`]. This trait is a restricted version of [`JuliaType`] that is
/// implemented by all primitive types except `bool` and `char`. Types that derive [`JuliaStruct`]
/// or [`JuliaTuple`] can derive this trait if is implemented by all of its fields. A new array
/// whose data is completely managed by Julia can be created by calling [`Value::new_array`]. You
/// can also transfer the ownership of some `Vec` to Julia and treat it as an n-dimensional array
/// with [`Value::move_array`]. Finally, you can borrow anything that can be borrowed as a mutable
/// slice with [`Value::borrow_array`].
///
/// Functions and other global values defined in a module can be accessed through that module.
/// Please see the documentation for [`Module`] for more information.
///
/// ### Casting values
///
/// A `Value`'s type information can be accessed by calling [`Value::dataype`], this is usually
/// not necessary to determine what kind of data it contains; you can use [`Value::is`] to query
/// properties of the value's type. You can use [`Value::cast`] to convert the value to the
/// appropriate type. If a type implements both [`JuliaTypecheck`] and [`Cast`], which are used by
/// [`Value::is`] and [`Value::cast`] respectively, the former returning `true` when called with
/// that type as generic parameter indicates that the latter will succeed. For example,
/// `value.is::<u8>()` returning true means `value.cast::<u8>()` will succeed.
///
/// The methods that create a new `Value` come in two varieties: `<method>` and `<method>_output`.
/// The first will use a slot in the current frame to protect the value from garbage collection,
/// while the latter uses a slot in another active frame. Other features offered by `Value`
/// include accessing the fields of these values and (im)mutably borrowing their underlying array
/// data.
///
/// [`Value::assume_owned`]: struct.Value.html#method.assume_owned
/// [`IntoJulia`]: ../traits/trait.IntoJulia.html
/// [`JuliaType`]: ../traits/trait.JuliaType.html
/// [`Value::new`]: struct.Value.html#method.new
/// [`JuliaStruct`]: ../traits/trait.JuliaStruct.html
/// [`JuliaTuple`]: ../traits/trait.JuliaTuple.html
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

    /// Returns `Nothing` as a `Value`. Because `Nothing` is a singleton this takes no slot on the
    /// GC stack.
    pub fn nothing<F>(_frame: &mut F) -> Value<'frame, 'static>
    where
        F: Frame<'frame>,
    {
        unsafe { Value::wrap(null_mut()) }
    }

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

    /// Returns true if the value is `Nothing`.
    pub fn is_nothing(self) -> bool {
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
    /// More complex types that are specific to Julia, currently [`Array`], [`DataType`],
    /// [`Module`], and [`Symbol`] are also supported:
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
    /// If you derive [`JuliaStruct`] or [`JuliaTuple`] for some type, that type will also be
    /// supported by this method. A full list of supported checks can be found [here].
    ///
    /// [`Array`]: array/struct.Array.html
    /// [`DataType`]: datatype/struct.DataType.html
    /// [`Module`]: module/struct.Module.html
    /// [`Symbol`]: symbol/struct.Symbol.html
    /// [`JuliaStruct`]: ../traits/trait.JuliaStruct.html
    /// [`JuliaTuple`]: ../traits/trait.JuliaTuple.html
    /// [here]: ../traits/trait.JuliaTypecheck.html#implementors
    pub fn is<T: JuliaTypecheck>(self) -> bool {
        if self.is_nothing() {
            return false;
        }

        self.datatype().unwrap().is::<T>()
    }

    /// Returns the `DataType` of this value, or `None` if it is `Nothing`.
    pub fn datatype(self) -> Option<DataType<'frame>> {
        if self.is_nothing() {
            return None;
        }

        unsafe { Some(DataType::wrap(jl_typeof(self.ptr()).cast())) }
    }

    /// Cast the value to one of the following types: [`Array`], [`DataType`], [`Module`], or
    /// [`Symbol`].
    pub fn cast<T: Cast<'frame, 'data>>(self) -> JlrsResult<<T as Cast<'frame, 'data>>::Output> {
        T::cast(self)
    }

    /// Cast the value to one of the following types without checking if the cast is valid:
    /// [`Array`], [`DataType`], [`Module`], or [`Symbol`].
    ///
    /// Safety:
    ///
    /// You must guarantee `self.is::<T>()` would have returned `true`.
    pub unsafe fn cast_unchecked<T: Cast<'frame, 'data>>(
        self,
    ) -> <T as Cast<'frame, 'data>>::Output {
        T::cast_unchecked(self)
    }

    /// Returns the type name of this value.
    pub fn type_name(self) -> &'frame str {
        if self.is_nothing() {
            return "Nothing";
        }

        unsafe {
            let type_name = jl_typeof_str(self.ptr());
            let type_name_ref = CStr::from_ptr(type_name);
            type_name_ref.to_str().unwrap()
        }
    }

    /// Returns true if the value is an array with elements of type `T`.
    pub fn is_array_of<T: ArrayDataType>(self) -> bool {
        match self.cast::<Array>() {
            Ok(arr) => arr.contains::<T>(),
            Err(_) => false,
        }
    }

    /// Returns the field names of this value as a slice of `Symbol`s. These symbols can be used
    /// to access their fields with [`Value::get_field`].
    ///
    /// [`Value::get_field`]: struct.Value.html#method.get_field
    pub fn field_names<'base>(self, _: Global<'base>) -> &[Symbol<'base>] {
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

    /// Allocates a new n-dimensional array in Julia.
    ///
    /// Creating an an array with 1, 2 or 3 dimensions requires one slot on the GC stack. If you
    /// create an array with more dimensions an extra frame is created with a single slot,
    /// temporarily taking 3 additional slots.
    ///
    /// This function returns an error if there are not enough slots available.
    pub fn new_array<T, D, F>(frame: &mut F, dimensions: D) -> JlrsResult<Value<'frame, 'static>>
    where
        T: ArrayDataType,
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
        T: ArrayDataType,
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
        T: ArrayDataType,
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
        T: ArrayDataType,
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
        T: ArrayDataType,
        D: Into<Dimensions>,
        F: Frame<'frame>,
    {
        unsafe {
            let array = move_array(frame, data, dimensions)?;
            frame.protect(array, Internal).map_err(Into::into)
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
        T: ArrayDataType,
        D: Into<Dimensions>,
        F: Frame<'frame>,
    {
        unsafe {
            let array = move_array(frame, data, dimensions)?;
            Ok(frame.assign_output(output, array, Internal))
        }
    }

    /// Wraps a `Value` so that a function call will not require a slot in the current frame but
    /// uses the one that was allocated for the output.
    pub fn with_output<'output>(
        self,
        output: Output<'output>,
    ) -> WithOutput<'output, Value<'frame, 'data>> {
        WithOutput {
            value: self,
            output,
        }
    }

    /// Call this value as a function that takes zero arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call0<F>(self, frame: &mut F) -> JlrsResult<CallResult<'frame, 'static>>
    where
        F: Frame<'frame>,
    {
        unsafe {
            let res = jl_call0(self.ptr());
            try_protect(frame, res)
        }
    }

    /// Call this value as a function that takes one argument, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call1<'borrow, F>(
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

    /// Call this value as a function that takes two arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call2<'borrow, F>(
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

    /// Call this value as a function that takes three arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call3<'borrow, F>(
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

    /// Call this value as a function that takes several arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call<'value, 'borrow, V, F>(
        self,
        frame: &mut F,
        mut args: V,
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

    /// Call this value as a function that takes several arguments in a single `Values`, this
    /// takes one slot on the GC stack. Returns the result of this function call if no exception
    /// is thrown, the exception if one is, or an error if no space is left on the stack.
    pub fn call_values<F>(
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

    /// Returns an anonymous function that wraps this value in a try-catch block. Calling this
    /// anonymous function with some arguments will call the value as a function with those
    /// arguments and return its result, or catch the exception, print the stackstrace, and
    /// rethrow that exception. This takes one slot on the GC stack. You must include `jlrs.jl` to
    /// use this function.
    pub fn tracing_call<F>(self, frame: &mut F) -> JlrsResult<CallResult<'frame, 'data>>
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
    /// respectively. This takes one slot on the GC stack. You must include `jlrs.jl` to use this
    /// function.
    pub fn attach_stacktrace<F>(self, frame: &mut F) -> JlrsResult<CallResult<'frame, 'data>>
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

impl<'frame, 'data> Debug for Value<'frame, 'data> {
    #[cfg_attr(tarpaulin, skip)]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Value").field(&self.type_name()).finish()
    }
}

/// A wrapper that will let you call a `Value` as a function and store the result using an
/// `Output`. The function call will not require a slot in the current frame but uses the one
/// that was allocated for the output. You can create this by calling [`Value::with_output`].
///
/// Because the result of a function call is stored in an already allocated slot, calling a
/// function returns the `CallResult` directly rather than wrapping it in a `JlrsResult` except
/// for the methods that depend on `jlrs.jl`.
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
        mut args: V,
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
    /// rethrow that exception. The output is used to protect the result. You must include
    /// `jlrs.jl` to use this function.
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
    /// respectively. The output is used to protect the result. You must include `jlrs.jl` to use
    /// this function.
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
    T: JuliaType,
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
    T: JuliaType,
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
    T: JuliaType,
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
