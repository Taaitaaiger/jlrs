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

use crate::{
    convert::into_julia::IntoJulia,
    error::{AccessError, ArrayLayoutError, InstantiationError, JlrsResult, CANNOT_DISPLAY_TYPE},
    layout::{typecheck::Typecheck, valid_layout::ValidLayout},
    memory::{
        frame::{Frame, GcFrame},
        get_tls,
        global::Global,
        output::Output,
        scope::{private::PartialScopePriv, PartialScope, Scope},
    },
    private::Private,
    wrappers::ptr::{
        array::{
            data::copied::CopiedArray,
            dimensions::{ArrayDimensions, Dims},
        },
        datatype::DataType,
        private::WrapperPriv,
        type_name::TypeName,
        union::Union,
        value::Value,
        Wrapper, WrapperRef,
    },
};
use cfg_if::cfg_if;
use jl_sys::{
    jl_alloc_array_1d, jl_alloc_array_2d, jl_alloc_array_3d, jl_apply_array_type,
    jl_apply_tuple_type_v, jl_array_data, jl_array_del_beg, jl_array_del_end, jl_array_dims_ptr,
    jl_array_eltype, jl_array_grow_beg, jl_array_grow_end, jl_array_ndims, jl_array_t,
    jl_datatype_t, jl_gc_add_ptr_finalizer, jl_new_array, jl_new_struct_uninit, jl_pchar_to_array,
    jl_ptr_to_array, jl_ptr_to_array_1d, jl_reshape_array,
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

use self::data::accessor::{
    ArrayAccessor, BitsArrayAccessorI, BitsArrayAccessorMut, Immutable, IndeterminateArrayAccessor,
    IndeterminateArrayAccessorI, InlinePtrArrayAccessorI, InlinePtrArrayAccessorMut, Mutable,
    PtrArrayAccessorI, PtrArrayAccessorMut, UnionArrayAccessorI, UnionArrayAccessorMut,
};

use super::{union_all::UnionAll, value::ValueRef, Ref, Root};

cfg_if! {
    if #[cfg(not(all(target_os = "windows", feature = "lts")))] {
        use crate::error::JuliaResult;
    }
}

pub mod data;
pub mod dimensions;
pub mod tracked;

/// An n-dimensional Julia array.
///
/// Each element in the backing storage is either stored as a [`Value`] or inline. If the inline
/// data is a bits union, the flag indicating the active variant is stored separately from the
/// elements. You can check how the data is stored by calling [`Array::is_value_array`],
/// [`Array::is_inline_array`], or [`Array::is_union_array`].
///
/// Arrays that contain integers or floats are examples of inline arrays. Their data is stored as
/// an array that contains numbers of the appropriate type, for example an array of `Float32`s in
/// Julia is backed by an an array of `f32`s. The data in these arrays can be accessed with
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
    PhantomData<&'data mut ()>,
);

