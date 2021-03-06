//! Wrappers for `Array`, create and access n-dimensional Julia arrays from Rust.
//!
//! You will find two wrappers in this module that can be used to work with Julia arrays from
//! Rust. An [`Array`] is the Julia array itself, [`TypedArray`] is also available which can be
//! used if the element type implements [`ValidLayout`].
//!
//! Several methods are available to create new arrays. [`Array::new`] and [`TypedArray::new`]
//! let you create a new array for any type that implements [`IntoJulia`], while
//! [`Array::new_for`] can be used to create a new array for arbitrary types. These methods
//! allocate a new array, it's also possible to use data from Rust directly if it implements
//! `IntoJulia`. [`Array::from_vec`] and [`TypedArray::from_vec`] can be used to move the data
//! from Rust to Julia, while [`Array::from_slice`] and [`TypedArray::from_slice`] can be used
//! to mutably borrow data from Rust as a Julia array.
//!
//! How the contents of the array must be accessed from Rust depends on the type of the elements.
//! [`Array`] provides methods to (mutably) access their contents for all three possible
//! "layouts" of the elements: inline, pointer, and bits union.
//!
//! Accessing the contents of an array requires an n-dimensional index. The [`Dims`] trait is
//! available for this purpose. This trait is implemented for tuples of four or fewer `usize`s;
//! `[usize; N]` and `&[usize; N]` implement it for all `N`, `&[usize]` can be used if `N` is not
//! a constant at compile time.

