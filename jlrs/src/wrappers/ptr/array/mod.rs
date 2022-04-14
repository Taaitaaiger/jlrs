//! Wrappers for `Array`, create and access n-dimensional Julia arrays from Rust.
//!
//! You will find two wrappers in this module that can be used to work with Julia arrays from
//! Rust. An [`Array`] is the Julia array itself, [`TypedArray`] is also available which can be
//! used if the element type implements [`ValidLayout`].
//!
//! Several methods are available to create new arrays. [`Array::new`] lets you create a new array
//! for any type that implements [`IntoJulia`], while [`Array::new_for`] can be used to create a
//! new array for arbitrary types. These methods allocate a new array, it's also possible to use
//! data from Rust directly if it implements `IntoJulia`. [`Array::from_vec`] and can be used to
//! move the data from Rust to Julia, while [`Array::from_slice`] can be used to mutably borrow
//! data from Rust as a Julia array.
//!
//! How the contents of the array must be accessed from Rust depends on the type of the elements.
//! [`Array`] provides methods to (mutably) access their contents for all three possible
//! layouts of the elements: inline, pointer, and bits union.
//!
//! Accessing the contents of an array requires an n-dimensional index. The [`Dims`] trait is
//! available for this purpose. This trait is implemented for tuples of four or fewer `usize`s;
//! `[usize; N]` and `&[usize; N]` implement it for all `N`, `&[usize]` can be used if `N` is not
//! a constant at compile time.

#[cfg(not(all(target_os = "windows", feature = "lts")))]
use crate::error::JuliaResult;
// // #[cfg(not(all(target_os = "windows", feature = "lts")))]
use crate::memory::scope::PartialScope;
use crate::{
    convert::into_julia::IntoJulia,
    error::{JlrsError, JlrsResult, CANNOT_DISPLAY_TYPE},
    impl_debug,
    layout::{typecheck::Typecheck, valid_layout::ValidLayout},
    memory::{
        frame::private::Frame as _,
        frame::Frame,
        get_tls,
        global::Global,
        output::Output,
        scope::{private::PartialScope as _, Scope},
    },
    private::Private,
    wrappers::ptr::{
        array::{
            data::{
                copied::CopiedArray,
                inline::{InlineArrayData, InlineArrayDataMut, UnrestrictedInlineArrayDataMut},
                union::{UnionArrayData, UnionArrayDataMut, UnresistrictedUnionArrayDataMut},
                value::{UnrestrictedValueArrayDataMut, ValueArrayData, ValueArrayDataMut},
            },
            dimensions::{ArrayDimensions, Dims},
        },
        datatype::DataType,
        private::Wrapper as WrapperPriv,
        union::Union,
        value::Value,
        Wrapper,
    },
};
use jl_sys::{
    jl_alloc_array_1d, jl_alloc_array_2d, jl_alloc_array_3d, jl_apply_array_type,
    jl_apply_tuple_type_v, jl_array_data, jl_array_dims_ptr, jl_array_eltype, jl_array_ndims,
    jl_array_t, jl_datatype_t, jl_gc_add_ptr_finalizer, jl_new_array, jl_new_struct_uninit,
    jl_pchar_to_array, jl_ptr_to_array, jl_ptr_to_array_1d,
};

#[cfg(not(all(target_os = "windows", feature = "lts")))]
use jl_sys::{
    jlrs_alloc_array_1d, jlrs_alloc_array_2d, jlrs_alloc_array_3d, jlrs_array_del_beg,
    jlrs_array_del_end, jlrs_array_grow_beg, jlrs_array_grow_end, jlrs_new_array,
    jlrs_reshape_array, jlrs_result_tag_t_JLRS_RESULT_ERR,
};

use std::{
    cell::UnsafeCell,
    ffi::c_void,
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    mem,
    ptr::{null_mut, NonNull},
    slice,
};

use super::{type_name::TypeName, WrapperRef};

pub mod data;
pub mod dimensions;