impl<'data> Array<'_, 'data> {
    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. If you
    /// want to create an array for a type that doesn't implement this trait you must use
    /// [`Array::new_for`].
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn new<'target, 'current, T, D, S, F>(
        scope: S,
        dims: D,
    ) -> JuliaResult<'target, 'static, Array<'target, 'static>>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        use crate::catch::catch_exceptions_with_slots;
        use std::mem::MaybeUninit;

        let (output, frame) = scope.split();
        frame
            .scope(|mut frame| {
                let global = frame.as_scope().global();
                let elty_ptr = T::julia_type(global).ptr();

                // Safety: The array type is rooted until the array has been constructed, all C API
                // functions are called with valid data.
                unsafe {
                    let mut callback =
                        |frame: &mut GcFrame, result: &mut MaybeUninit<*mut jl_array_t>| {
                            let array_type =
                                jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());
                            frame.push_root(NonNull::new_unchecked(array_type));

                            let array = match dims.n_dimensions() {
                                1 => jl_alloc_array_1d(array_type, dims.n_elements(0)),
                                2 => jl_alloc_array_2d(
                                    array_type,
                                    dims.n_elements(0),
                                    dims.n_elements(1),
                                ),
                                3 => jl_alloc_array_3d(
                                    array_type,
                                    dims.n_elements(0),
                                    dims.n_elements(1),
                                    dims.n_elements(2),
                                ),
                                n if n <= 8 => {
                                    let tuple = small_dim_tuple(frame, &dims);
                                    jl_new_array(array_type, tuple.unwrap(Private))
                                }
                                _ => {
                                    let tuple = large_dim_tuple(frame, &dims);
                                    jl_new_array(array_type, tuple.unwrap(Private))
                                }
                            };

                            result.write(array);
                            Ok(())
                        };

                    match catch_exceptions_with_slots(&mut frame, &mut callback).unwrap() {
                        Ok(array_ptr) => Ok(Ok(output.set_root(NonNull::new_unchecked(array_ptr)))),
                        Err(e) => Ok(Err(output.set_root(NonNull::new_unchecked(e.ptr())))),
                    }
                }
            })
            .unwrap()
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method is equivalent to [`Array::new`] except that Julia exceptions are not caught.
    ///
    /// Safety: If the array size is too large, Julia will throw an error. This error is not
    /// caught, which is UB from a `ccall`ed function.
    pub unsafe fn new_unchecked<'target, 'current, T, D, S, F>(
        scope: S,
        dims: D,
    ) -> Array<'target, 'static>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        let (output, frame) = scope.split();
        frame
            .scope(|mut frame| {
                let global = frame.as_scope().global();
                let elty_ptr = T::julia_type(global).ptr();
                let array_type = jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());
                let _: Value = frame
                    .as_scope()
                    .value(NonNull::new_unchecked(array_type), Private);

                let array = match dims.n_dimensions() {
                    1 => jl_alloc_array_1d(array_type, dims.n_elements(0)),
                    2 => jl_alloc_array_2d(array_type, dims.n_elements(0), dims.n_elements(1)),
                    3 => jl_alloc_array_3d(
                        array_type,
                        dims.n_elements(0),
                        dims.n_elements(1),
                        dims.n_elements(2),
                    ),
                    n if n <= 8 => {
                        let tuple = small_dim_tuple(&mut frame, &dims);
                        jl_new_array(array_type, tuple.unwrap(Private))
                    }
                    _ => {
                        let tuple = large_dim_tuple(&mut frame, &dims);
                        jl_new_array(array_type, tuple.unwrap(Private))
                    }
                };

                Ok(output.value(NonNull::new_unchecked(array), Private))
            })
            .unwrap()
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `ty`.
    ///
    /// The elementy type, ty` must be a` Union`, `UnionAll` or `DataType`.
    ///
    /// If the array size is too large or if the type is invalid, Julia will throw an error. This
    /// error is caught and returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn new_for<'target, 'current, D, S, F>(
        scope: S,
        dims: D,
        ty: Value,
    ) -> JuliaResult<'target, 'static, Array<'target, 'static>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        use crate::catch::catch_exceptions_with_slots;
        use std::mem::MaybeUninit;

        let (output, frame) = scope.split();
        frame
            .scope(|mut frame| {
                let elty_ptr = ty.unwrap(Private);
                // Safety: The array type is rooted until the array has been constructed, all C API
                // functions are called with valid data.
                unsafe {
                    let mut callback =
                        |frame: &mut GcFrame, result: &mut MaybeUninit<*mut jl_array_t>| {
                            let array_type =
                                jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());
                            frame.push_root(NonNull::new_unchecked(array_type));

                            let array = match dims.n_dimensions() {
                                1 => jl_alloc_array_1d(array_type, dims.n_elements(0)),
                                2 => jl_alloc_array_2d(
                                    array_type,
                                    dims.n_elements(0),
                                    dims.n_elements(1),
                                ),
                                3 => jl_alloc_array_3d(
                                    array_type,
                                    dims.n_elements(0),
                                    dims.n_elements(1),
                                    dims.n_elements(2),
                                ),
                                n if n <= 8 => {
                                    let tuple = small_dim_tuple(frame, &dims);
                                    jl_new_array(array_type, tuple.unwrap(Private))
                                }
                                _ => {
                                    let tuple = large_dim_tuple(frame, &dims);
                                    jl_new_array(array_type, tuple.unwrap(Private))
                                }
                            };

                            result.write(array);
                            Ok(())
                        };

                    match catch_exceptions_with_slots(&mut frame, &mut callback).unwrap() {
                        Ok(array_ptr) => Ok(Ok(output.set_root(NonNull::new_unchecked(array_ptr)))),
                        Err(e) => Ok(Err(output.set_root(NonNull::new_unchecked(e.ptr())))),
                    }
                }
            })
            .unwrap()
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method is equivalent to [`Array::new_for`] except that Julia exceptions are not
    /// caught.
    ///
    /// Safety: If the array size is too large or if the type is invalid, Julia will throw an
    /// error. This error is not caught, which is UB from a `ccall`ed function.
    pub unsafe fn new_for_unchecked<'target, 'current, D, S, F>(
        scope: S,
        dims: D,
        ty: Value,
    ) -> Array<'target, 'static>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        let (output, frame) = scope.split();
        frame
            .scope(|mut frame| {
                let array_type = jl_apply_array_type(ty.unwrap(Private), dims.n_dimensions());
                let _: Value = frame
                    .as_scope()
                    .value(NonNull::new_unchecked(array_type), Private);

                let array = match dims.n_dimensions() {
                    1 => jl_alloc_array_1d(array_type, dims.n_elements(0)),
                    2 => jl_alloc_array_2d(array_type, dims.n_elements(0), dims.n_elements(1)),
                    3 => jl_alloc_array_3d(
                        array_type,
                        dims.n_elements(0),
                        dims.n_elements(1),
                        dims.n_elements(2),
                    ),
                    n if n <= 8 => {
                        let tuple = small_dim_tuple(&mut frame, &dims);
                        jl_new_array(array_type, tuple.unwrap(Private))
                    }
                    _ => {
                        let tuple = large_dim_tuple(&mut frame, &dims);
                        jl_new_array(array_type, tuple.unwrap(Private))
                    }
                };

                Ok(output.set_root(NonNull::new_unchecked(array)))
            })
            .unwrap()
    }

    /// Create a new n-dimensional Julia array of dimensions `dims` that borrows data from Rust.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. Because
    /// the data is borrowed from Rust, operations that can change the size of the array (e.g.
    /// `push!`) will fail.
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn from_slice<'target, 'current, T, D, S, F>(
        scope: S,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<JuliaResult<'target, 'static, Array<'target, 'data>>>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        use std::mem::MaybeUninit;

        use crate::catch::catch_exceptions_with_slots;

        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let (output, frame) = scope.split();
        frame.scope(|mut frame| {
            let global = frame.as_scope().global();
            let elty_ptr = T::julia_type(global).ptr().cast();

            // Safety: The array type is rooted until the array has been constructed, all C API
            // functions are called with valid data. The data-lifetime ensures the data can't be
            // used from Rust after the borrow ends.
            unsafe {
                let mut callback =
                    |frame: &mut GcFrame, result: &mut MaybeUninit<*mut jl_array_t>| {
                        let array_type = jl_apply_array_type(elty_ptr, dims.n_dimensions());
                        frame
                            .as_scope()
                            .push_root(NonNull::new_unchecked(array_type));

                        let array = match dims.n_dimensions() {
                            1 => jl_ptr_to_array_1d(
                                array_type,
                                data.as_mut_ptr().cast(),
                                dims.n_elements(0),
                                0,
                            ),
                            n if n <= 8 => {
                                let tuple = small_dim_tuple(frame, &dims);
                                jl_ptr_to_array(
                                    array_type,
                                    data.as_mut_ptr().cast(),
                                    tuple.unwrap(Private),
                                    0,
                                )
                            }
                            _ => {
                                let tuple = large_dim_tuple(frame, &dims);
                                jl_ptr_to_array(
                                    array_type,
                                    data.as_mut_ptr().cast(),
                                    tuple.unwrap(Private),
                                    0,
                                )
                            }
                        };

                        result.write(array);
                        Ok(())
                    };

                match catch_exceptions_with_slots(&mut frame, &mut callback).unwrap() {
                    Ok(array_ptr) => Ok(Ok(output.set_root(NonNull::new_unchecked(array_ptr)))),
                    Err(e) => Ok(Err(e.root(output)?)),
                }
            }
        })
    }

    /// Create a new n-dimensional Julia array of dimensions `dims` that borrows data from Rust.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. Because
    /// the data is borrowed from Rust, operations that can change the size of the array (e.g.
    /// `push!`) will fail.
    ///
    /// Safety: If the array size is too large, Julia will throw an error. This error is not
    /// caught, which is UB from a `ccall`ed function.
    pub unsafe fn from_slice_unchecked<'target, 'current, T, D, S, F>(
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
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let (output, frame) = scope.split();
        frame.scope(|mut frame| {
            let global = frame.as_scope().global();
            let array_type =
                jl_apply_array_type(T::julia_type(global).ptr().cast(), dims.n_dimensions());
            frame
                .as_scope()
                .push_root(NonNull::new_unchecked(array_type));

            let array = match dims.n_dimensions() {
                1 => {
                    jl_ptr_to_array_1d(array_type, data.as_mut_ptr().cast(), dims.n_elements(0), 0)
                }
                n if n <= 8 => {
                    let tuple = small_dim_tuple(&mut frame, &dims);
                    jl_ptr_to_array(
                        array_type,
                        data.as_mut_ptr().cast(),
                        tuple.unwrap(Private),
                        0,
                    )
                }
                _ => {
                    let tuple = large_dim_tuple(&mut frame, &dims);
                    jl_ptr_to_array(
                        array_type,
                        data.as_mut_ptr().cast(),
                        tuple.unwrap(Private),
                        0,
                    )
                }
            };

            Ok(output.value(NonNull::new_unchecked(array), Private))
        })
    }

    /// Create a new n-dimensional Julia array of dimensions `dims` that takes ownership of Rust
    /// data.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. Because
    /// the data is allocated by Rust, operations that can change the size of the array (e.g.
    /// `push!`) will fail.
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn from_vec<'target, 'current, T, D, S, F>(
        scope: S,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<JuliaResult<'target, 'static, Array<'target, 'static>>>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        use std::mem::MaybeUninit;

        use crate::catch::catch_exceptions_with_slots;

        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let (output, scope) = scope.split();
        scope.scope(|mut frame| {
            let global = frame.as_scope().global();
            let elty_ptr = T::julia_type(global).ptr().cast();
            let data = Box::leak(data.into_boxed_slice());

            // Safety: The array type is rooted until the array has been constructed, all C API
            // functions are called with valid data. The data-lifetime ensures the data can't be
            // used from Rust after the borrow ends.
            unsafe {
                let mut callback =
                    |frame: &mut GcFrame, result: &mut MaybeUninit<*mut jl_array_t>| {
                        let array_type = jl_apply_array_type(elty_ptr, dims.n_dimensions());
                        frame
                            .as_scope()
                            .push_root(NonNull::new_unchecked(array_type));

                        let array = match dims.n_dimensions() {
                            1 => jl_ptr_to_array_1d(
                                array_type,
                                data.as_mut_ptr().cast(),
                                dims.n_elements(0),
                                1,
                            ),
                            n if n <= 8 => {
                                let tuple = small_dim_tuple(frame, &dims);
                                jl_ptr_to_array(
                                    array_type,
                                    data.as_mut_ptr().cast(),
                                    tuple.unwrap(Private),
                                    1,
                                )
                            }
                            _ => {
                                let tuple = large_dim_tuple(frame, &dims);
                                jl_ptr_to_array(
                                    array_type,
                                    data.as_mut_ptr().cast(),
                                    tuple.unwrap(Private),
                                    1,
                                )
                            }
                        };

                        jl_gc_add_ptr_finalizer(
                            get_tls(),
                            array.cast(),
                            droparray::<T> as *mut c_void,
                        );

                        result.write(array);
                        Ok(())
                    };

                match catch_exceptions_with_slots(&mut frame, &mut callback).unwrap() {
                    Ok(array_ptr) => Ok(Ok(output.set_root(NonNull::new_unchecked(array_ptr)))),
                    Err(e) => Ok(Err(output.set_root(NonNull::new_unchecked(e.ptr())))),
                }
            }
        })
    }

    /// Create a new n-dimensional Julia array of dimensions `dims` that takes ownership of Rust
    /// data.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. Because
    /// the data is allocated by Rust, operations that can change the size of the array (e.g.
    /// `push!`) will fail.
    ///
    /// Safety: If the array size is too large, Julia will throw an error. This error is not
    /// caught, which is UB from a `ccall`ed function.
    pub unsafe fn from_vec_unchecked<'target, 'current, T, D, S, F>(
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
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let (output, scope) = scope.split();
        scope.scope(|mut frame| {
            let global = frame.as_scope().global();
            let array_type =
                jl_apply_array_type(T::julia_type(global).ptr().cast(), dims.n_dimensions());
            frame
                .as_scope()
                .push_root(NonNull::new_unchecked(array_type));

            let array = match dims.n_dimensions() {
                1 => jl_ptr_to_array_1d(
                    array_type,
                    Box::into_raw(data.into_boxed_slice()).cast(),
                    dims.n_elements(0),
                    1,
                ),
                n if n <= 8 => {
                    let tuple = small_dim_tuple(&mut frame, &dims);
                    jl_ptr_to_array(
                        array_type,
                        Box::into_raw(data.into_boxed_slice()).cast(),
                        tuple.unwrap(Private),
                        1,
                    )
                }
                _ => {
                    let tuple = large_dim_tuple(&mut frame, &dims);
                    jl_ptr_to_array(
                        array_type,
                        Box::into_raw(data.into_boxed_slice()).cast(),
                        tuple.unwrap(Private),
                        1,
                    )
                }
            };

            jl_gc_add_ptr_finalizer(get_tls(), array.cast(), droparray::<T> as *mut c_void);
            Ok(output.value(NonNull::new_unchecked(array), Private))
        })
    }

    /// Convert a string to a Julia array.
    pub fn from_string<'target, A, S>(scope: S, data: A) -> Array<'target, 'static>
    where
        A: AsRef<str>,
        S: PartialScope<'target>,
    {
        let string = data.as_ref();
        let nbytes = string.bytes().len();
        let ptr = string.as_ptr();
        // Safety: a string can be converted to an array of bytes.
        unsafe {
            let arr = jl_pchar_to_array(ptr.cast(), nbytes);
            scope.value(NonNull::new_unchecked(arr), Private)
        }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> Array<'target, 'data> {
        let ptr = self.unwrap_non_null(Private);
        // Safety: the data is valid.
        unsafe {
            output.set_root::<Array>(ptr);
            Array::wrap_non_null(ptr, Private)
        }
    }

    #[inline(always)]
    pub(crate) fn data_ptr(self) -> *mut c_void {
        // Safety: the pointer points to valid data.
        unsafe { self.unwrap_non_null(Private).as_ref().data }
    }
}

