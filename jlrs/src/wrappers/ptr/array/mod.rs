//! Wrappers for `Array`, create and access n-dimensional Julia arrays from Rust.
//!
//! You will find two wrappers in this module that can be used to work with Julia arrays from
//! Rust. An [`Array`] is the Julia array itself, [`TypedArray`] is also available which can be
//! used if the element type implements [`ValidField`].
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

use self::data::accessor::{
    ArrayAccessor, BitsArrayAccessorI, BitsArrayAccessorMut, Immutable, IndeterminateArrayAccessor,
    IndeterminateArrayAccessorI, InlinePtrArrayAccessorI, InlinePtrArrayAccessorMut, Mutable,
    PtrArrayAccessorI, PtrArrayAccessorMut, UnionArrayAccessorI, UnionArrayAccessorMut,
};
use crate::layout::valid_layout::ValidField;
use crate::memory::target::ExtendedTarget;
use crate::{
    convert::into_julia::IntoJulia,
    error::{AccessError, ArrayLayoutError, InstantiationError, JlrsResult, CANNOT_DISPLAY_TYPE},
    layout::{typecheck::Typecheck, valid_layout::ValidLayout},
    memory::{
        get_tls,
        target::frame::GcFrame,
        target::global::Global,
        target::{private::TargetPriv, Target},
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

use super::{union_all::UnionAll, value::ValueRef, Ref};

cfg_if! {
    if #[cfg(not(all(target_os = "windows", feature = "lts")))] {
        use crate::{catch::{catch_exceptions_with_slots, catch_exceptions}};
        use std::mem::MaybeUninit;
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
/// provided, this type must implement [`ValidField`] to ensure the layouts in Rust and Julia are
/// compatible.
///
/// If the data isn't inlined, e.g. because it's mutable, each element is stored as a [`Value`].
/// This data can be accessed using [`Array::value_data`] and [`Array::value_data_mut`].
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
    pub fn new<'target, 'current, 'borrow, T, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> ArrayResult<'target, 'static, S>
    where
        T: IntoJulia,
        D: Dims,
        S: Target<'target>,
    {
        let (output, frame) = target.split();
        frame
            .scope(|mut frame| {
                let global = frame.global();
                let elty_ptr = T::julia_type(&global).ptr();

                // Safety: The array type is rooted until the array has been constructed, all C API
                // functions are called with valid data.
                unsafe {
                    let mut callback =
                        |frame: &mut GcFrame, result: &mut MaybeUninit<*mut jl_array_t>| {
                            let array_type =
                                jl_apply_array_type(elty_ptr.as_ptr().cast(), dims.n_dimensions());
                            let _: Value = frame
                                .as_mut()
                                .data_from_ptr(NonNull::new_unchecked(array_type), Private);

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

                    let res = match catch_exceptions_with_slots(&mut frame, &mut callback).unwrap()
                    {
                        Ok(array_ptr) => Ok(NonNull::new_unchecked(array_ptr)),
                        Err(e) => Err(e.ptr()),
                    };

                    Ok(output.result_from_ptr(res, Private))
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
    pub unsafe fn new_unchecked<'target, 'current, 'borrow, T, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> ArrayData<'target, 'static, S>
    where
        T: IntoJulia,
        D: Dims,
        S: Target<'target>,
    {
        let (output, frame) = target.split();
        frame
            .scope(|mut frame| {
                let elty_ptr = T::julia_type(&frame).ptr();
                let array_type = jl_apply_array_type(elty_ptr.cast().as_ptr(), dims.n_dimensions());
                let _: Value = frame
                    .as_mut()
                    .data_from_ptr(NonNull::new_unchecked(array_type), Private);

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

                Ok(output.data_from_ptr(NonNull::new_unchecked(array), Private))
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
    pub fn new_for<'target, 'current, 'borrow, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
        ty: Value,
    ) -> ArrayResult<'target, 'static, S>
    where
        D: Dims,
        S: Target<'target>,
    {
        let (output, frame) = target.split();
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
                            let _: Value = frame
                                .as_mut()
                                .data_from_ptr(NonNull::new_unchecked(array_type), Private);

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

                    let res = match catch_exceptions_with_slots(&mut frame, &mut callback).unwrap()
                    {
                        Ok(array_ptr) => Ok(NonNull::new_unchecked(array_ptr)),
                        Err(e) => Err(e.ptr()),
                    };

                    Ok(output.result_from_ptr(res, Private))
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
    pub unsafe fn new_for_unchecked<'target, 'current, 'borrow, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
        ty: Value,
    ) -> ArrayData<'target, 'static, S>
    where
        D: Dims,
        S: Target<'target>,
    {
        let (output, frame) = target.split();
        frame
            .scope(|mut frame| {
                let array_type = jl_apply_array_type(ty.unwrap(Private), dims.n_dimensions());
                let _: Value = frame
                    .as_mut()
                    .data_from_ptr(NonNull::new_unchecked(array_type), Private);

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

                Ok(output.data_from_ptr(NonNull::new_unchecked(array), Private))
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
    pub fn from_slice<'target: 'current, 'current: 'borrow, 'borrow, T, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<ArrayResult<'target, 'data, S>>
    where
        T: IntoJulia,
        D: Dims,
        S: Target<'target>,
    {
        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let (output, frame) = target.split();
        frame.scope(|mut frame| {
            let elty_ptr = T::julia_type(&frame).ptr().cast();

            // Safety: The array type is rooted until the array has been constructed, all C API
            // functions are called with valid data. The data-lifetime ensures the data can't be
            // used from Rust after the borrow ends.
            unsafe {
                let mut callback =
                    |frame: &mut GcFrame, result: &mut MaybeUninit<*mut jl_array_t>| {
                        let array_type =
                            jl_apply_array_type(elty_ptr.as_ptr(), dims.n_dimensions());
                        let _: Value = frame
                            .as_mut()
                            .data_from_ptr(NonNull::new_unchecked(array_type), Private);

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

                let res = match catch_exceptions_with_slots(&mut frame, &mut callback).unwrap() {
                    Ok(array_ptr) => Ok(NonNull::new_unchecked(array_ptr)),
                    Err(e) => Err(e.ptr()),
                };

                Ok(output.result_from_ptr(res, Private))
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
    pub unsafe fn from_slice_unchecked<'target, 'current, 'borrow, T, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<ArrayData<'target, 'data, S>>
    where
        T: IntoJulia,
        D: Dims,
        S: Target<'target>,
    {
        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let (output, frame) = target.split();
        frame.scope(|mut frame| {
            let array_type = jl_apply_array_type(
                T::julia_type(&frame).ptr().cast().as_ptr(),
                dims.n_dimensions(),
            );
            let _: Value = frame
                .as_mut()
                .data_from_ptr(NonNull::new_unchecked(array_type), Private);

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

            Ok(output.data_from_ptr(NonNull::new_unchecked(array), Private))
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
    pub fn from_vec<'target, 'current, 'borrow, T, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<ArrayResult<'target, 'static, S>>
    where
        T: IntoJulia,
        D: Dims,
        S: Target<'target>,
    {
        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let (output, scope) = target.split();
        scope.scope(|mut frame| {
            let elty_ptr = T::julia_type(&frame).ptr().cast();
            let data = Box::leak(data.into_boxed_slice());

            // Safety: The array type is rooted until the array has been constructed, all C API
            // functions are called with valid data. The data-lifetime ensures the data can't be
            // used from Rust after the borrow ends.
            unsafe {
                let mut callback =
                    |frame: &mut GcFrame, result: &mut MaybeUninit<*mut jl_array_t>| {
                        let array_type =
                            jl_apply_array_type(elty_ptr.as_ptr(), dims.n_dimensions());
                        let _: Value = frame
                            .as_mut()
                            .data_from_ptr(NonNull::new_unchecked(array_type), Private);

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

                let res = match catch_exceptions_with_slots(&mut frame, &mut callback).unwrap() {
                    Ok(array_ptr) => Ok(NonNull::new_unchecked(array_ptr)),
                    Err(e) => Err(e.ptr()),
                };

                Ok(output.result_from_ptr(res, Private))
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
    pub unsafe fn from_vec_unchecked<'target, 'current, 'borrow, T, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<ArrayData<'target, 'static, S>>
    where
        T: IntoJulia,
        D: Dims,
        S: Target<'target>,
    {
        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let (output, scope) = target.split();
        scope.scope(|mut frame| {
            let array_type = jl_apply_array_type(
                T::julia_type(&frame).ptr().cast().as_ptr(),
                dims.n_dimensions(),
            );
            let _: Value = frame
                .as_mut()
                .data_from_ptr(NonNull::new_unchecked(array_type), Private);

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
            Ok(output.data_from_ptr(NonNull::new_unchecked(array), Private))
        })
    }

    /// Convert a string to a Julia array.
    pub fn from_string<'target, A, T>(target: T, data: A) -> ArrayData<'target, 'static, T>
    where
        A: AsRef<str>,
        T: Target<'target>,
    {
        let string = data.as_ref();
        let nbytes = string.bytes().len();
        let ptr = string.as_ptr();
        // Safety: a string can be converted to an array of bytes.
        unsafe {
            let arr = jl_pchar_to_array(ptr.cast(), nbytes);
            target.data_from_ptr(NonNull::new_unchecked(arr), Private)
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
    ///
    /// TODO safety
    /// TODO make safe? Mutation is unsafe already
    pub unsafe fn dimensions(self) -> ArrayDimensions<'scope> {
        ArrayDimensions::new(self)
    }

    /// Returns the type of this array's elements.
    pub fn element_type(self) -> Value<'scope, 'static> {
        // Safety: C API function is called valid arguments.
        unsafe {
            Value::wrap_non_null(
                NonNull::new_unchecked(jl_array_eltype(self.unwrap(Private).cast()).cast()),
                Private,
            )
        }
    }

    /// Returns the size of this array's elements.
    pub fn element_size(self) -> usize {
        // Safety: the pointer points to valid data.
        unsafe { self.unwrap_non_null(Private).as_ref().elsize as usize }
    }

    /// Returns `true` if the layout of the elements is compatible with `T`.
    pub fn contains<T: ValidField>(self) -> bool {
        // Safety: C API function is called valid arguments.
        T::valid_field(self.element_type())
    }

    /// Returns `true` if the layout of the elements is compatible with `T` and these elements are
    /// stored inline.
    pub fn contains_inline<T: ValidField>(self) -> bool {
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
        self.is_inline_array() && self.element_type().is::<Union>()
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
        T: ValidField,
    {
        if self.contains::<T>() {
            let ptr = self.unwrap_non_null(Private);
            // Safety: the type is correct
            unsafe { Ok(TypedArray::wrap_non_null(ptr, Private)) }
        } else {
            let value_type = self.element_type().display_string_or(CANNOT_DISPLAY_TYPE);
            Err(AccessError::InvalidLayout { value_type })?
        }
    }

    /// Convert this untyped array to a [`TypedArray`] without checking if this conversion is
    /// valid.
    ///
    /// Safety: `T` must be a valid representation of the data stored in the array.
    pub unsafe fn as_typed_unchecked<T>(self) -> TypedArray<'scope, 'data, T>
    where
        T: ValidField,
    {
        TypedArray::wrap_non_null(self.unwrap_non_null(Private), Private)
    }

    /// Copy the data of an inline array to Rust.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline or `AccessError::InvalidLayout`
    /// if the type of the elements is incorrect.
    pub unsafe fn copy_inline_data<T>(&self) -> JlrsResult<CopiedArray<T>>
    where
        T: 'static + ValidField,
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

    // TODO docs
    // TODO safety for all
    pub unsafe fn bits_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<BitsArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ValidField,
    {
        self.ensure_bits_containing::<T>()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new(self);
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
        T: ValidField,
    {
        self.ensure_bits_containing::<T>()?;

        let accessor = ArrayAccessor::new(self);
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
        T: ValidField,
    {
        self.ensure_inline_containing::<T>()?;

        let accessor = ArrayAccessor::new(self);
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
        T: ValidField,
    {
        self.ensure_inline_containing::<T>()?;

        let accessor = ArrayAccessor::new(self);
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
        Option<T>: ValidField,
    {
        self.ensure_ptr_containing::<T>()?;

        let accessor = ArrayAccessor::new(self);
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
        Option<T>: ValidField,
    {
        self.ensure_ptr_containing::<T>()?;

        let accessor = ArrayAccessor::new(self);
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

        let accessor = ArrayAccessor::new(self);
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

        let accessor = ArrayAccessor::new(self);
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

        let accessor = ArrayAccessor::new(self);
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

        let accessor = ArrayAccessor::new(self);
        Ok(accessor)
    }

    /// Immutably access the contents of this array.
    ///
    /// You can borrow data from multiple arrays at the same time.
    pub unsafe fn indeterminate_data<'borrow>(
        &'borrow self,
    ) -> IndeterminateArrayAccessorI<'borrow, 'scope, 'data> {
        ArrayAccessor::new(self)
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
        ArrayAccessor::new(self)
    }

    /// Reshape the array, a new array is returned that has dimensions `dims`. The new array and
    /// `self` share their data.
    ///
    /// This method returns an exception if the old and new array have a different number of
    /// elements.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn reshape<'target, 'current, 'borrow, D, S>(
        &self,
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> ArrayResult<'target, 'data, S>
    where
        D: Dims,
        S: Target<'target>,
    {
        let (output, scope) = target.split();
        scope
            .scope(|mut frame| {
                let elty_ptr = self.element_type().unwrap(Private);

                // Safety: The array type is rooted until the array has been constructed, all C API
                // functions are called with valid data. If an exception is thrown it's caught.
                let mut callback =
                    |frame: &mut GcFrame, result: &mut MaybeUninit<*mut jl_array_t>| {
                        let array_type = jl_apply_array_type(elty_ptr, dims.n_dimensions());
                        let _: Value = frame
                            .as_mut()
                            .data_from_ptr(NonNull::new_unchecked(array_type), Private);

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

                let res = match catch_exceptions_with_slots(&mut frame, &mut callback).unwrap() {
                    Ok(array_ptr) => Ok(NonNull::new_unchecked(array_ptr)),
                    Err(e) => Err(e.ptr()),
                };

                Ok(output.result_from_ptr(res, Private))
            })
            .unwrap()
    }

    /// Reshape the array, a new array is returned that has dimensions `dims`. The new array and
    /// `self` share their data.
    ///
    /// Safety: If the dimensions are incompatible with the array size, Julia will throw an error.
    /// This error is not caught, which is UB from a `ccall`ed function.
    pub unsafe fn reshape_unchecked<'target, 'current, 'borrow, D, S>(
        &self,
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> ArrayData<'target, 'data, S>
    where
        D: Dims,
        S: Target<'target>,
    {
        let (output, scope) = target.split();
        scope
            .scope(|mut frame| {
                let elty_ptr = self.element_type().unwrap(Private);
                let array_type = jl_apply_array_type(elty_ptr.cast(), dims.n_dimensions());
                let _: Value = frame
                    .as_mut()
                    .data_from_ptr(NonNull::new_unchecked(array_type), Private);

                let tuple = if dims.n_dimensions() <= 8 {
                    small_dim_tuple(&mut frame, &dims)
                } else {
                    large_dim_tuple(&mut frame, &dims)
                };

                let res = jl_reshape_array(array_type, self.unwrap(Private), tuple.unwrap(Private));
                Ok(output.data_from_ptr(NonNull::new_unchecked(res), Private))
            })
            .unwrap()
    }

    fn ensure_bits_containing<T>(self) -> JlrsResult<()>
    where
        T: ValidField,
    {
        if !self.is_inline_array() {
            Err(ArrayLayoutError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        // Safety: Inline array must have a DataType as element type
        if unsafe {
            self.element_type()
                .cast_unchecked::<DataType>()
                .has_pointer_fields()?
        } {
            Err(ArrayLayoutError::NotBits {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        if !self.contains::<T>() {
            Err(AccessError::InvalidLayout {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(())
    }

    fn ensure_inline_containing<T>(self) -> JlrsResult<()>
    where
        T: ValidField,
    {
        if !self.is_inline_array() {
            Err(ArrayLayoutError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        if !self.contains::<T>() {
            Err(AccessError::InvalidLayout {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(())
    }

    fn ensure_ptr_containing<'fr, 'da, T>(self) -> JlrsResult<()>
    where
        T: WrapperRef<'fr, 'da>,
        Option<T>: ValidField,
    {
        if !self.is_value_array() {
            Err(ArrayLayoutError::NotPointer {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        if !self.contains::<Option<T>>() {
            Err(AccessError::InvalidLayout {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(())
    }

    fn ensure_union(self) -> JlrsResult<()> {
        if !self.is_union_array() {
            let element_type = self.element_type().display_string_or(CANNOT_DISPLAY_TYPE);
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
    pub unsafe fn grow_end<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.

        let mut callback = |result: &mut MaybeUninit<()>| {
            jl_array_grow_end(self.unwrap(Private), inc);
            result.write(());
            Ok(())
        };

        let res = match catch_exceptions(&mut callback).unwrap() {
            Ok(_) => Ok(()),
            Err(e) => Err(e.ptr()),
        };

        target.exception_from_ptr(res, Private)
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
    pub unsafe fn del_end<'target, S>(&mut self, target: S, dec: usize) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.
        let mut callback = |result: &mut MaybeUninit<()>| {
            jl_array_del_end(self.unwrap(Private), dec);
            result.write(());
            Ok(())
        };

        let res = match catch_exceptions(&mut callback).unwrap() {
            Ok(_) => Ok(()),
            Err(e) => Err(e.ptr()),
        };

        target.exception_from_ptr(res, Private)
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
    pub unsafe fn grow_begin<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.
        let mut callback = |result: &mut MaybeUninit<()>| {
            jl_array_grow_beg(self.unwrap(Private), inc);
            result.write(());
            Ok(())
        };

        let res = match catch_exceptions(&mut callback).unwrap() {
            Ok(_) => Ok(()),
            Err(e) => Err(e.ptr()),
        };

        target.exception_from_ptr(res, Private)
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
    pub unsafe fn del_begin<'target, S>(
        &mut self,
        target: S,
        dec: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.
        let mut callback = |result: &mut MaybeUninit<()>| {
            jl_array_del_beg(self.unwrap(Private), dec);
            result.write(());
            Ok(())
        };

        let res = match catch_exceptions(&mut callback).unwrap() {
            Ok(_) => Ok(()),
            Err(e) => Err(e.ptr()),
        };

        target.exception_from_ptr(res, Private)
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
        unsafe { t.type_name() == TypeName::of_array(&Global::new()) }
    }
}

impl_debug!(Array<'_, '_>);

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Array<'scope, 'data> {
    type Wraps = jl_array_t;
    type TypeConstructorPriv<'target, 'da> = Array<'target, 'da>;
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
    T: ValidField;

impl<'scope, 'data, T> Clone for TypedArray<'scope, 'data, T>
where
    T: ValidField,
{
    fn clone(&self) -> Self {
        unsafe { TypedArray::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }
}
impl<'scope, 'data, T> Copy for TypedArray<'scope, 'data, T> where T: ValidField {}

impl<'data, T> TypedArray<'_, 'data, T>
where
    T: ValidField + IntoJulia,
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
    pub fn new<'target, 'current, 'borrow, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> TypedArrayResult<'target, 'static, S, T>
    where
        D: Dims,
        S: Target<'target>,
    {
        unsafe {
            let (output, frame) = target.split();
            frame
                .scope(|mut frame| {
                    let global = frame.global();
                    let target = frame.extended_target(global);
                    let x = Array::new::<T, _, _>(target, dims);

                    let res = match x {
                        Ok(arr) => Ok(arr
                            .wrapper()
                            .as_typed_unchecked::<T>()
                            .unwrap_non_null(Private)),
                        Err(e) => Err(e.wrapper().unwrap_non_null(Private)),
                    };

                    Ok(output.result_from_ptr(res, Private))
                })
                .unwrap()
        }
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method is equivalent to [`Array::new`] except that Julia exceptions are not caught.
    ///
    /// Safety: If the array size is too large, Julia will throw an error. This error is not
    /// caught, which is UB from a `ccall`ed function.
    pub unsafe fn new_unchecked<'target, 'current, 'borrow, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> TypedArrayData<'target, 'data, S, T>
    where
        D: Dims,
        S: Target<'target>,
    {
        let (output, frame) = target.split();
        frame
            .scope(|mut frame| {
                let inner_output = frame.global();
                let target = frame.extended_target(inner_output);

                let res = Array::new_unchecked::<T, _, _>(target, dims)
                    .wrapper()
                    .as_typed_unchecked::<T>();

                Ok(output.data_from_ptr(res.unwrap_non_null(Private), Private))
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
    pub fn from_slice<'target: 'current, 'current: 'borrow, 'borrow, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<TypedArrayResult<'target, 'data, S, T>>
    where
        T: IntoJulia,
        D: Dims,
        S: Target<'target>,
    {
        unsafe {
            let (output, frame) = target.split();
            frame.scope(|mut frame| {
                let global = frame.global();
                let target = frame.extended_target(global);

                let res = match Array::from_slice::<T, _, _>(target, data, dims)? {
                    Ok(arr) => Ok(arr
                        .wrapper()
                        .as_typed_unchecked::<T>()
                        .unwrap_non_null(Private)),
                    Err(e) => Err(e.wrapper().unwrap_non_null(Private)),
                };

                Ok(output.result_from_ptr(res, Private))
            })
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
    pub unsafe fn from_slice_unchecked<'target, 'current, 'borrow, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<TypedArrayData<'target, 'data, S, T>>
    where
        T: IntoJulia,
        D: Dims,
        S: Target<'target>,
    {
        let (output, frame) = target.split();
        frame.scope(|mut frame| {
            let inner_output = frame.global();
            let target = frame.extended_target(inner_output);

            let res = Array::from_slice_unchecked::<T, _, _>(target, data, dims)?
                .wrapper()
                .as_typed_unchecked::<T>();

            Ok(output.data_from_ptr(res.unwrap_non_null(Private), Private))
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
    pub fn from_vec<'target, 'current, 'borrow, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<TypedArrayResult<'target, 'static, S, T>>
    where
        T: IntoJulia,
        D: Dims,
        S: Target<'target>,
    {
        unsafe {
            let (output, frame) = target.split();
            frame.scope(|mut frame| {
                let global = frame.global();
                let target = frame.extended_target(global);

                let res = match Array::from_vec::<T, _, _>(target, data, dims)? {
                    Ok(arr) => Ok(arr
                        .wrapper()
                        .as_typed_unchecked::<T>()
                        .unwrap_non_null(Private)),
                    Err(e) => Err(e.wrapper().unwrap_non_null(Private)),
                };

                Ok(output.result_from_ptr(res, Private))
            })
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
    pub unsafe fn from_vec_unchecked<'target, 'current, 'borrow, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<TypedArrayData<'target, 'static, S, T>>
    where
        T: IntoJulia,
        D: Dims,
        S: Target<'target>,
    {
        let (output, frame) = target.split();
        frame.scope(|mut frame| {
            let inner_output = frame.global();
            let target = frame.extended_target(inner_output);

            let res = Array::from_vec_unchecked::<T, _, _>(target, data, dims)?
                .wrapper()
                .as_typed_unchecked::<T>();

            Ok(output.data_from_ptr(res.unwrap_non_null(Private), Private))
        })
    }
}

impl<'data, T> TypedArray<'_, 'data, T>
where
    T: ValidField,
{
    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `ty`.
    ///
    /// The elementy type, ty` must be a `Union`, `UnionAll` or `DataType`.
    ///
    /// If the array size is too large or if the type is invalid, Julia will throw an error. This
    /// error is caught and returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn new_for<'target, 'current, 'borrow, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
        ty: Value,
    ) -> JlrsResult<TypedArrayResult<'target, 'static, S, T>>
    where
        D: Dims,
        S: Target<'target>,
    {
        if !T::valid_field(ty) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        unsafe {
            let (output, frame) = target.split();
            frame.scope(|mut frame| {
                let global = frame.global();
                let target = frame.extended_target(global);

                let res = match Array::new_for(target, dims, ty) {
                    Ok(arr) => Ok(arr
                        .wrapper()
                        .as_typed_unchecked::<T>()
                        .unwrap_non_null(Private)),
                    Err(e) => Err(e.wrapper().unwrap_non_null(Private)),
                };

                Ok(output.result_from_ptr(res, Private))
            })
        }
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method is equivalent to [`Array::new_for`] except that Julia exceptions are not
    /// caught.
    ///
    /// Safety: If the array size is too large or if the type is invalid, Julia will throw an
    /// error. This error is not caught, which is UB from a `ccall`ed function.
    pub unsafe fn new_for_unchecked<'target, 'current, 'borrow, D, S>(
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
        ty: Value,
    ) -> JlrsResult<TypedArrayData<'target, 'static, S, T>>
    where
        D: Dims,
        S: Target<'target>,
    {
        if !T::valid_field(ty) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        let (output, frame) = target.split();
        frame.scope(|mut frame| {
            let inner_output = frame.global();
            let target = frame.extended_target(inner_output);

            let res = Array::new_for_unchecked(target, dims, ty)
                .wrapper()
                .as_typed_unchecked::<T>();

            Ok(output.data_from_ptr(res.unwrap_non_null(Private), Private))
        })
    }
}

impl<'data> TypedArray<'_, 'data, u8> {
    /// Convert a string to a Julia array.
    pub fn from_string<'target, A, T>(target: T, data: A) -> TypedArrayData<'target, 'static, T, u8>
    where
        A: AsRef<str>,
        T: Target<'target>,
    {
        let string = data.as_ref();
        let nbytes = string.bytes().len();
        let ptr = string.as_ptr();

        // Safety: a string can be converted to an array of bytes.
        unsafe {
            let arr = jl_pchar_to_array(ptr.cast(), nbytes);
            target.data_from_ptr(NonNull::new_unchecked(arr), Private)
        }
    }
}

impl<'scope, 'data, T> TypedArray<'scope, 'data, T>
where
    T: ValidField,
{
    /// Returns the array's dimensions.
    pub unsafe fn dimensions(self) -> ArrayDimensions<'scope> {
        self.as_array().dimensions()
    }

    /// Returns the type of this array's elements.
    pub fn element_type(self) -> Value<'scope, 'static> {
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
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        // Safety: Inline array must have a DataType as element type
        if unsafe {
            self.element_type()
                .cast_unchecked::<DataType>()
                .has_pointer_fields()?
        } {
            Err(ArrayLayoutError::NotBits {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(())
    }

    fn ensure_inline(self) -> JlrsResult<()> {
        if !self.is_inline_array() {
            Err(ArrayLayoutError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
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
        let accessor = ArrayAccessor::new(self.as_array_ref());
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
        let accessor = ArrayAccessor::new(self.as_array_ref());
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
        T: ValidField,
    {
        self.ensure_inline()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new(self.as_array_ref());
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
        T: ValidField,
    {
        self.ensure_inline()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new(self.as_array_ref());
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
    pub unsafe fn reshape<'target, 'current, 'borrow, D, S>(
        &self,
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> TypedArrayResult<'target, 'data, S, T>
    where
        D: Dims,
        S: Target<'target>,
    {
        let (output, frame) = target.split();
        frame
            .scope(|mut frame| {
                let global = frame.global();
                let target = frame.extended_target(global);

                let res = match self.as_array().reshape(target, dims) {
                    Ok(arr) => Ok(arr
                        .wrapper()
                        .as_typed_unchecked::<T>()
                        .unwrap_non_null(Private)),
                    Err(e) => Err(e.wrapper().unwrap_non_null(Private)),
                };

                Ok(output.result_from_ptr(res, Private))
            })
            .unwrap()
    }

    /// Reshape the array, a new array is returned that has dimensions `dims`. The new array and
    /// `self` share their data.
    ///
    /// Safety: If the dimensions are incompatible with the array size, Julia will throw an error.
    /// This error is not caught, which is UB from a `ccall`ed function.
    pub unsafe fn reshape_unchecked<'target, 'current, 'borrow, D, S>(
        self,
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> TypedArrayData<'target, 'data, S, T>
    where
        D: Dims,
        S: Target<'target>,
    {
        let (output, frame) = target.split();
        frame
            .scope(|mut frame| {
                let inner_output = frame.global();
                let target = frame.extended_target(inner_output);

                let res = self
                    .as_array()
                    .reshape_unchecked(target, dims)
                    .wrapper()
                    .as_typed_unchecked::<T>()
                    .unwrap_non_null(Private);
                Ok(output.data_from_ptr(res, Private))
            })
            .unwrap()
    }

    /// Immutably access the contents of this array.
    ///
    /// You can borrow data from multiple arrays at the same time.
    pub unsafe fn indeterminate_data<'borrow>(
        &'borrow self,
    ) -> IndeterminateArrayAccessor<'borrow, 'scope, 'data, Immutable<'borrow, u8>> {
        // Safety: layouts are compatible, access is immutable.
        ArrayAccessor::new(self.as_array_ref())
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
        ArrayAccessor::new(self.as_array_ref())
    }
}

impl<'scope, 'data, T> TypedArray<'scope, 'data, Option<T>>
where
    T: WrapperRef<'scope, 'data>,
    Option<T>: ValidField,
{
    fn ensure_ptr(self) -> JlrsResult<()> {
        if !self.as_array().is_value_array() {
            Err(ArrayLayoutError::NotPointer {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
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
        let accessor = ArrayAccessor::new(self.as_array_ref());
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
        let accessor = ArrayAccessor::new(self.as_array_ref());
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
        let accessor = ArrayAccessor::new(self.as_array_ref());
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
        let accessor = ArrayAccessor::new(self.as_array_ref());
        Ok(accessor)
    }
}

impl<'scope, 'data, T> TypedArray<'scope, 'data, T>
where
    T: 'static + ValidField,
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
    T: ValidField,
{
    /// Insert `inc` elements at the end of the array.
    ///
    /// The array must be 1D and not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn grow_end<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.as_array().grow_end(target, inc)
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
    pub unsafe fn del_end<'target, S>(&mut self, target: S, dec: usize) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.as_array().del_end(target, dec)
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
    pub unsafe fn grow_begin<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.as_array().grow_begin(target, inc)
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
    pub unsafe fn del_begin<'target, S>(
        &mut self,
        target: S,
        dec: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.as_array().del_begin(target, dec)
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

unsafe impl<'scope, 'data, T: ValidField> Typecheck for TypedArray<'scope, 'data, T> {
    fn typecheck(t: DataType) -> bool {
        // Safety: borrow is only temporary
        unsafe {
            t.is::<Array>() && T::valid_field(t.parameters().data().as_slice()[0].unwrap().value())
        }
    }
}

impl<T: ValidField> Debug for TypedArray<'_, '_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.display_string() {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "<Cannot display value: {}>", e),
        }
    }
}

impl<'scope, 'data, T: ValidField> WrapperPriv<'scope, 'data> for TypedArray<'scope, 'data, T> {
    type Wraps = jl_array_t;
    type TypeConstructorPriv<'target, 'da> = TypedArray<'target, 'da, T>;
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
        let t = isize::julia_type(global).ptr().as_ptr();
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
    let _: Value = frame.data_from_ptr(tup_nn, Private);

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
        let mut elem_types = vec![isize::julia_type(&frame); n];
        let tuple_type = jl_apply_tuple_type_v(elem_types.as_mut_ptr().cast(), n);
        let tuple = jl_new_struct_uninit(tuple_type);
        let tup_nn = NonNull::new_unchecked(tuple);
        let _: Value = frame.data_from_ptr(tup_nn, Private);

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

/// A reference to an [`Array`] that has not been explicitly rooted.
pub type ArrayRef<'scope, 'data> = Ref<'scope, 'data, Array<'scope, 'data>>;

unsafe impl ValidLayout for ArrayRef<'_, '_> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<Array>()
        } else if let Ok(ua) = v.cast::<UnionAll>() {
            ua.base_type().is::<Array>()
        } else {
            false
        }
    }

    const IS_REF: bool = true;
}

unsafe impl ValidField for Option<ArrayRef<'_, '_>> {
    fn valid_field(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<Array>()
        } else if let Ok(ua) = v.cast::<UnionAll>() {
            ua.base_type().is::<Array>()
        } else {
            false
        }
    }
}

impl_ref_root!(Array, ArrayRef, 2);

/// A reference to an [`TypedArray`] that has not been explicitly rooted.
pub type TypedArrayRef<'scope, 'data, T> = Ref<'scope, 'data, TypedArray<'scope, 'data, T>>;

unsafe impl<T: ValidField> ValidLayout for TypedArrayRef<'_, '_, T> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<TypedArray<T>>()
        } else if let Ok(ua) = v.cast::<UnionAll>() {
            ua.base_type().is::<TypedArray<T>>()
        } else {
            false
        }
    }

    const IS_REF: bool = true;
}

unsafe impl<T: ValidField> ValidField for Option<TypedArrayRef<'_, '_, T>> {
    fn valid_field(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<TypedArray<T>>()
        } else if let Ok(ua) = v.cast::<UnionAll>() {
            ua.base_type().is::<TypedArray<T>>()
        } else {
            false
        }
    }
}

impl<'scope, 'data, U> TypedArrayRef<'scope, 'data, U>
where
    U: ValidField,
{
    pub unsafe fn root<'target, T>(self, target: T) -> TypedArrayData<'target, 'data, T, U>
    where
        T: Target<'target>,
    {
        target.data_from_ptr(self.ptr(), Private)
    }
}

use crate::memory::target::target_type::TargetType;

/// `Array` or `ArrayRef`, depending on the target type `T`.
pub type ArrayData<'target, 'data, T> =
    <T as TargetType<'target>>::Data<'data, Array<'target, 'data>>;

/// `JuliaResult<Array>` or `JuliaResultRef<ArrayRef>`, depending on the target type `T`.
pub type ArrayResult<'target, 'data, T> =
    <T as TargetType<'target>>::Result<'data, Array<'target, 'data>>;

/// `TypedArray<U>` or `TypedArrayRef<U>`, depending on the target type `T`.
pub type TypedArrayData<'target, 'data, T, U> =
    <T as TargetType<'target>>::Data<'data, TypedArray<'target, 'data, U>>;

/// `JuliaResult<TypedArray<U>>` or `JuliaResultRef<TypedArrayRef<U>>`, depending on the target
/// type `T`.
pub type TypedArrayResult<'target, 'data, T, U> =
    <T as TargetType<'target>>::Result<'data, TypedArray<'target, 'data, U>>;