/// An n-dimensional Julia array. It can be used in combination with [`DataType::is`] and
/// [`Value::is`], if the check returns `true` the [`Value`] can be cast to `Array`:
///
/// ```
/// # #[cfg(not(all(target_os = "windows", feature = "lts")))]
/// # mod example {
/// # use jlrs::prelude::*;
/// # use jlrs::util::JULIA;
/// # fn main() {
/// # JULIA.with(|j| {
/// # let mut julia = j.borrow_mut();
/// julia.scope(|_global, frame| {
///     let _arr = Array::new::<f64, _, _, _>(&mut *frame, (3, 3))?
///         .into_jlrs_result()?;
///
///     Ok(())
/// }).unwrap();
/// # });
/// # }
/// # }
/// ```
///
/// Each element in the backing storage is either stored as a [`Value`] or inline. If the inline
/// data is a bits union, the flag indicating the active variant is stored separately from the
/// elements. You can check how the data is stored by calling [`Array::is_value_array`],
/// [`Array::is_inline_array`], or [`Array::is_union_array`].
///
/// Arrays that contain integers or floats are an example of inline arrays. Their data is stored
/// as an array that contains numbers of the appropriate type, for example an array of `Float32`s
/// in Julia is backed by an an array of `f32`s. The data in these arrays can be accessed with
/// [`Array::inline_data`] and [`Array::inline_data_mut`], and copied from Julia to Rust with
/// [`Array::copy_inline_data`]. In order to call these methods the type of the elements must be
/// provided, this type must implement [`ValidLayout`] to ensure the layouts in Rust and Julia are
/// compatible.
///
/// If the data isn't inlined each element is stored as a [`Value`]. This data can be accessed
/// using [`Array::value_data`] and [`Array::value_data_mut`].
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Array<'scope, 'data>(
    NonNull<jl_array_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

impl<'data> Array<'_, 'data> {
    /// Allocates a new n-dimensional array in Julia of dimensions `dims`. If `dims = (4, 2)` a
    /// two-dimensional array with 4 rows and 2 columns is created. This method can only be used
    /// in combination with types that implement `IntoJulia`. If you want to create an array for a
    /// type that doesn't implement this trait you must use [`Array::new_for`].
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn new<'target: 'current, 'current, T, D, S, F>(
        scope: S,
        dims: D,
    ) -> JlrsResult<JuliaResult<'target, 'static, Array<'target, 'static>>>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        unsafe {
            let (output, frame) = scope.split()?;
            frame.scope(|frame| {
                let global = frame.global();
                let elty_ptr = T::julia_type(global).ptr();
                let array_type = jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());
                let _: Value = (&mut *frame).value(NonNull::new_unchecked(array_type), Private)?;

                let array = match dims.n_dimensions() {
                    1 => jlrs_alloc_array_1d(array_type, dims.n_elements(0)),
                    2 => jlrs_alloc_array_2d(array_type, dims.n_elements(0), dims.n_elements(1)),
                    3 => jlrs_alloc_array_3d(
                        array_type,
                        dims.n_elements(0),
                        dims.n_elements(1),
                        dims.n_elements(2),
                    ),
                    n if n <= 8 => {
                        let tuple = small_dim_tuple(frame, dims)?;
                        jlrs_new_array(array_type, tuple.unwrap(Private))
                    }
                    _ => {
                        let tuple = large_dim_tuple(frame, dims)?;
                        jlrs_new_array(array_type, tuple.unwrap(Private))
                    }
                };

                let res = if array.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                    Err(NonNull::new_unchecked(array.data))
                } else {
                    Ok(NonNull::new_unchecked(array.data.cast::<jl_array_t>()))
                };

                let output = output.into_scope(frame);
                output.call_result(res, Private)
            })
        }
    }

    /// Allocates a new n-dimensional array in Julia of dimensions `dims`. If `dims = (4, 2)` a
    /// two-dimensional array with 4 rows and 2 columns is created. This method can only be used
    /// in combination with types that implement `IntoJulia`. If you want to create an array for a
    /// type that doesn't implement this trait you must use [`Array::new_for`].
    ///
    /// If the array size is too large, Julia throws an exception which isn't caught.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn new_unchecked<'target: 'current, 'current, T, D, S, F>(
        scope: S,
        dims: D,
    ) -> JlrsResult<Array<'target, 'static>>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        let global = scope.global();
        let elty_ptr = T::julia_type(global).ptr();
        let array_type = jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());

        match dims.n_dimensions() {
            1 => scope.value(
                NonNull::new_unchecked(jl_alloc_array_1d(array_type, dims.n_elements(0))),
                Private,
            ),
            2 => scope.value(
                NonNull::new_unchecked(jl_alloc_array_2d(
                    array_type,
                    dims.n_elements(0),
                    dims.n_elements(1),
                )),
                Private,
            ),
            3 => scope.value(
                NonNull::new_unchecked(jl_alloc_array_3d(
                    array_type,
                    dims.n_elements(0),
                    dims.n_elements(1),
                    dims.n_elements(2),
                )),
                Private,
            ),
            n if n <= 8 => {
                let (output, scope) = scope.split()?;
                scope.scope_with_capacity(1, |frame| {
                    let tuple = small_dim_tuple(frame, dims)?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(jl_new_array(array_type, tuple.unwrap(Private))),
                        Private,
                    )
                })
            }
            _ => {
                let (output, scope) = scope.split()?;
                scope.scope_with_capacity(1, |frame| {
                    let tuple = large_dim_tuple(frame, dims)?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(jl_new_array(array_type, tuple.unwrap(Private))),
                        Private,
                    )
                })
            }
        }
    }

    /// Allocates a new n-dimensional array in Julia for elements of type `ty`, which must be a
    /// `Union`, `UnionAll` or `DataType`, and dimensions `dims`. If `dims = (4, 2)` a
    /// two-dimensional array with 4 rows and 2 columns is created. If an exception is thrown due
    /// to either the type or dimensions being invalid it's caught and returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn new_for<'target: 'current, 'current, D, S, F>(
        scope: S,
        dims: D,
        ty: Value,
    ) -> JlrsResult<JuliaResult<'target, 'static, Array<'target, 'static>>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        unsafe {
            let (output, frame) = scope.split()?;
            frame.scope(|frame| {
                let array_type = jl_apply_array_type(ty.unwrap(Private), dims.n_dimensions());
                let _: Value = (&mut *frame).value(NonNull::new_unchecked(array_type), Private)?;

                let array = match dims.n_dimensions() {
                    1 => jlrs_alloc_array_1d(array_type, dims.n_elements(0)),
                    2 => jlrs_alloc_array_2d(array_type, dims.n_elements(0), dims.n_elements(1)),
                    3 => jlrs_alloc_array_3d(
                        array_type,
                        dims.n_elements(0),
                        dims.n_elements(1),
                        dims.n_elements(2),
                    ),
                    n if n <= 8 => {
                        let tuple = small_dim_tuple(frame, dims)?;
                        jlrs_new_array(array_type, tuple.unwrap(Private))
                    }
                    _ => {
                        let tuple = large_dim_tuple(frame, dims)?;
                        jlrs_new_array(array_type, tuple.unwrap(Private))
                    }
                };

                let res = if array.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                    Err(NonNull::new_unchecked(array.data))
                } else {
                    Ok(NonNull::new_unchecked(array.data.cast::<jl_array_t>()))
                };

                let output = output.into_scope(frame);
                output.call_result(res, Private)
            })
        }
    }

    /// Allocates a new n-dimensional array in Julia for elements of type `ty`, which must be a
    /// `Union`, `UnionAll` or `DataType`, and dimensions `dims`. If `dims = (4, 2)` a
    /// two-dimensional array with 4 rows and 2 columns is created. If an exception is thrown due
    /// to either the type or dimensions being invalid which isn't caught.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn new_for_unchecked<'target: 'current, 'current, D, S, F>(
        scope: S,
        dims: D,
        ty: Value,
    ) -> JlrsResult<Array<'target, 'static>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        let array_type = jl_apply_array_type(ty.unwrap(Private), dims.n_dimensions());

        match dims.n_dimensions() {
            1 => scope.value(
                NonNull::new_unchecked(jl_alloc_array_1d(array_type, dims.n_elements(0))),
                Private,
            ),
            2 => scope.value(
                NonNull::new_unchecked(jl_alloc_array_2d(
                    array_type,
                    dims.n_elements(0),
                    dims.n_elements(1),
                )),
                Private,
            ),
            3 => scope.value(
                NonNull::new_unchecked(jl_alloc_array_3d(
                    array_type,
                    dims.n_elements(0),
                    dims.n_elements(1),
                    dims.n_elements(2),
                )),
                Private,
            ),
            n if n <= 8 => {
                let (output, scope) = scope.split()?;
                scope.scope_with_capacity(1, |frame| {
                    let tuple = small_dim_tuple(frame, dims)?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(jl_new_array(array_type, tuple.unwrap(Private))),
                        Private,
                    )
                })
            }
            _ => {
                let (output, scope) = scope.split()?;
                scope.scope_with_capacity(1, |frame| {
                    let tuple = large_dim_tuple(frame, dims)?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(jl_new_array(array_type, tuple.unwrap(Private))),
                        Private,
                    )
                })
            }
        }
    }

    /// Borrows an n-dimensional array from Rust for use in Julia with dimensions `dims`. If
    /// `dims` = (4, 2)` a two-dimensional array with 4 rows and 2 columns is created.
    pub fn from_slice<'target: 'current, 'current, T, D, S, F>(
        scope: S,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<Array<'target, 'data>>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        if dims.size() != data.len() {
            Err(JlrsError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        unsafe {
            let global = scope.global();
            let array_type =
                jl_apply_array_type(T::julia_type(global).ptr().cast(), dims.n_dimensions());

            match dims.n_dimensions() {
                1 => scope.value(
                    NonNull::new_unchecked(
                        jl_ptr_to_array_1d(
                            array_type,
                            data.as_mut_ptr().cast(),
                            dims.n_elements(0),
                            0,
                        )
                        .cast(),
                    ),
                    Private,
                ),
                n if n <= 8 => {
                    let (output, scope) = scope.split()?;
                    scope.scope_with_capacity(1, |frame| {
                        let tuple = small_dim_tuple(frame, dims)?;
                        output.into_scope(frame).value(
                            NonNull::new_unchecked(
                                jl_ptr_to_array(
                                    array_type,
                                    data.as_mut_ptr().cast(),
                                    tuple.unwrap(Private),
                                    0,
                                )
                                .cast(),
                            ),
                            Private,
                        )
                    })
                }
                _ => {
                    let (output, scope) = scope.split()?;
                    scope.scope_with_capacity(1, |frame| {
                        let tuple = large_dim_tuple(frame, dims)?;
                        output.into_scope(frame).value(
                            NonNull::new_unchecked(
                                jl_ptr_to_array(
                                    array_type,
                                    data.as_mut_ptr().cast(),
                                    tuple.unwrap(Private),
                                    0,
                                )
                                .cast(),
                            ),
                            Private,
                        )
                    })
                }
            }
        }
    }

    /// Moves an n-dimensional array from Rust for use in Julia with dimensions `dims`. If
    /// `dims = (4, 2)` a two-dimensional array with 4 rows and 2 columns is created.
    pub fn from_vec<'target: 'current, 'current, T, D, S, F>(
        scope: S,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<Array<'target, 'static>>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        if dims.size() != data.len() {
            Err(JlrsError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        unsafe {
            let global = scope.global();

            let (output, scope) = scope.split()?;
            scope.scope_with_capacity(1, |frame| {
                let array_type =
                    jl_apply_array_type(T::julia_type(global).ptr().cast(), dims.n_dimensions());
                let _: Value = frame
                    .push_root(NonNull::new_unchecked(array_type), Private)
                    .map_err(JlrsError::alloc_error)?;

                match dims.n_dimensions() {
                    1 => {
                        let array = jl_ptr_to_array_1d(
                            array_type,
                            Box::into_raw(data.into_boxed_slice()).cast(),
                            dims.n_elements(0),
                            0,
                        );

                        jl_gc_add_ptr_finalizer(
                            get_tls(),
                            array.cast(),
                            droparray::<T> as *mut c_void,
                        );

                        output
                            .into_scope(frame)
                            .value(NonNull::new_unchecked(array), Private)
                    }
                    n if n <= 8 => {
                        let tuple = small_dim_tuple(frame, dims)?;
                        let array = jl_ptr_to_array(
                            array_type,
                            Box::into_raw(data.into_boxed_slice()).cast(),
                            tuple.unwrap(Private),
                            0,
                        );

                        jl_gc_add_ptr_finalizer(
                            get_tls(),
                            array.cast(),
                            droparray::<T> as *mut c_void,
                        );

                        output
                            .into_scope(frame)
                            .value(NonNull::new_unchecked(array), Private)
                    }
                    _ => {
                        let tuple = large_dim_tuple(frame, dims)?;
                        let array = jl_ptr_to_array(
                            array_type,
                            Box::into_raw(data.into_boxed_slice()).cast(),
                            tuple.unwrap(Private),
                            0,
                        );

                        jl_gc_add_ptr_finalizer(
                            get_tls(),
                            array.cast(),
                            droparray::<T> as *mut c_void,
                        );

                        output
                            .into_scope(frame)
                            .value(NonNull::new_unchecked(array), Private)
                    }
                }
            })
        }
    }

    /// Convert a string to an array.
    pub fn from_string<'target, A: AsRef<str>, S>(
        scope: S,
        data: A,
    ) -> JlrsResult<Array<'target, 'static>>
    where
        A: IntoJulia,
        S: PartialScope<'target>,
    {
        let string = data.as_ref();
        let nbytes = string.bytes().len();
        let ptr = string.as_ptr();
        unsafe {
            let arr = jl_pchar_to_array(ptr.cast(), nbytes);
            scope.value(NonNull::new_unchecked(arr), Private)
        }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> Array<'target, 'data> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<Array>(ptr);
            Array::wrap_non_null(ptr, Private)
        }
    }

    #[inline(always)]
    pub(crate) fn data_ptr(self) -> *mut c_void {
        unsafe { self.unwrap_non_null(Private).as_ref().data }
    }
}