impl<'scope, 'data> Array<'scope, 'data> {
    /// Returns the array's dimensions.
    pub unsafe fn dimensions(self) -> ArrayDimensions<'scope> {
        ArrayDimensions::new(self)
    }

    /// TODO: Rooted/unrooted
    /// Returns the type of this array's elements.
    pub fn element_type(self) -> ValueRef<'scope, 'static> {
        // Safety: C API function is called valid arguments.
        unsafe { ValueRef::wrap(jl_array_eltype(self.unwrap(Private).cast()).cast()) }
    }

    /// Returns the size of this array's elements.
    pub fn element_size(self) -> usize {
        // Safety: the pointer points to valid data.
        unsafe { self.unwrap_non_null(Private).as_ref().elsize as usize }
    }

    /// Returns `true` if the layout of the elements is compatible with `T`.
    pub fn contains<T: ValidLayout>(self) -> bool {
        // Safety: C API function is called valid arguments.
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

    /// Returns `true` if the elements of the array are stored inline.
    pub fn is_inline_array(self) -> bool {
        // Safety: the pointer points to valid data.
        unsafe { self.unwrap_non_null(Private).as_ref().flags.ptrarray() == 0 }
    }

    /// Returns `true` if the elements of the array are stored inline and the element type is a
    /// union type.
    pub fn is_union_array(self) -> bool {
        self.is_inline_array() && unsafe { self.element_type().value_unchecked().is::<Union>() }
    }

    /// Returns true if the elements of the array are stored inline and at least one of the fields
    /// of the inlined type is a pointer.
    pub fn has_inlined_pointers(self) -> bool {
        // Safety: the pointer points to valid data.
        unsafe {
            let flags = self.unwrap_non_null(Private).as_ref().flags;
            self.is_inline_array() && flags.hasptr() != 0
        }
    }

    /// Returns `true` if elements of this array are zero-initialized.
    pub fn zero_init(self) -> bool {
        // Safety: the pointer points to valid data.
        unsafe {
            let flags = self.unwrap_non_null(Private).as_ref().flags;
            if flags.ptrarray() == 1 || flags.hasptr() == 1 {
                return true;
            }

            let elty = self.element_type().value_unchecked();
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
        T: ValidLayout,
    {
        if self.contains::<T>() {
            let ptr = self.unwrap_non_null(Private);
            // Safety: the type is correct
            unsafe { Ok(TypedArray::wrap_non_null(ptr, Private)) }
        } else {
            let value_type_str = unsafe {
                self.element_type()
                    .value_unchecked()
                    .display_string_or(CANNOT_DISPLAY_TYPE)
            };
            Err(AccessError::InvalidLayout { value_type_str })?
        }
    }

    /// Convert this untyped array to a [`TypedArray`] without checking if this conversion is
    /// valid.
    ///
    /// Safety: `T` must be a valid representation of the data stored in the array.
    pub unsafe fn as_typed_unchecked<T>(self) -> TypedArray<'scope, 'data, T>
    where
        T: ValidLayout,
    {
        TypedArray::wrap_non_null(self.unwrap_non_null(Private), Private)
    }

    /// Copy the data of an inline array to Rust.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline or `AccessError::InvalidLayout`
    /// if the type of the elements is incorrect.
    pub unsafe fn copy_inline_data<T>(&self) -> JlrsResult<CopiedArray<T>>
    where
        T: 'static + ValidLayout,
    {
        self.ensure_bits_containing::<T>()?;

        let dimensions = self.dimensions().into_dimensions();
        let sz = dimensions.size();
        let mut data = Vec::with_capacity(sz);

        // Safety: layouts are compatible and is guaranteed to be a bits type due to the
        // 'static constraint on T.
        let jl_data = jl_array_data(self.unwrap(Private).cast()).cast();
        let ptr = data.as_mut_ptr();
        std::ptr::copy_nonoverlapping(jl_data, ptr, sz);
        data.set_len(sz);

        Ok(CopiedArray::new(data.into_boxed_slice(), dimensions))
    }

    pub unsafe fn bits_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<BitsArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.ensure_bits_containing::<T>()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self);
        Ok(accessor)
    }

    /// Mutably access the contents of this array. The elements must have an `isbits` type.
    ///
    /// This method can be used to gain mutable access to the contents of a single array.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline, `ArrayLayoutError::NotBits`
    /// if the type is not an `isbits` type, or `AccessError::InvalidLayout` if `T` is not a valid
    /// layout for the array elements.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn bits_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<BitsArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.ensure_bits_containing::<T>()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(accessor)
    }

    /// Immutably the contents of this array. The elements must be stored inline.
    ///
    /// You can borrow data from multiple arrays at the same time.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline or
    /// `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements.
    pub unsafe fn inline_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<InlinePtrArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.ensure_inline_containing::<T>()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(accessor)
    }

    /// Mutably access the contents of this array. The elements must be stored inline.
    ///
    /// This method can be used to gain mutable access to the contents of a single array.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline or
    /// `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn inline_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<InlinePtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.ensure_inline_containing::<T>()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(accessor)
    }

    /// Immutably the contents of this array. The elements must not be stored inline.
    ///
    /// You can borrow data from multiple arrays at the same time.
    ///
    /// Returns `ArrayLayoutError::NotPointer` if the data is stored inline or `AccessError::InvalidLayout` if `T`
    /// is not a valid layout for the array elements.
    pub unsafe fn wrapper_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: WrapperRef<'scope, 'data>,
    {
        self.ensure_ptr_containing::<T>()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(accessor)
    }

    /// Mutably access the contents of this array. The elements must not be stored inline.
    ///
    /// This method can be used to gain mutable access to the contents of a single array.
    ///
    /// Returns `ArrayLayoutError::NotPointer` if the data is stored inline or `AccessError::InvalidLayout` if `T`
    /// is not a valid layout for the array elements.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn wrapper_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: WrapperRef<'scope, 'data>,
    {
        self.ensure_ptr_containing::<T>()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(accessor)
    }

    /// Immutably the contents of this array. The elements must not be stored inline.
    ///
    /// You can borrow data from multiple arrays at the same time.
    ///
    /// Returns `ArrayLayoutError::NotPointer` if the data is stored inline.
    pub unsafe fn value_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        self.ensure_ptr_containing::<ValueRef>()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(accessor)
    }

    /// Mutably access the contents of this array. The elements must not be stored inline.
    ///
    /// This method can be used to gain mutable access to the contents of a single array.
    ///
    /// Returns `ArrayLayoutError::NotPointer` if the data is stored inline.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn value_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        self.ensure_ptr_containing::<ValueRef>()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(accessor)
    }

    /// Immutably access the contents of this array. The element type must be a bits union type.
    ///
    /// You can borrow data from multiple arrays at the same time.
    ///
    /// Returns `ArrayLayoutError::NotUnion` if the data is not stored as a bits union.
    pub unsafe fn union_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<UnionArrayAccessorI<'borrow, 'scope, 'data>> {
        self.ensure_union()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(accessor)
    }

    /// Mutably access the contents of this array. The element type must be a bits union.
    ///
    /// This method can be used to gain mutable access to the contents of a single array.
    ///
    /// Returns `ArrayLayoutError::NotUnion` if the data is not stored as a bits union.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn union_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<UnionArrayAccessorMut<'borrow, 'scope, 'data>> {
        self.ensure_union()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(accessor)
    }

    /// Immutably access the contents of this array.
    ///
    /// You can borrow data from multiple arrays at the same time.
    pub unsafe fn indeterminate_data<'borrow>(
        &'borrow self,
    ) -> IndeterminateArrayAccessorI<'borrow, 'scope, 'data> {
        ArrayAccessor::new2(self)
    }

    /// Mutably access the contents of this array.
    ///
    /// This method can be used to gain mutable access to the contents of a single array.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn indeterminate_data_mut<'borrow>(
        &'borrow mut self,
    ) -> IndeterminateArrayAccessor<'borrow, 'scope, 'data, Mutable<'borrow, u8>> {
        ArrayAccessor::new2(self)
    }

    /// Reshape the array, a new array is returned that has dimensions `dims`. The new array and
    /// `self` share their data.
    ///
    /// This method returns an exception if the old and new array have a different number of
    /// elements.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn reshape<'target, 'current, D, S, F>(
        &self,
        scope: S,
        dims: D,
    ) -> JuliaResult<'target, 'data, Array<'target, 'data>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        use std::mem::MaybeUninit;

        use crate::catch::catch_exceptions_with_slots;

        let (output, scope) = scope.split();
        scope
            .scope(|mut frame| {
                let elty_ptr = self.element_type().value_unchecked().unwrap(Private);

                // Safety: The array type is rooted until the array has been constructed, all C API
                // functions are called with valid data. If an exception is thrown it's caught.
                let mut callback =
                    |frame: &mut GcFrame, result: &mut MaybeUninit<*mut jl_array_t>| {
                        let array_type = jl_apply_array_type(elty_ptr, dims.n_dimensions());
                        frame.push_root(NonNull::new_unchecked(array_type));

                        let tuple = if dims.n_dimensions() <= 8 {
                            small_dim_tuple(frame, &dims)
                        } else {
                            large_dim_tuple(frame, &dims)
                        };

                        let array = jl_reshape_array(
                            array_type,
                            self.unwrap(Private),
                            tuple.unwrap(Private),
                        );

                        result.write(array);
                        Ok(())
                    };

                match catch_exceptions_with_slots(&mut frame, &mut callback).unwrap() {
                    Ok(array_ptr) => Ok(Ok(output.set_root(NonNull::new_unchecked(array_ptr)))),
                    Err(e) => Ok(Err(output.set_root(NonNull::new_unchecked(e.ptr())))),
                }
            })
            .unwrap()
    }

    /// Reshape the array, a new array is returned that has dimensions `dims`. The new array and
    /// `self` share their data.
    ///
    /// Safety: If the dimensions are incompatible with the array size, Julia will throw an error.
    /// This error is not caught, which is UB from a `ccall`ed function.
    pub unsafe fn reshape_unchecked<'target, 'current, D, S, F>(
        &self,
        scope: S,
        dims: D,
    ) -> Array<'target, 'data>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        let (output, scope) = scope.split();
        scope
            .scope(|mut frame| {
                let elty_ptr = self.element_type().value_unchecked().unwrap(Private);
                let array_type = jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());
                frame
                    .as_scope()
                    .push_root(NonNull::new_unchecked(array_type));

                let tuple = if dims.n_dimensions() <= 8 {
                    small_dim_tuple(&mut frame, &dims)
                } else {
                    large_dim_tuple(&mut frame, &dims)
                };

                let res = jl_reshape_array(array_type, self.unwrap(Private), tuple.unwrap(Private));
                Ok(output.value(NonNull::new_unchecked(res), Private))
            })
            .unwrap()
    }

    fn ensure_bits_containing<T>(self) -> JlrsResult<()>
    where
        T: ValidLayout,
    {
        if !self.is_inline_array() {
            Err(ArrayLayoutError::NotInline {
                element_type: unsafe { self.element_type().value_unchecked() }
                    .display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        // Safety: Inline array must have a DataType as element type
        if unsafe {
            self.element_type()
                .value_unchecked()
                .cast_unchecked::<DataType>()
                .has_pointer_fields()?
        } {
            Err(ArrayLayoutError::NotBits {
                element_type: unsafe { self.element_type().value_unchecked() }
                    .display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        if !self.contains::<T>() {
            Err(AccessError::InvalidLayout {
                value_type_str: unsafe { self.element_type().value_unchecked() }
                    .display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(())
    }

    fn ensure_inline_containing<T>(self) -> JlrsResult<()>
    where
        T: ValidLayout,
    {
        if !self.is_inline_array() {
            Err(ArrayLayoutError::NotInline {
                element_type: unsafe { self.element_type().value_unchecked() }
                    .display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        if !self.contains::<T>() {
            Err(AccessError::InvalidLayout {
                value_type_str: unsafe { self.element_type().value_unchecked() }
                    .display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(())
    }

    fn ensure_ptr_containing<'fr, 'da, T>(self) -> JlrsResult<()>
    where
        T: WrapperRef<'fr, 'da>,
    {
        if !self.is_value_array() {
            Err(ArrayLayoutError::NotPointer {
                element_type: unsafe { self.element_type().value_unchecked() }
                    .display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        if !self.contains::<T>() {
            Err(AccessError::InvalidLayout {
                value_type_str: unsafe { self.element_type().value_unchecked() }
                    .display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(())
    }

    fn ensure_union(self) -> JlrsResult<()> {
        if !self.is_union_array() {
            let element_type = unsafe { self.element_type().value_unchecked() }
                .display_string_or(CANNOT_DISPLAY_TYPE);
            Err(ArrayLayoutError::NotUnion { element_type })?
        }

        Ok(())
    }
}

impl<'scope> Array<'scope, 'static> {
    /// Insert `inc` elements at the end of the array.
    ///
    /// The array must be 1D and not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn grow_end<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        inc: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        use crate::catch::catch_exceptions;
        use std::mem::MaybeUninit;

        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.
        let mut callback = |result: &mut MaybeUninit<()>| {
            jl_array_grow_end(self.unwrap(Private), inc);
            result.write(());
            Ok(())
        };

        match catch_exceptions(&mut callback).unwrap() {
            Ok(_) => Ok(()),
            Err(e) => Err(frame.value(NonNull::new_unchecked(e.ptr()), Private)),
        }
    }

    /// Insert `inc` elements at the end of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    pub unsafe fn grow_end_unchecked(&mut self, inc: usize) {
        jl_array_grow_end(self.unwrap(Private), inc);
    }

    /// Remove `dec` elements from the end of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn del_end<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        dec: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        use crate::catch::catch_exceptions;
        use std::mem::MaybeUninit;

        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.
        let mut callback = |result: &mut MaybeUninit<()>| {
            jl_array_del_end(self.unwrap(Private), dec);
            result.write(());
            Ok(())
        };

        match catch_exceptions(&mut callback).unwrap() {
            Ok(_) => Ok(()),
            Err(e) => Err(frame.value(NonNull::new_unchecked(e.ptr()), Private)),
        }
    }

    /// Remove `dec` elements from the end of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    pub unsafe fn del_end_unchecked(&mut self, dec: usize) {
        jl_array_del_end(self.unwrap(Private), dec);
    }

    /// Insert `inc` elements at the beginning of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn grow_begin<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        inc: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        use crate::catch::catch_exceptions;
        use std::mem::MaybeUninit;

        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.
        let mut callback = |result: &mut MaybeUninit<()>| {
            jl_array_grow_beg(self.unwrap(Private), inc);
            result.write(());
            Ok(())
        };

        match catch_exceptions(&mut callback).unwrap() {
            Ok(_) => Ok(()),
            Err(e) => Err(frame.value(NonNull::new_unchecked(e.ptr()), Private)),
        }
    }

    /// Insert `inc` elements at the beginning of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    pub unsafe fn grow_begin_unchecked(&mut self, inc: usize) {
        jl_array_grow_beg(self.unwrap(Private), inc);
    }

    /// Remove `dec` elements from the beginning of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn del_begin<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        dec: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        use crate::catch::catch_exceptions;
        use std::mem::MaybeUninit;

        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.
        let mut callback = |result: &mut MaybeUninit<()>| {
            jl_array_del_beg(self.unwrap(Private), dec);
            result.write(());
            Ok(())
        };

        match catch_exceptions(&mut callback).unwrap() {
            Ok(_) => Ok(()),
            Err(e) => Err(frame.value(NonNull::new_unchecked(e.ptr()), Private)),
        }
    }

    /// Remove `dec` elements from the beginning of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    pub unsafe fn del_begin_unchecked(&mut self, dec: usize) {
        jl_array_del_beg(self.unwrap(Private), dec);
    }
}

unsafe impl<'scope, 'data> Typecheck for Array<'scope, 'data> {
    fn typecheck(t: DataType) -> bool {
        // Safety: Array is a UnionAll. so check if the typenames match
        unsafe { t.type_name().wrapper_unchecked() == TypeName::of_array(Global::new()) }
    }
}

impl_debug!(Array<'_, '_>);

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Array<'scope, 'data> {
    type Wraps = jl_array_t;
    const NAME: &'static str = "Array";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// Exactly the same as [`Array`], except it has an explicit element type `T`.
#[repr(transparent)]
pub struct TypedArray<'scope, 'data, T>(
    NonNull<jl_array_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
    PhantomData<T>,
)
where
    T: ValidLayout;

impl<'scope, 'data, T> Clone for TypedArray<'scope, 'data, T>
where
    T: ValidLayout,
{
    fn clone(&self) -> Self {
        unsafe { TypedArray::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }
}
impl<'scope, 'data, T> Copy for TypedArray<'scope, 'data, T> where T: ValidLayout {}

impl<'data, T> TypedArray<'_, 'data, T>
where
    T: ValidLayout + IntoJulia,
{
    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. If you
    /// want to create an array for a type that doesn't implement this trait you must use
    /// [`Array::new_for`].
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn new<'target, 'current, D, S, F>(
        scope: S,
        dims: D,
    ) -> JuliaResult<'target, 'static, TypedArray<'target, 'static, T>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        match Array::new::<T, _, _, _>(scope, dims) {
            // Safety: the type is correct.
            Ok(arr) => Ok(unsafe { arr.as_typed_unchecked() }),
            Err(e) => Err(e),
        }
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method is equivalent to [`Array::new`] except that Julia exceptions are not caught.
    ///
    /// Safety: If the array size is too large, Julia will throw an error. This error is not
    /// caught, which is UB from a `ccall`ed function.
    pub unsafe fn new_unchecked<'target, 'current, D, S, F>(
        scope: S,
        dims: D,
    ) -> TypedArray<'target, 'static, T>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        Array::new_unchecked::<T, _, _, _>(scope, dims).as_typed_unchecked()
    }

    /// Create a new n-dimensional Julia array of dimensions `dims` that borrows data from Rust.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. Because
    /// the data is borrowed from Rust, operations that can change the size of the array (e.g.
    /// `push!`) will fail.
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn from_slice<'target, 'current, D, S, F>(
        scope: S,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<JuliaResult<'target, 'static, TypedArray<'target, 'data, T>>>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        match Array::from_slice(scope, data, dims)? {
            // Safety: the type is correct.
            Ok(arr) => Ok(Ok(unsafe { arr.as_typed_unchecked() })),
            Err(err) => Ok(Err(err)),
        }
    }

    /// Create a new n-dimensional Julia array of dimensions `dims` that borrows data from Rust.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. Because
    /// the data is borrowed from Rust, operations that can change the size of the array (e.g.
    /// `push!`) will fail.
    ///
    /// Safety: If the array size is too large, Julia will throw an error. This error is not
    /// caught, which is UB from a `ccall`ed function.
    pub unsafe fn from_slice_unchecked<'target, 'current, D, S, F>(
        scope: S,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<TypedArray<'target, 'data, T>>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        Ok(Array::from_slice_unchecked(scope, data, dims)?.as_typed_unchecked())
    }

    /// Create a new n-dimensional Julia array of dimensions `dims` that takes ownership of Rust
    /// data.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. Because
    /// the data is allocated by Rust, operations that can change the size of the array (e.g.
    /// `push!`) will fail.
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn from_vec<'target, 'current, D, S, F>(
        scope: S,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<JuliaResult<'target, 'static, TypedArray<'target, 'static, T>>>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        match Array::from_vec(scope, data, dims)? {
            // Safety: the type is correct.
            Ok(arr) => Ok(Ok(unsafe { arr.as_typed_unchecked() })),
            Err(err) => Ok(Err(err)),
        }
    }

    /// Create a new n-dimensional Julia array of dimensions `dims` that takes ownership of Rust
    /// data.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. Because
    /// the data is allocated by Rust, operations that can change the size of the array (e.g.
    /// `push!`) will fail.
    ///
    /// Safety: If the array size is too large, Julia will throw an error. This error is not
    /// caught, which is UB from a `ccall`ed function.
    pub unsafe fn from_vec_unchecked<'target, 'current, D, S, F>(
        scope: S,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<TypedArray<'target, 'static, T>>
    where
        T: IntoJulia,
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        Ok(Array::from_vec_unchecked(scope, data, dims)?.as_typed_unchecked())
    }
}

impl<'data, T> TypedArray<'_, 'data, T>
where
    T: ValidLayout,
{
    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `ty`.
    ///
    /// The elementy type, ty` must be a `Union`, `UnionAll` or `DataType`.
    ///
    /// If the array size is too large or if the type is invalid, Julia will throw an error. This
    /// error is caught and returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn new_for<'target, 'current, D, S, F>(
        scope: S,
        dims: D,
        ty: Value,
    ) -> JlrsResult<JuliaResult<'target, 'static, TypedArray<'target, 'static, T>>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        if !T::valid_layout(ty) {
            let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type_str })?;
        }

        match Array::new_for(scope, dims, ty) {
            // Safety: the type is correct.
            Ok(arr) => Ok(Ok(unsafe { arr.as_typed_unchecked() })),
            Err(err) => Ok(Err(err)),
        }
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method is equivalent to [`Array::new_for`] except that Julia exceptions are not
    /// caught.
    ///
    /// Safety: If the array size is too large or if the type is invalid, Julia will throw an
    /// error. This error is not caught, which is UB from a `ccall`ed function.
    pub unsafe fn new_for_unchecked<'target, 'current, D, S, F>(
        scope: S,
        dims: D,
        ty: Value,
    ) -> JlrsResult<TypedArray<'target, 'static, T>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        if !T::valid_layout(ty) {
            let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type_str })?;
        }

        Ok(Array::new_for_unchecked(scope, dims, ty).as_typed_unchecked())
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> TypedArray<'target, 'data, T> {
        let ptr = self.unwrap_non_null(Private);
        // Safety: the data is valid.
        unsafe {
            output.set_root::<Array>(ptr);
            TypedArray::wrap_non_null(ptr, Private)
        }
    }
}

impl<'data> TypedArray<'_, 'data, u8> {
    /// Convert a string to a Julia array.
    pub fn from_string<'target, A: AsRef<str>, S>(
        scope: S,
        data: A,
    ) -> TypedArray<'target, 'static, u8>
    where
        A: IntoJulia,
        S: PartialScope<'target>,
    {
        let string = data.as_ref();
        let nbytes = string.bytes().len();
        let ptr = string.as_ptr();

        // Safety: a string can be converted to an array of bytes.
        unsafe {
            let arr = jl_pchar_to_array(ptr.cast(), nbytes);
            scope.value(NonNull::new_unchecked(arr), Private)
        }
    }
}

impl<'scope, 'data, T> TypedArray<'scope, 'data, T>
where
    T: ValidLayout,
{
    /// Returns the array's dimensions.
    pub unsafe fn dimensions(self) -> ArrayDimensions<'scope> {
        self.as_array().dimensions()
    }

    /// Returns the type of this array's elements.
    pub fn element_type(self) -> ValueRef<'scope, 'static> {
        self.as_array().element_type()
    }

    /// Returns the size of this array's elements.
    pub fn element_size(self) -> usize {
        self.as_array().element_size()
    }

    /// Returns `true` if the elements of the array are stored inline.
    pub fn is_inline_array(self) -> bool {
        self.as_array().is_inline_array()
    }

    /// Returns true if the elements of the array are stored inline and at least one of the fields
    /// of the inlined type is a pointer.
    pub fn has_inlined_pointers(self) -> bool {
        self.as_array().has_inlined_pointers()
    }

    /// Returns `true` if elements of this array are zero-initialized.
    pub fn zero_init(self) -> bool {
        self.as_array().zero_init()
    }

    /// Returns true if the elements of the array are stored as [`Value`]s.
    pub fn is_value_array(self) -> bool {
        !self.is_inline_array()
    }

    fn ensure_bits(self) -> JlrsResult<()> {
        if !self.is_inline_array() {
            Err(ArrayLayoutError::NotInline {
                element_type: unsafe { self.element_type().value_unchecked() }
                    .display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        // Safety: Inline array must have a DataType as element type
        if unsafe {
            self.element_type()
                .value_unchecked()
                .cast_unchecked::<DataType>()
                .has_pointer_fields()?
        } {
            Err(ArrayLayoutError::NotBits {
                element_type: unsafe { self.element_type().value_unchecked() }
                    .display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(())
    }

    fn ensure_inline(self) -> JlrsResult<()> {
        if !self.is_inline_array() {
            Err(ArrayLayoutError::NotInline {
                element_type: unsafe { self.element_type().value_unchecked() }
                    .display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(())
    }

    /// Immutably the contents of this array. The elements must have an `isbits` type.
    ///
    /// You can borrow data from multiple arrays at the same time.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline, `ArrayLayoutError::NotBits`
    /// if the type is not an `isbits` type, or `AccessError::InvalidLayout` if `T` is not a valid
    /// layout for the array elements.
    pub unsafe fn bits_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<BitsArrayAccessorI<'borrow, 'scope, 'data, T>> {
        self.ensure_bits()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self.as_array_ref());
        Ok(accessor)
    }

    /// Mutably access the contents of this array. The elements must have an `isbits` type.
    ///
    /// This method can be used to gain mutable access to the contents of a single array.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline, `ArrayLayoutError::NotBits`
    /// if the type is not an `isbits` type, or `AccessError::InvalidLayout` if `T` is not a valid
    /// layout for the array elements.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn bits_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<BitsArrayAccessorMut<'borrow, 'scope, 'data, T>> {
        self.ensure_bits()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self.as_array_ref());
        Ok(accessor)
    }

    /// Immutably the contents of this array. The elements must be stored inline.
    ///
    /// You can borrow data from multiple arrays at the same time.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline or
    /// `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements.
    pub unsafe fn inline_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<InlinePtrArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.ensure_inline()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self.as_array_ref());
        Ok(accessor)
    }

    /// Mutably access the contents of this array. The elements must be stored inline.
    ///
    /// This method can be used to gain mutable access to the contents of a single array.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline or
    /// `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn inline_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<InlinePtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.ensure_inline()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self.as_array_ref());
        Ok(accessor)
    }

    /// Convert `self` to `Array`.
    pub fn as_array(self) -> Array<'scope, 'data> {
        unsafe { Array::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }

    /// Convert `self` to `Array`.
    pub fn as_array_ref(&self) -> &Array<'scope, 'data> {
        unsafe { std::mem::transmute(self) }
    }

    /// Reshape the array, a new array is returned that has dimensions `dims`. The new array and
    /// `self` share their data.
    ///
    /// This method returns an exception if the old and new array have a different number of
    /// elements.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn reshape<'target, 'current, D, S, F>(
        &self,
        scope: S,
        dims: D,
    ) -> JuliaResult<'target, 'data, TypedArray<'target, 'data, T>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        match self.as_array().reshape(scope, dims) {
            // Safety: the type is correct.
            Ok(arr) => Ok(arr.as_typed_unchecked()),
            Err(err) => Err(err),
        }
    }

    /// Reshape the array, a new array is returned that has dimensions `dims`. The new array and
    /// `self` share their data.
    ///
    /// Safety: If the dimensions are incompatible with the array size, Julia will throw an error.
    /// This error is not caught, which is UB from a `ccall`ed function.
    pub unsafe fn reshape_unchecked<'target, 'current, 'borrow, D, S, F>(
        self,
        scope: S,
        dims: D,
    ) -> TypedArray<'target, 'data, T>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        self.as_array()
            .reshape_unchecked(scope, dims)
            .as_typed_unchecked()
    }

    /// Immutably access the contents of this array.
    ///
    /// You can borrow data from multiple arrays at the same time.
    pub unsafe fn indeterminate_data<'borrow>(
        &'borrow self,
    ) -> IndeterminateArrayAccessor<'borrow, 'scope, 'data, Immutable<'borrow, u8>> {
        // Safety: layouts are compatible, access is immutable.
        ArrayAccessor::new2(self.as_array_ref())
    }

    /// Mutably access the contents of this array.
    ///
    /// This method can be used to gain mutable access to the contents of a single array.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn indeterminate_data_mut<'borrow>(
        &'borrow mut self,
    ) -> IndeterminateArrayAccessor<'borrow, 'scope, 'data, Mutable<'borrow, u8>> {
        // Safety: layouts are compatible, access is immutable.
        ArrayAccessor::new2(self.as_array_ref())
    }
}

impl<'scope, 'data, T: WrapperRef<'scope, 'data> + ValidLayout> TypedArray<'scope, 'data, T> {
    fn ensure_ptr(self) -> JlrsResult<()> {
        if !self.as_array().is_value_array() {
            Err(ArrayLayoutError::NotPointer {
                element_type: unsafe { self.element_type().value_unchecked() }
                    .display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(())
    }

    /// Immutably the contents of this array. The elements must not be stored inline.
    ///
    /// You can borrow data from multiple arrays at the same time.
    ///
    /// Returns `ArrayLayoutError::NotPointer` if the data is stored inline or `AccessError::InvalidLayout` if `T`
    /// is not a valid layout for the array elements.
    pub unsafe fn wrapper_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, T>> {
        self.ensure_ptr()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self.as_array_ref());
        Ok(accessor)
    }

    /// Mutably access the contents of this array. The elements must not be stored inline.
    ///
    /// This method can be used to gain mutable access to the contents of a single array.
    ///
    /// Returns `ArrayLayoutError::NotPointer` if the data is stored inline or `AccessError::InvalidLayout` if `T`
    /// is not a valid layout for the array elements.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn wrapper_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, T>> {
        self.ensure_ptr()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self.as_array_ref());
        Ok(accessor)
    }

    /// Immutably the contents of this array. The elements must not be stored inline.
    ///
    /// You can borrow data from multiple arrays at the same time.
    ///
    /// Returns `ArrayLayoutError::NotPointer` if the data is stored inline.
    pub unsafe fn value_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        self.ensure_ptr()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self.as_array_ref());
        Ok(accessor)
    }

    /// Mutably access the contents of this array. The elements must not be stored inline.
    ///
    /// This method can be used to gain mutable access to the contents of a single array.
    ///
    /// Returns `ArrayLayoutError::NotPointer` if the data is stored inline.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn value_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        self.ensure_ptr()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self.as_array_ref());
        Ok(accessor)
    }
}