use crate::{
    convert::into_julia::IntoJulia,
    error::{JlrsError, JlrsResult, CANNOT_DISPLAY_TYPE},
    impl_debug,
    layout::{typecheck::Typecheck, valid_layout::ValidLayout},
    memory::{
        frame::private::Frame as _,
        frame::Frame,
        global::Global,
        output::{OutputResult, OutputValue},
        scope::private::Scope as _,
        scope::Scope,
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
        module::Module,
        private::Wrapper as WrapperPriv,
        union::Union,
        value::Value,
        Wrapper,
    },
};
use jl_sys::{
    jl_alloc_array_1d, jl_alloc_array_2d, jl_alloc_array_3d, jl_apply_array_type,
    jl_apply_tuple_type_v, jl_array_data, jl_array_eltype, jl_array_t, jl_datatype_t,
    jl_gc_add_finalizer, jl_is_array_type, jl_new_array, jl_new_struct_uninit, jl_ptr_to_array,
    jl_ptr_to_array_1d, jl_tparam0, jlrs_alloc_array_1d, jlrs_alloc_array_2d, jlrs_alloc_array_3d,
    jlrs_new_array, jlrs_result_tag_t_JLRS_RESULT_ERR,
};
use std::{
    cell::UnsafeCell,
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

pub mod data;
pub mod dimensions;

/// An n-dimensional Julia array. It can be used in combination with [`DataType::is`] and
/// [`Value::is`], if the check returns `true` the [`Value`] can be cast to `Array`:
///
/// ```
/// # use jlrs::prelude::*;
/// # use jlrs::util::JULIA;
/// # fn main() {
/// # JULIA.with(|j| {
/// # let mut julia = j.borrow_mut();
/// julia.scope(|_global, frame| {
///     let arr = Array::new::<f64, _, _, _>(&mut *frame, (3, 3))?
///         .into_jlrs_result()?;
///
///     assert!(arr.is::<Array>());
///     assert!(arr.cast::<Array>().is_ok());
///     Ok(())
/// }).unwrap();
/// # });
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
    pub fn new<'target, 'current, T, D, S, F>(scope: S, dims: D) -> JlrsResult<S::JuliaResult>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            scope.result_scope_with_slots(2, |_, frame| {
                let global = frame.global();
                let elty_ptr = T::julia_type(global).ptr();
                let array_type = jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());
                (&mut *frame).value(NonNull::new_unchecked(array_type), Private)?;

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

                if array.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                    Ok(OutputResult::Err(OutputValue::wrap_non_null(
                        NonNull::new_unchecked(array.data),
                    )))
                } else {
                    Ok(OutputResult::Ok(OutputValue::wrap_non_null(
                        NonNull::new_unchecked(array.data),
                    )))
                }
            })
        }
    }

    /// Allocates a new n-dimensional array in Julia of dimensions `dims`. If `dims = (4, 2)` a
    /// two-dimensional array with 4 rows and 2 columns is created. This method can only be used
    /// in combination with types that implement `IntoJulia`. If you want to create an array for a
    /// type that doesn't implement this trait you must use [`Array::new_for`].
    ///
    /// If the array size is too large, the process will abort.
    pub fn new_unchecked<'target, 'current, T, D, S, F>(scope: S, dims: D) -> JlrsResult<S::Value>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            let global = scope.global();
            let elty_ptr = T::julia_type(global).ptr();
            let array_type = jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());

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
                    let tuple = small_dim_tuple(frame, dims)?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(
                            jl_new_array(array_type, tuple.unwrap(Private)).cast(),
                        ),
                        Private,
                    )
                }),
                _ => scope.value_scope_with_slots(1, |output, frame| {
                    let tuple = large_dim_tuple(frame, dims)?;
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
    /// `Union`, `UnionAll` or `DataType`, and dimensions `dims`. If `dims = (4, 2)` a
    /// two-dimensional array with 4 rows and 2 columns is created. If an exception is thrown due
    /// to either the type or dimensions being invalid it's caught and returned.
    pub fn new_for<'target, 'current, D, S, F>(
        scope: S,
        dims: D,
        ty: Value,
    ) -> JlrsResult<S::JuliaResult>
    where
        D: Dims,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            scope.result_scope_with_slots(2, |_, frame| {
                let array_type = jl_apply_array_type(ty.unwrap(Private), dims.n_dimensions());
                (&mut *frame).value(NonNull::new_unchecked(array_type), Private)?;

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

                if array.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                    Ok(OutputResult::Err(OutputValue::wrap_non_null(
                        NonNull::new_unchecked(array.data),
                    )))
                } else {
                    Ok(OutputResult::Ok(OutputValue::wrap_non_null(
                        NonNull::new_unchecked(array.data),
                    )))
                }
            })
        }
    }

    /// Allocates a new n-dimensional array in Julia for elements of type `ty`, which must be a
    /// `Union`, `UnionAll` or `DataType`, and dimensions `dims`. If `dims = (4, 2)` a
    /// two-dimensional array with 4 rows and 2 columns is created. If an exception is thrown due
    /// to either the type or dimensions being invalid the process aborts.
    pub fn new_for_unchecked<'target, 'current, D, S, F>(
        scope: S,
        dims: D,
        ty: Value,
    ) -> JlrsResult<S::Value>
    where
        D: Dims,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
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
                    let tuple = small_dim_tuple(frame, dims)?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(
                            jl_new_array(array_type, tuple.unwrap(Private)).cast(),
                        ),
                        Private,
                    )
                }),
                _ => scope.value_scope_with_slots(1, |output, frame| {
                    let tuple = large_dim_tuple(frame, dims)?;
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

    /// Borrows an n-dimensional array from Rust for use in Julia with dimensions `dims`. If
    /// `dims` = (4, 2)` a two-dimensional array with 4 rows and 2 columns is created.
    pub fn from_slice<'target, 'current, T, D, S, F>(
        scope: S,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<S::Value>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, 'data, F>,
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
                n if n <= 8 => scope.value_scope_with_slots(1, |output, frame| {
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
                }),
                _ => scope.value_scope_with_slots(1, |output, frame| {
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
                }),
            }
        }
    }

    /// Moves an n-dimensional array from Rust for use in Julia with dimensions `dims`. If
    /// `dims = (4, 2)` a two-dimensional array with 4 rows and 2 columns is created.
    pub fn from_vec<'target, 'current, T, D, S, F>(
        scope: S,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<S::Value>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, 'static, F>,
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
                        let tuple = small_dim_tuple(frame, dims)?;
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
                        let tuple = large_dim_tuple(frame, dims)?;
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
}