impl<'scope, 'data> Array<'scope, 'data> {
    /// Returns the array's dimensions.
    pub fn dimensions(self) -> ArrayDimensions<'scope> {
        ArrayDimensions::new(self)
    }

    /// Returns the type of this array's elements.
    pub fn element_type(self) -> Value<'scope, 'static> {
        unsafe { Value::wrap(jl_array_eltype(self.unwrap(Private).cast()).cast(), Private) }
    }

    /// Returns `true` if the layout of the elements is compatible with `T`.
    pub fn contains<T: ValidLayout>(self) -> bool {
        unsafe {
            T::valid_layout(Value::wrap(
                jl_array_eltype(self.unwrap(Private).cast()).cast(),
                Private,
            ))
        }
    }

    /// Returns `true` if the layout of the elements is compatible with `T` and these elements are
    /// stored inline.
    pub fn contains_inline<T: ValidLayout>(self) -> bool {
        self.contains::<T>() && self.is_inline_array()
    }

    /// Returns true if the elements of the array are stored inline.
    pub fn is_inline_array(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().flags.ptrarray() == 0 }
    }

    /// Returns true if the elements of the array are stored inline and the element type is a
    /// union type. In this case the contents of the array can be accessed from Rust with
    /// [`Array::union_data`] and [`Array::union_data_mut`].
    pub fn is_union_array(self) -> bool {
        self.is_inline_array() && self.element_type().is::<Union>()
    }

    /// Returns true if the elements of the array are stored inline and at least one of the fields
    /// of the inlined type is a pointer.
    pub fn has_inlined_pointers(self) -> bool {
        unsafe {
            let flags = self.unwrap_non_null(Private).as_ref().flags;
            self.is_inline_array() && flags.hasptr() != 0
        }
    }

    /// Returns `true` if elements of this array are zero-initialized.
    pub fn zero_init(self) -> bool {
        unsafe {
            let flags = self.unwrap_non_null(Private).as_ref().flags;
            if flags.ptrarray() == 1 || flags.hasptr() == 1 {
                return true;
            }

            let elty = self.element_type();
            if let Ok(dt) = elty.cast::<DataType>() {
                return dt.zero_init();
            } else {
                false
            }
        }
    }

    /// Returns true if the elements of the array are stored as [`Value`]s.
    pub fn is_value_array(self) -> bool {
        !self.is_inline_array()
    }

    /// Convert this untyped array to a [`TypedArray`].
    pub fn try_as_typed<T>(self) -> JlrsResult<TypedArray<'scope, 'data, T>>
    where
        T: Clone + ValidLayout + Debug,
    {
        if self.contains::<T>() {
            unsafe {
                Ok(TypedArray::wrap_non_null(
                    self.unwrap_non_null(Private),
                    Private,
                ))
            }
        } else {
            Err(JlrsError::WrongType {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }
    }

    /// Convert this untyped array to a [`TypedArray`] without checking if this conversion is
    /// valid.
    ///
    /// Safety: `T` must be a valid representation of the data stored in the array.
    pub unsafe fn as_typed_unchecked<T>(self) -> TypedArray<'scope, 'data, T>
    where
        T: Clone + ValidLayout + Debug,
    {
        TypedArray::wrap_non_null(self.unwrap_non_null(Private), Private)
    }

    /// Copy the data of an inline array to Rust. Returns `JlrsError::NotInline` if the data is
    /// not stored inline or `JlrsError::WrongType` if the type of the elements is incorrect.
    pub fn copy_inline_data<'frame, T, F>(self, _: &F) -> JlrsResult<CopiedArray<T>>
    where
        T: ValidLayout,
        F: Frame<'frame>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe {
            let jl_data = jl_array_data(self.unwrap(Private).cast()).cast();
            let dimensions = self.dimensions().into_dimensions();

            let sz = dimensions.size();
            let mut data = Vec::with_capacity(sz);
            let ptr = data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(jl_data, ptr, sz);
            data.set_len(sz);

            Ok(CopiedArray::new(data.into_boxed_slice(), dimensions))
        }
    }

    /// Immutably borrow inline array data, you can borrow data from multiple arrays at the same
    /// time. Returns `JlrsError::NotInline` if the data is not stored inline or
    /// `JlrsError::WrongType` if the type of the elements is incorrect.
    pub fn inline_data<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<InlineArrayData<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
        F: Frame<'frame>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe { Ok(InlineArrayData::new(self, frame)) }
    }

    /// Mutably borrow inline array data, you can mutably borrow a single array at a time. Returns
    /// `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType` if the
    /// type of the elements is incorrect.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn inline_data_mut<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<InlineArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        'borrow: 'data,
        T: ValidLayout,
        F: Frame<'frame>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(InlineArrayDataMut::new(self, frame))
    }

    /// Mutably borrow inline array data without the restriction that only a single array can be
    /// mutably borrowed.
    ///
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
    ///
    /// Safety: It's your responsibility to ensure you don't create multiple mutable
    /// references to the same data. Mutating Julia data is generally unsafe because it can't be
    /// guaranteed mutating this value is allowed.
    pub unsafe fn unrestricted_inline_data_mut<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedInlineArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
        F: Frame<'frame>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(UnrestrictedInlineArrayDataMut::new(self, frame))
    }

    /// Immutably borrow the data of this array of values, you can borrow data from multiple
    /// arrays at the same time. The values themselves can be mutable, but you can't replace an
    /// element with another value. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn value_data<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ValueArrayData<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        if !self.is_value_array() {
            Err(JlrsError::Inline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe { Ok(ValueArrayData::new(self, frame)) }
    }

    /// Immutably borrow the data of this array of wrappers, you can borrow data from multiple
    /// arrays at the same time. The values themselves can be mutable, but you can't replace an
    /// element with another value. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn wrapper_data<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ValueArrayData<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
        T: WrapperRef<'scope, 'data> + ValidLayout,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe { Ok(ValueArrayData::new(self, frame)) }
    }

    /// Mutably borrow the data of this array of values, you can mutably borrow a single array at
    /// the same time. Returns `JlrsError::Inline` if the data is stored inline.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn value_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ValueArrayDataMut<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        if !self.is_value_array() {
            Err(JlrsError::Inline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(ValueArrayDataMut::new(self, frame))
    }

    /// Mutably borrow the data of this array of wrappers, you can mutably borrow a single array
    /// at the same time. Returns `JlrsError::Inline` if the data is stored inline.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn wrapper_data_mut<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ValueArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
        T: WrapperRef<'scope, 'data> + ValidLayout,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(ValueArrayDataMut::new(self, frame))
    }

    /// Mutably borrow the data of this array of values without the restriction that only a single
    /// array can be mutably borrowed.
    ///
    /// Returns `JlrsError::Inline` if the data is stored inline.
    ///
    /// Safety: It's your responsibility to ensure you don't create multiple mutable
    /// references to the same data. Mutating Julia data is generally unsafe because it can't be
    /// guaranteed mutating this value is allowed.
    pub unsafe fn unrestricted_value_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedValueArrayDataMut<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        if !self.is_value_array() {
            Err(JlrsError::Inline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(UnrestrictedValueArrayDataMut::new(self, frame))
    }

    /// Mutably borrow the data of this array of wrappers without the restriction that only a
    /// single array can be mutably borrowed.
    ///
    /// Returns `JlrsError::WrongType` if the type doesn't match the type of the elements.
    ///
    /// Safety: It's your responsibility to ensure you don't create multiple mutable
    /// references to the same data. Mutating Julia data is generally unsafe because it can't be
    /// guaranteed mutating this value is allowed.
    pub unsafe fn unrestricted_wrapper_data_mut<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedValueArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
        T: WrapperRef<'scope, 'data> + ValidLayout,
    {
        if !self.is_value_array() {
            Err(JlrsError::WrongType {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(UnrestrictedValueArrayDataMut::new(self, frame))
    }
}

impl<'scope> Array<'scope, 'static> {
    /// Access the contents of a bits-union array.
    pub fn union_data<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnionArrayData<'borrow, 'scope>>
    where
        F: Frame<'frame>,
    {
        if self.is_union_array() {
            unsafe { Ok(UnionArrayData::new(self, frame)) }
        } else {
            let elem_ty = self.element_type().display_string_or(CANNOT_DISPLAY_TYPE);
            let inline = !self.is_value_array();
            Err(JlrsError::NotAUnionArray { elem_ty, inline })?
        }
    }

    /// Mutably borrow the data of this array of bits-unions, you can mutably borrow a single
    /// array at a time.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn union_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<UnionArrayDataMut<'borrow, 'scope>>
    where
        F: Frame<'frame>,
    {
        if self.is_union_array() {
            Ok(UnionArrayDataMut::new(self, frame))
        } else {
            let elem_ty = self.element_type().display_string_or(CANNOT_DISPLAY_TYPE);
            let inline = !self.is_value_array();
            Err(JlrsError::NotAUnionArray { elem_ty, inline })?
        }
    }

    /// Mutably borrow the data of this array of bits-unions without the restriction that only a
    /// single array can be mutably borrowed.
    ///
    /// Safety: It's your responsibility to ensure you don't create multiple mutable references to
    /// the same data. Mutating Julia data is generally unsafe because it can't be guaranteed
    /// mutating this value is allowed.
    pub unsafe fn unrestricted_union_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnresistrictedUnionArrayDataMut<'borrow, 'scope>>
    where
        F: Frame<'frame>,
    {
        if self.is_union_array() {
            Ok(UnresistrictedUnionArrayDataMut::new(self, frame))
        } else {
            let elem_ty = self.element_type().display_string_or(CANNOT_DISPLAY_TYPE);
            let inline = !self.is_value_array();
            Err(JlrsError::NotAUnionArray { elem_ty, inline })?
        }
    }
}

impl<'scope> Array<'scope, 'static> {
    /// Reshape the array, a new array is returned that has dimensions `dims`. This new array and
    /// `self` share their data. This method returns an exception if the old and new array have a
    /// different number of elements or if the array contains data that has been borrowed or moved
    /// from Rust.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn reshape<'target: 'current, 'current, 'borrow, D, S, F>(
        self,
        scope: S,
        dims: D,
    ) -> JlrsResult<JuliaResult<'target, 'static>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        unsafe {
            let (output, scope) = scope.split()?;
            scope.scope_with_capacity(2, |frame| {
                let elty_ptr = self.element_type().unwrap(Private);
                let array_type = jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());
                let _: Value = frame
                    .push_root(NonNull::new_unchecked(array_type), Private)
                    .map_err(|x| JlrsError::alloc_error(x))?;

                let tuple = if dims.n_dimensions() <= 8 {
                    small_dim_tuple(frame, dims)?
                } else {
                    large_dim_tuple(frame, dims)?
                };

                let res =
                    jlrs_reshape_array(array_type, self.unwrap(Private), tuple.unwrap(Private));

                let output = output.into_scope(frame);
                let result = if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                    Err(NonNull::new_unchecked(res.data))
                } else {
                    Ok(NonNull::new_unchecked(res.data).cast())
                };

                output.call_result(result, Private)
            })
        }
    }

    /// Inserts `inc` more elements at the end of the array. The array must be 1D and
    /// contain no data borrowed or moved from Rust, otherwise an exception is returned.
    /// Depending on the type of the array elements the newly added elements will either be
    /// left uninitialized, or their contents will be set to 0s. It's set to 0s if
    /// `DataType::zero_init` returns true, if the elements are stored as pointers to Julia data,
    /// or if the elements contain pointers to Julia data.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn grow_end<'current, F>(
        self,
        frame: &mut F,
        inc: usize,
    ) -> JlrsResult<JuliaResult<'current, 'static, ()>>
    where
        F: Frame<'current>,
    {
        unsafe {
            let res = jlrs_array_grow_end(self.unwrap(Private), inc);

            if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                let e = (&mut *frame).value(NonNull::new_unchecked(res.data), Private)?;
                Ok(Err(e))
            } else {
                Ok(Ok(()))
            }
        }
    }

    /// Removes the final `dec` elements from the array. The array must be 1D and contain no data
    /// borrowed or moved from Rust, otherwise an exception is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn del_end<'current, F>(
        self,
        frame: &mut F,
        dec: usize,
    ) -> JlrsResult<JuliaResult<'current, 'static, ()>>
    where
        F: Frame<'current>,
    {
        unsafe {
            let res = jlrs_array_del_end(self.unwrap(Private), dec);

            if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                let e = (&mut *frame).value(NonNull::new_unchecked(res.data), Private)?;
                Ok(Err(e))
            } else {
                Ok(Ok(()))
            }
        }
    }

    /// Inserts `inc` more elements at the beginning of the array. The array must be 1D and
    /// contain no data borrowed or moved from Rust, otherwise an exception is returned.
    /// Depending on the type of the array elements the newly added elements will either be
    /// left uninitialized, or their contents will be set to 0s. It's set to 0s if
    /// `DataType::zero_init` returns true, if the elements are stored as pointers to Julia data,
    /// or if the elements contain pointers to Julia data.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn grow_begin<'current, F>(
        self,
        frame: &mut F,
        inc: usize,
    ) -> JlrsResult<JuliaResult<'current, 'static, ()>>
    where
        F: Frame<'current>,
    {
        unsafe {
            let res = jlrs_array_grow_beg(self.unwrap(Private), inc);

            if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                let e = (&mut *frame).value(NonNull::new_unchecked(res.data), Private)?;
                Ok(Err(e))
            } else {
                Ok(Ok(()))
            }
        }
    }

    /// Removes the first `dec` elements from the array. The array must be 1D and contain no data
    /// borrowed or moved from Rust, otherwise an exception is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn del_begin<'current, F>(
        self,
        frame: &mut F,
        dec: usize,
    ) -> JlrsResult<JuliaResult<'current, 'static, ()>>
    where
        F: Frame<'current>,
    {
        unsafe {
            let res = jlrs_array_del_beg(self.unwrap(Private), dec);

            if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                let e = (&mut *frame).value(NonNull::new_unchecked(res.data), Private)?;
                Ok(Err(e))
            } else {
                Ok(Ok(()))
            }
        }
    }
}