impl<'scope, 'data, T> TypedArray<'scope, 'data, T>
where
    T: 'static + ValidLayout,
{
    /// Copy the data of an inline array to Rust.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline or `AccessError::InvalidLayout`
    /// if the type of the elements is incorrect.
    pub unsafe fn copy_inline_data(&self) -> JlrsResult<CopiedArray<T>> {
        self.ensure_bits()?;

        // Safety: layouts are compatible and is guaranteed to be a bits type due to the
        // 'static constraint on T.
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

impl<'scope, T> TypedArray<'scope, 'static, T>
where
    T: ValidLayout,
{
    /// Insert `inc` elements at the end of the array.
    ///
    /// The array must be 1D and not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn grow_end<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        inc: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.as_array().grow_end(frame, inc)
    }

    /// Insert `inc` elements at the end of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    pub unsafe fn grow_end_unchecked(&mut self, inc: usize) {
        self.as_array().grow_end_unchecked(inc)
    }

    /// Remove `dec` elements from the end of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn del_end<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        dec: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.as_array().del_end(frame, dec)
    }

    /// Remove `dec` elements from the end of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    pub unsafe fn del_end_unchecked(&mut self, dec: usize) {
        self.as_array().del_end_unchecked(dec)
    }

    /// Insert `inc` elements at the beginning of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn grow_begin<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        inc: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.as_array().grow_begin(frame, inc)
    }

    /// Insert `inc` elements at the beginning of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    pub unsafe fn grow_begin_unchecked(&mut self, inc: usize) {
        self.as_array().grow_begin_unchecked(inc)
    }

    /// Remove `dec` elements from the beginning of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn del_begin<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        dec: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.as_array().del_begin(frame, dec)
    }

    /// Remove `dec` elements from the beginning of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    pub unsafe fn del_begin_unchecked(&mut self, dec: usize) {
        self.as_array().del_begin_unchecked(dec)
    }
}