impl<'scope, 'data> Array<'scope, 'data> {
    /// Returns the array's dimensions.
    pub fn dimensions(self) -> ArrayDimensions<'scope> {
        unsafe { ArrayDimensions::new(self) }
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

    /// Returns true if the elements of the array are stored as [`Value`]s.
    pub fn is_value_array(self) -> bool {
        !self.is_inline_array()
    }

    /// Convert this untyped array to a [`TypedArray`].
    pub fn as_typed_array<T>(self) -> JlrsResult<TypedArray<'scope, 'data, T>>
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

    /// Copy the data of an inline array to Rust. Returns `JlrsError::NotInline` if the data is
    /// not stored inline or `JlrsError::WrongType` if the type of the elements is incorrect.
    pub fn copy_inline_data<T>(self) -> JlrsResult<CopiedArray<T>>
    where
        T: ValidLayout,
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

            Ok(CopiedArray::new(data, dimensions))
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
    pub fn inline_data_mut<'borrow, 'frame, T, F>(
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

        unsafe { Ok(InlineArrayDataMut::new(self, frame)) }
    }

    /// Mutably borrow inline array data without the restriction that only a single array can be
    /// mutably borrowed. It's your responsibility to ensure you don't create multiple mutable
    /// references to the same array data.
    ///
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
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
        unsafe {
            if !self.is_value_array() {
                Err(JlrsError::Inline {
                    element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
                })?;
            }

            Ok(ValueArrayData::new(self, frame))
        }
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
        T: Wrapper<'scope, 'data>,
        T::Ref: ValidLayout,
    {
        unsafe {
            if !self.contains::<T::Ref>() {
                Err(JlrsError::WrongType {
                    value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
                })?;
            }

            Ok(ValueArrayData::new(self, frame))
        }
    }

    /// Mutably borrow the data of this array of values, you can mutably borrow a single array at
    /// the same time. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn value_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ValueArrayDataMut<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        unsafe {
            if !self.is_value_array() {
                Err(JlrsError::Inline {
                    element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
                })?;
            }

            Ok(ValueArrayDataMut::new(self, frame))
        }
    }

    /// Mutably borrow the data of this array of wrappers, you can mutably borrow a single array
    /// at the same time. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn wrapper_data_mut<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ValueArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
        T: Wrapper<'scope, 'data>,
        T::Ref: ValidLayout,
    {
        unsafe {
            if !self.contains::<T::Ref>() {
                Err(JlrsError::WrongType {
                    value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
                })?;
            }

            Ok(ValueArrayDataMut::new(self, frame))
        }
    }

    /// Mutably borrow the data of this array of values without the restriction that only a single
    /// array can be mutably borrowed. It's your responsibility to ensure you don't create
    /// multiple mutable references to the same array data. Returns `JlrsError::Inline` if the
    /// data is stored inline.
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
    /// single array can be mutably borrowed. It's your responsibility to ensure you don't create
    /// multiple mutable references to the same array data. Returns `JlrsError::Inline` if the
    /// data is stored inline.
    pub unsafe fn unrestricted_wrapper_data_mut<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedValueArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
        T: Wrapper<'scope, 'data>,
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
            Ok(UnionArrayData::new(self, frame))
        } else {
            let elem_ty = self.element_type().display_string_or(CANNOT_DISPLAY_TYPE);
            let inline = !self.is_value_array();
            Err(JlrsError::NotAUnionArray { elem_ty, inline })?
        }
    }

    /// Mutably borrow the data of this array of bits-unions, you can mutably borrow a single
    /// array at a time.
    pub fn union_data_mut<'borrow, 'frame, F>(
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
    /// single array can be mutably borrowed. It's your responsibility to ensure you don't create
    /// multiple mutable references to the same array data.
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

unsafe impl<'scope, 'data> Typecheck for Array<'scope, 'data> {
    fn typecheck(t: DataType) -> bool {
        unsafe { jl_is_array_type(t.unwrap(Private).cast()) }
    }
}

unsafe impl<'scope, 'data> ValidLayout for Array<'scope, 'data> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<Array>()
        } else if let Ok(ua) = v.cast::<super::union_all::UnionAll>() {
            unsafe { ua.base_type().wrapper_unchecked().is::<Array>() }
        } else {
            false
        }
    }
}