unsafe impl<'scope, 'data> Typecheck for Array<'scope, 'data> {
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name().wrapper_unchecked() == TypeName::of_array(Global::new()) }
    }
}

impl_debug!(Array<'_, '_>);

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Array<'scope, 'data> {
    type Wraps = jl_array_t;
    const NAME: &'static str = "Array";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// Exactly the same as [`Array`], except it has an explicit element type `T`.
#[derive(Clone)]
#[repr(transparent)]
pub struct TypedArray<'scope, 'data, T>(
    NonNull<jl_array_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
    PhantomData<T>,
)
where
    T: Clone + ValidLayout + Debug;

impl<'scope, 'data, T> Copy for TypedArray<'scope, 'data, T> where T: Clone + ValidLayout + Debug {}

impl<'scope, 'data, T: Clone + ValidLayout + Debug> TypedArray<'scope, 'data, T> {
    /// Returns the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'scope> {
        ArrayDimensions::new(self.as_array())
    }

    /// Returns the type of this array's elements.
    pub fn element_type(self) -> Value<'scope, 'static> {
        unsafe { Value::wrap(jl_array_eltype(self.unwrap(Private).cast()).cast(), Private) }
    }

    /// Returns true if the elements of the array are stored inline.
    pub fn is_inline_array(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().flags.ptrarray() == 0 }
    }

    /// Returns true if the elements of the array are stored inline and at least one of the field
    /// of the inlined type is a pointer.
    pub fn has_inlined_pointers(self) -> bool {
        unsafe {
            let flags = self.unwrap_non_null(Private).as_ref().flags;
            self.is_inline_array() && flags.hasptr() != 0
        }
    }

    /// Returns true if the elements of the array are stored as [`Value`]s.
    pub fn is_value_array(self) -> bool {
        !self.is_inline_array()
    }

    /// Copy the data of an inline array to Rust. Returns `JlrsError::NotInline` if the data is
    /// not stored inline.
    pub fn copy_inline_data<'frame, F>(self, _: &F) -> JlrsResult<CopiedArray<T>>
    where
        F: Frame<'frame>,
    {
        if !self.is_inline_array() {
            Err(JlrsError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe {
            let jl_data = jl_array_data(self.unwrap(Private).cast()).cast();
            let dimensions = self.dimensions().into_dimensions();

            let sz = dimensions.size();
            let mut data = Vec::with_capacity(sz);
            let ptr = data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(jl_data, ptr, sz);
            data.set_len(sz);

            Ok(CopiedArray::new(data.into_boxed_slice(), dimensions))
        }
    }

    /// Immutably borrow inline array data, you can borrow data from multiple arrays at the same
    /// time. Returns `JlrsError::NotInline` if the data is not stored inline or
    /// `JlrsError::WrongType` if the type of the elements is incorrect.
    pub fn inline_data<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<InlineArrayData<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
    {
        if !self.is_inline_array() {
            Err(JlrsError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe { Ok(InlineArrayData::new(self.as_array(), frame)) }
    }

    /// Mutably borrow inline array data, you can mutably borrow a single array at the same time.
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn inline_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<InlineArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        'borrow: 'data,
        F: Frame<'frame>,
    {
        if !self.is_inline_array() {
            Err(JlrsError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(InlineArrayDataMut::new(self.as_array(), frame))
    }

    /// Mutably borrow inline array data without the restriction that only a single array can be
    /// mutably borrowed.
    ///
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
    ///
    /// Safety: It's your responsibility to ensure you don't create multiple mutable references to
    /// the same data. Mutating Julia data is generally unsafe because it can't be guaranteed
    /// mutating this value is allowed.
    pub unsafe fn unrestricted_inline_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedInlineArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
    {
        if !self.is_inline_array() {
            Err(JlrsError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(UnrestrictedInlineArrayDataMut::new(self.as_array(), frame))
    }

    /// Convert `self` to `Array`.
    pub fn as_array(self) -> Array<'scope, 'data> {
        unsafe { Array::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }
}

impl<'scope, 'data, T: WrapperRef<'scope, 'data> + ValidLayout> TypedArray<'scope, 'data, T> {
    /// Immutably borrow the data of this array as an array of values, you can borrow data
    /// from multiple arrays at the same time.
    pub fn value_data<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> ValueArrayData<'borrow, 'scope, 'data>
    where
        F: Frame<'frame>,
    {
        unsafe { ValueArrayData::new(self.as_array(), frame) }
    }

    /// Immutably borrow the data of this array of wrappers, you can borrow data from multiple
    /// arrays at the same time.
    pub fn wrapper_data<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> ValueArrayData<'borrow, 'scope, 'data, T>
    where
        F: Frame<'frame>,
    {
        unsafe { ValueArrayData::new(self.as_array(), frame) }
    }

    /// Mutably borrow the data of this array as an array of values, you can mutably borrow a
    /// single array at the same time.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn value_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> ValueArrayDataMut<'borrow, 'scope, 'data>
    where
        F: Frame<'frame>,
    {
        ValueArrayDataMut::new(self.as_array(), frame)
    }

    /// Mutably borrow the data of this array of wrappers, you can mutably borrow a single array
    /// at the same time.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn wrapper_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> ValueArrayDataMut<'borrow, 'scope, 'data, T>
    where
        F: Frame<'frame>,
    {
        ValueArrayDataMut::new(self.as_array(), frame)
    }

    /// Mutably borrow the data of this array as an array of values without the restriction that
    /// only a single array can be mutably borrowed.
    ///
    /// Safety: It's your responsibility to ensure you don't create multiple mutable references to
    /// the same data. Mutating Julia data is generally unsafe because it can't be guaranteed
    /// mutating this value is allowed.
    pub unsafe fn unrestricted_value_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> UnrestrictedValueArrayDataMut<'borrow, 'scope, 'data>
    where
        F: Frame<'frame>,
    {
        UnrestrictedValueArrayDataMut::new(
            Array::wrap_non_null(self.unwrap_non_null(Private), Private),
            frame,
        )
    }

    /// Mutably borrow the data of this array of wrappers without the restriction that only a
    /// single array can be mutably borrowed.
    ///
    /// Safety: It's your responsibility to ensure you don't create multiple mutable references to
    /// the same data. Mutating Julia data is generally unsafe because it can't be guaranteed
    /// mutating this value is allowed.
    pub unsafe fn unrestricted_wrapper_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> UnrestrictedValueArrayDataMut<'borrow, 'scope, 'data, T>
    where
        F: Frame<'frame>,
    {
        UnrestrictedValueArrayDataMut::new(
            Array::wrap_non_null(self.unwrap_non_null(Private), Private),
            frame,
        )
    }
}

impl<'scope, T> TypedArray<'scope, 'static, T>
where
    T: Clone + ValidLayout + Debug,
{
    /// Reshape the array, a new array is returned that has dimensions `dims`. This new array and
    /// `self` share their data. This method returns an exception if the old and new array have a
    /// different number of elements or if the array contains data that has been borrowed or moved
    /// from Rust.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn reshape<'target: 'current, 'current, 'borrow, D, S, F>(
        self,
        scope: S,
        dims: D,
    ) -> JlrsResult<JuliaResult<'target, 'static>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        self.as_array().reshape(scope, dims)
    }

    /// Inserts `inc` more elements at the end of the array. The array must be 1D and
    /// contain no data borrowed or moved from Rust, otherwise an exception is returned.
    /// Depending on the type of the array elements the newly added elements will either be
    /// left uninitialized, or their contents will be set to 0s. It's set to 0s if
    /// `DataType::zero_init` returns true, if the elements are stored as pointers to Julia data,
    /// or if the elements contain pointers to Julia data.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn grow_end<'current, F>(
        self,
        frame: &mut F,
        inc: usize,
    ) -> JlrsResult<JuliaResult<'current, 'static, ()>>
    where
        F: Frame<'current>,
    {
        self.as_array().grow_end(frame, inc)
    }

    /// Removes the final `dec` elements from the array. The array must be 1D and contain no data
    /// borrowed or moved from Rust, otherwise an exception is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn del_end<'current, F>(
        self,
        frame: &mut F,
        dec: usize,
    ) -> JlrsResult<JuliaResult<'current, 'static, ()>>
    where
        F: Frame<'current>,
    {
        self.as_array().del_end(frame, dec)
    }

    /// Inserts `inc` more elements at the beginning of the array. The array must be 1D and
    /// contain no data borrowed or moved from Rust, otherwise an exception is returned.
    /// Depending on the type of the array elements the newly added elements will either be
    /// left uninitialized, or their contents will be set to 0s. It's set to 0s if
    /// `DataType::zero_init` returns true, if the elements are stored as pointers to Julia data,
    /// or if the elements contain pointers to Julia data.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn grow_begin<'current, F>(
        self,
        frame: &mut F,
        inc: usize,
    ) -> JlrsResult<JuliaResult<'current, 'static, ()>>
    where
        F: Frame<'current>,
    {
        self.as_array().grow_begin(frame, inc)
    }

    /// Removes the first `dec` elements from the array. The array must be 1D and contain no data
    /// borrowed or moved from Rust, otherwise an exception is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn del_begin<'current, F>(
        self,
        frame: &mut F,
        dec: usize,
    ) -> JlrsResult<JuliaResult<'current, 'static, ()>>
    where
        F: Frame<'current>,
    {
        self.as_array().del_begin(frame, dec)
    }
}

unsafe impl<'scope, 'data, T: Clone + ValidLayout + Debug> Typecheck
    for TypedArray<'scope, 'data, T>
{
    fn typecheck(t: DataType) -> bool {
        unsafe {
            t.is::<Array>()
                && T::valid_layout(
                    t.parameters().wrapper_unchecked().data_unchecked()[0].as_value(),
                )
        }
    }
}

impl<T: Clone + ValidLayout + Debug> Debug for TypedArray<'_, '_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.display_string() {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "<Cannot display value: {}>", e),
        }
    }
}