unsafe impl<'scope, 'data, T: ValidLayout> Typecheck for TypedArray<'scope, 'data, T> {
    fn typecheck(t: DataType) -> bool {
        // Safety: borrow is only temporary
        unsafe {
            t.is::<Array>()
                && T::valid_layout(
                    t.parameters()
                        .wrapper_unchecked()
                        .unrestricted_data()
                        .as_slice()[0]
                        .value_unchecked(),
                )
        }
    }
}

impl<T: ValidLayout> Debug for TypedArray<'_, '_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.display_string() {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "<Cannot display value: {}>", e),
        }
    }
}

impl<'scope, 'data, T: ValidLayout> WrapperPriv<'scope, 'data>
    for TypedArray<'scope, 'data, T>
{
    type Wraps = jl_array_t;
    const NAME: &'static str = "Array";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it. T must be correct
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
        let t = isize::julia_type(global).ptr();
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

// Safety: dims.m_dimensions() <= 8
unsafe fn small_dim_tuple<'scope, D>(
    frame: &mut GcFrame<'scope>,
    dims: &D,
) -> Value<'scope, 'static>
where
    D: Dims,
{
    let n = dims.n_dimensions();
    debug_assert!(n <= 8, "Too many dimensions for small_dim_tuple");
    let elem_types = JL_LONG_TYPE.with(|longs| longs.get());
    let tuple_type = jl_apply_tuple_type_v(elem_types.cast(), n);
    let tuple = jl_new_struct_uninit(tuple_type);
    let dims = dims.into_dimensions();
    let tup_nn = NonNull::new_unchecked(tuple);
    frame.push_root(tup_nn);

    let usize_ptr: *mut usize = tuple.cast();
    std::ptr::copy_nonoverlapping(dims.as_slice().as_ptr(), usize_ptr, n);

    Value::wrap_non_null(tup_nn, Private)
}