impl_debug!(Array<'_, '_>);

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Array<'scope, 'data> {
    type Internal = jl_array_t;
    const NAME: &'static str = "Array";

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
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

impl<'scope, 'data, T> TypedArray<'scope, 'data, T>
where
    T: Clone + ValidLayout + Debug + IntoJulia,
{
    /// Allocates a new n-dimensional array in Julia of dimensions `dims`. If `dims = (4, 2)` a
    /// two-dimensional array with 4 rows and 2 columns is created. This method can only be used
    /// in combination with types that implement `IntoJulia`. If you want to create an array for a
    /// type that doesn't implement this trait you must use [`Array::new_for`].
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    pub fn new<'target, 'current, D, S, F>(scope: S, dims: D) -> JlrsResult<S::JuliaResult>
    where
        D: Dims,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            scope.result_scope_with_slots(2, |_, frame| {
                let global = frame.global();
                let elty_ptr = T::julia_type(global).ptr();
                let array_type = jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());
                (&mut *frame).value(NonNull::new_unchecked(array_type), Private)?;

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

                if array.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                    Ok(OutputResult::Err(OutputValue::wrap_non_null(
                        NonNull::new_unchecked(array.data),
                    )))
                } else {
                    Ok(OutputResult::Ok(OutputValue::wrap_non_null(
                        NonNull::new_unchecked(array.data),
                    )))
                }
            })
        }
    }

    /// Allocates a new n-dimensional array in Julia of dimensions `dims`. If `dims = (4, 2)` a
    /// two-dimensional array with 4 rows and 2 columns is created. This method can only be used
    /// in combination with types that implement `IntoJulia`. If you want to create an array for a
    /// type that doesn't implement this trait you must use [`Array::new_for`].
    ///
    /// If the array size is too large, the process will abort.
    pub fn new_unchecked<'target, 'current, D, S, F>(scope: S, dims: D) -> JlrsResult<S::Value>
    where
        D: Dims,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            let global = scope.global();
            let elty_ptr = T::julia_type(global).ptr();
            let array_type = jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());

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
                    let tuple = small_dim_tuple(frame, dims)?;
                    output.into_scope(frame).value(
                        NonNull::new_unchecked(
                            jl_new_array(array_type, tuple.unwrap(Private)).cast(),
                        ),
                        Private,
                    )
                }),
                _ => scope.value_scope_with_slots(1, |output, frame| {
                    let tuple = large_dim_tuple(frame, dims)?;
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

    /// Borrows an n-dimensional array from Rust for use in Julia with dimensions `dims`. If
    /// `dims = (4, 2)` a two-dimensional array with 4 rows and 2 columns is created.
    pub fn from_slice<'target, 'current, D, S, F>(
        scope: S,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<S::Value>
    where
        D: Dims,
        S: Scope<'target, 'current, 'data, F>,
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
                n if n <= 8 => scope.value_scope_with_slots(1, |output, frame| {
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
                }),
                _ => scope.value_scope_with_slots(1, |output, frame| {
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
                }),
            }
        }
    }

    /// Moves an n-dimensional array from Rust for use in Julia with dimensions `dims`. If
    /// `dims = (4, 2)` a two-dimensional array with 4 rows and 2 columns is created.
    pub fn from_vec<'target, 'current, D, S, F>(
        scope: S,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<S::Value>
    where
        D: Dims,
        S: Scope<'target, 'current, 'static, F>,
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
                        let tuple = small_dim_tuple(frame, dims)?;
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
                        let tuple = large_dim_tuple(frame, dims)?;
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
}