impl<'scope, 'data, T: Clone + ValidLayout + Debug> WrapperPriv<'scope, 'data>
    for TypedArray<'scope, 'data, T>
{
    type Wraps = jl_array_t;
    const NAME: &'static str = "Array";

    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

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

unsafe fn small_dim_tuple<'scope, D, F>(
    frame: &mut F,
    dims: D,
) -> JlrsResult<Value<'scope, 'static>>
where
    D: Dims,
    F: Frame<'scope>,
{
    let n = dims.n_dimensions();
    debug_assert!(n <= 8, "Too many dimensions for small_dim_tuple");
    let elem_types = JL_LONG_TYPE.with(|longs| longs.get());
    let tuple_type = jl_apply_tuple_type_v(elem_types.cast(), n);
    let tuple = jl_new_struct_uninit(tuple_type);
    let dims = dims.into_dimensions();
    let v: Value = frame
        .push_root(NonNull::new_unchecked(tuple), Private)
        .map_err(JlrsError::alloc_error)?;

    let usize_ptr: *mut usize = v.unwrap(Private).cast();
    std::ptr::copy_nonoverlapping(dims.as_slice().as_ptr(), usize_ptr, n);

    Ok(v)
}

unsafe fn large_dim_tuple<'scope, D, F>(
    frame: &mut F,
    dims: D,
) -> JlrsResult<Value<'scope, 'static>>
where
    D: Dims,
    F: Frame<'scope>,
{
    let n = dims.n_dimensions();
    let global = Global::new();
    let mut elem_types = vec![usize::julia_type(global); n];
    let tuple_type = jl_apply_tuple_type_v(elem_types.as_mut_ptr().cast(), n);
    let tuple = jl_new_struct_uninit(tuple_type);
    let v: Value = frame
        .push_root(NonNull::new_unchecked(tuple), Private)
        .map_err(JlrsError::alloc_error)?;

    let usize_ptr: *mut usize = v.unwrap(Private).cast();
    let dims = dims.into_dimensions();
    std::ptr::copy_nonoverlapping(dims.as_slice().as_ptr(), usize_ptr, n);

    Ok(v)
}