fn large_dim_tuple<'scope, D>(frame: &mut GcFrame<'scope>, dims: &D) -> Value<'scope, 'static>
where
    D: Dims,
{
    // Safety: all C API functions are called with valid arguments.
    unsafe {
        let n = dims.n_dimensions();
        let global = Global::new();
        let mut elem_types = vec![isize::julia_type(global); n];
        let tuple_type = jl_apply_tuple_type_v(elem_types.as_mut_ptr().cast(), n);
        let tuple = jl_new_struct_uninit(tuple_type);
        let tup_nn = NonNull::new_unchecked(tuple);
        frame.push_root(tup_nn);

        let usize_ptr: *mut usize = tuple.cast();
        let dims = dims.into_dimensions();
        std::ptr::copy_nonoverlapping(dims.as_slice().as_ptr(), usize_ptr, n);

        Value::wrap_non_null(tup_nn, Private)
    }
}

// Safety: must be used as a finalizer when moving array data from Rust to Julia
// to ensure it's freed correctly.
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
    T: Debug + ValidLayout,
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
            Ok(scope.value(ptr, Private))
        } else {
            Err(crate::error::AccessError::UndefRef)?
        }
    }
}

/// A reference to an [`Array`] that has not been explicitly rooted.
pub type ArrayRef<'scope, 'data> = Ref<'scope, 'data, Array<'scope, 'data>>;