impl<'scope, 'data, T: Clone + ValidLayout + Debug> TypedArray<'scope, 'data, T> {
    /// Returns the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'scope> {
        unsafe { ArrayDimensions::new(self.as_array()) }
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
    /// not stored inline or `JlrsError::WrongType` if the type of the elements is incorrect.
    pub fn copy_inline_data(self) -> JlrsResult<CopiedArray<T>> {
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

            Ok(CopiedArray::new(data, dimensions))
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
    pub fn inline_data_mut<'borrow, 'frame, F>(
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

        unsafe { Ok(InlineArrayDataMut::new(self.as_array(), frame)) }
    }

    /// Mutably borrow inline array data without the restriction that only a single array can be
    /// mutably borrowed. It's your responsibility to ensure you don't create multiple mutable
    /// references to the same array data.
    ///
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
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

impl<'scope, 'data, T: Wrapper<'scope, 'data> + ValidLayout> TypedArray<'scope, 'data, T> {
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
    pub fn value_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> ValueArrayDataMut<'borrow, 'scope, 'data>
    where
        F: Frame<'frame>,
    {
        unsafe { ValueArrayDataMut::new(self.as_array(), frame) }
    }

    /// Mutably borrow the data of this array of wrappers, you can mutably borrow a single array
    /// at the same time.
    pub fn wrapper_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> ValueArrayDataMut<'borrow, 'scope, 'data, T>
    where
        F: Frame<'frame>,
    {
        unsafe { ValueArrayDataMut::new(self.as_array(), frame) }
    }

    /// Mutably borrow the data of this array as an array of values without the restriction that
    /// only a single array can be mutably borrowed. It's your responsibility to ensure you don't
    /// create multiple mutable references to the same array data.
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
    /// single array can be mutably borrowed. It's your responsibility to ensure you don't create
    /// multiple mutable references to the same array data.
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

unsafe impl<'scope, 'data, T: Clone + ValidLayout + Debug> Typecheck
    for TypedArray<'scope, 'data, T>
{
    fn typecheck(t: DataType) -> bool {
        unsafe {
            jl_is_array_type(t.unwrap(Private).cast())
                && T::valid_layout(Value::wrap(jl_tparam0(t.unwrap(Private)).cast(), Private))
        }
    }
}

unsafe impl<'scope, 'data, T: Clone + ValidLayout + Debug> ValidLayout
    for TypedArray<'scope, 'data, T>
{
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<TypedArray<T>>()
        } else if let Ok(ua) = v.cast::<super::union_all::UnionAll>() {
            unsafe { ua.base_type().wrapper_unchecked().is::<TypedArray<T>>() }
        } else {
            false
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
    type Internal = jl_array_t;
    const NAME: &'static str = "Array";

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
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
    let v = frame
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
    let global = frame.global();
    let mut elem_types = vec![usize::julia_type(global); n];
    let tuple_type = jl_apply_tuple_type_v(elem_types.as_mut_ptr().cast(), n);
    let tuple = jl_new_struct_uninit(tuple_type);
    let v = frame
        .push_root(NonNull::new_unchecked(tuple), Private)
        .map_err(JlrsError::alloc_error)?;

    let usize_ptr: *mut usize = v.unwrap(Private).cast();
    let dims = dims.into_dimensions();
    std::ptr::copy_nonoverlapping(dims.as_slice().as_ptr(), usize_ptr, n);

    Ok(v)
}