unsafe extern "C" fn droparray<T>(a: Array) {
    // The data of a moved array is allocated by Rust, this function is called by
    // a finalizer in order to ensure it's also freed by Rust.
    let mut arr_nn_ptr = a.unwrap_non_null(Private);
    let arr_ref = arr_nn_ptr.as_mut();

    if arr_ref.flags.how() != 2 {
        return;
    }

    // Set data to null pointer
    let data_ptr = arr_ref.data.cast::<T>();
    arr_ref.data = null_mut();

    // Set all dims to 0
    let arr_ptr = arr_nn_ptr.as_ptr();
    let dims_ptr = jl_array_dims_ptr(arr_ptr);
    let n_dims = jl_array_ndims(arr_ptr);
    for dim in slice::from_raw_parts_mut(dims_ptr, n_dims as _) {
        *dim = 0;
    }

    // Drop the data
    let data = Vec::from_raw_parts(data_ptr, arr_ref.length, arr_ref.length);
    mem::drop(data);
}

impl_root!(Array, 2);
impl<'target, 'value, 'data, T> crate::wrappers::ptr::Root<'target, 'value, 'data>
    for TypedArray<'value, 'data, T>
where
    T: Clone + Debug + ValidLayout,
{
    type Output = TypedArray<'target, 'data, T>;
    unsafe fn root<S>(
        scope: S,
        value: crate::wrappers::ptr::Ref<'value, 'data, Self>,
    ) -> crate::error::JlrsResult<Self::Output>
    where
        S: crate::memory::scope::PartialScope<'target>,
    {
        if let Some(v) = Self::wrapper(value, Private) {
            let ptr = v.unwrap_non_null(Private);
            scope.value(ptr, Private)
        } else {
            Err(crate::error::JlrsError::UndefRef)?
        }
    }
}