unsafe impl ValidLayout for ArrayRef<'_, '_> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<Array>()
        } else if let Ok(ua) = v.cast::<UnionAll>() {
            unsafe { ua.base_type().wrapper_unchecked().is::<Array>() }
        } else {
            false
        }
    }

    const IS_REF: bool = true;
}

impl_ref_root!(Array, ArrayRef, 2);

/// A reference to an [`TypedArray`] that has not been explicitly rooted.
pub type TypedArrayRef<'scope, 'data, T> = Ref<'scope, 'data, TypedArray<'scope, 'data, T>>;

unsafe impl<T: ValidLayout + Debug> ValidLayout for TypedArrayRef<'_, '_, T> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<TypedArray<T>>()
        } else if let Ok(ua) = v.cast::<UnionAll>() {
            unsafe { ua.base_type().wrapper_unchecked().is::<TypedArray<T>>() }
        } else {
            false
        }
    }

    const IS_REF: bool = true;
}

impl<'scope, 'data, T> TypedArrayRef<'scope, 'data, T>
where
    T: ValidLayout + Debug,
{
    pub unsafe fn root<'target, S>(self, scope: S) -> JlrsResult<TypedArray<'target, 'data, T>>
    where
        S: PartialScope<'target>,
    {
        <TypedArray<T> as Root>::root(scope, self)
    }
}
