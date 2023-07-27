//! Managed types for `Array`, create and access n-dimensional Julia arrays from Rust.
//!
//! You will find two managed types in this module that can be used to work with Julia arrays from
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

use std::{
    ffi::c_void,
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    mem,
    mem::MaybeUninit,
    ptr::{null_mut, NonNull},
    slice,
};

use jl_sys::{
    jl_apply_array_type, jl_array_data, jl_array_del_beg, jl_array_del_end, jl_array_dims_ptr,
    jl_array_eltype, jl_array_grow_beg, jl_array_grow_end, jl_array_ndims, jl_array_t,
    jl_gc_add_ptr_finalizer, jl_new_struct_uninit, jl_pchar_to_array, jl_reshape_array,
};

use self::{
    data::accessor::{
        ArrayAccessor, BitsArrayAccessorI, BitsArrayAccessorMut, Immutable,
        IndeterminateArrayAccessor, IndeterminateArrayAccessorI, InlinePtrArrayAccessorI,
        InlinePtrArrayAccessorMut, Mutable, PtrArrayAccessorI, PtrArrayAccessorMut,
        UnionArrayAccessorI, UnionArrayAccessorMut,
    },
    dimensions::DimsExt,
    tracked::{TrackedArray, TrackedArrayMut},
};
use super::{
    union_all::UnionAll,
    value::{typed::TypedValue, ValueRef},
    Ref,
};
use crate::{
    catch::catch_exceptions,
    convert::{
        ccall_types::{CCallArg, CCallReturn},
        into_julia::IntoJulia,
        unbox::Unbox,
    },
    data::{
        layout::valid_layout::{ValidField, ValidLayout},
        managed::{
            array::{
                data::copied::CopiedArray,
                dimensions::{ArrayDimensions, Dims},
            },
            datatype::DataType,
            private::ManagedPriv,
            type_name::TypeName,
            union::Union,
            value::Value,
            Managed, ManagedRef,
        },
        types::{
            construct_type::{
                ArrayTypeConstructor, ConstantIsize, ConstructType, Name, TypeVarConstructor,
            },
            typecheck::Typecheck,
        },
    },
    error::{
        AccessError, ArrayLayoutError, InstantiationError, JlrsResult, TypeError,
        CANNOT_DISPLAY_TYPE,
    },
    memory::{
        context::ledger::Ledger,
        get_tls,
        target::{unrooted::Unrooted, Target, TargetException, TargetResult},
    },
    prelude::ValueData,
    private::Private,
};

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
    /// This method can only be used in combination with types that implement `ConstructType`.
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    pub fn new<'target, 'current, 'borrow, T, D, Tgt>(
        target: Tgt,
        dims: D,
    ) -> ArrayResult<'target, 'static, Tgt>
    where
        T: ConstructType,
        D: DimsExt,
        Tgt: Target<'target>,
    {
        unsafe {
            let callback = || {
                let array_type = D::ArrayContructor::<T>::construct_type(&target).as_value();
                let array = dims.alloc_array(&target, array_type);
                array
            };

            let exc = |err: Value| err.unwrap_non_null(Private);

            let v = match catch_exceptions(callback, exc) {
                Ok(arr) => Ok(arr.ptr()),
                Err(e) => Err(e),
            };

            target.result_from_ptr(v, Private)
        }
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method is equivalent to [`Array::new`] except that Julia exceptions are not caught.
    ///
    /// Safety: If the array size is too large, Julia will throw an error. This error is not
    /// caught, which is UB from a `ccall`ed function.
    pub unsafe fn new_unchecked<'target, 'current, 'borrow, T, D, Tgt>(
        target: Tgt,
        dims: D,
    ) -> ArrayData<'target, 'static, Tgt>
    where
        T: ConstructType,
        D: DimsExt,
        Tgt: Target<'target>,
    {
        let array_type = D::ArrayContructor::<T>::construct_type(&target).as_value();
        dims.alloc_array(target, array_type)
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `ty`.
    ///
    /// The elementy type, ty` must be a` Union`, `UnionAll` or `DataType`.
    ///
    /// If the array size is too large or if the type is invalid, Julia will throw an error. This
    /// error is caught and returned.
    pub fn new_for<'target, 'current, 'borrow, D, Tgt>(
        target: Tgt,
        dims: D,
        ty: Value,
    ) -> ArrayResult<'target, 'static, Tgt>
    where
        D: DimsExt,
        Tgt: Target<'target>,
    {
        let elty_ptr = ty.unwrap(Private);
        // Safety: The array type is rooted until the array has been constructed, all C API
        // functions are called with valid data.
        unsafe {
            let callback = || {
                let array_type = Value::wrap_non_null(
                    NonNull::new_unchecked(jl_apply_array_type(elty_ptr, dims.rank())),
                    Private,
                );
                dims.alloc_array(&target, array_type).ptr()
            };

            let exc = |err: Value| err.unwrap_non_null(Private);
            let res = match catch_exceptions(callback, exc) {
                Ok(array_ptr) => Ok(array_ptr),
                Err(e) => Err(e),
            };

            target.result_from_ptr(res, Private)
        }
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method is equivalent to [`Array::new_for`] except that Julia exceptions are not
    /// caught.
    ///
    /// Safety: If the array size is too large or if the type is invalid, Julia will throw an
    /// error. This error is not caught, which is UB from a `ccall`ed function.
    pub unsafe fn new_for_unchecked<'target, 'current, 'borrow, D, Tgt>(
        target: Tgt,
        dims: D,
        ty: Value,
    ) -> ArrayData<'target, 'static, Tgt>
    where
        D: DimsExt,
        Tgt: Target<'target>,
    {
        let elty_ptr = ty.unwrap(Private);
        let array_type = Value::wrap_non_null(
            NonNull::new_unchecked(jl_apply_array_type(elty_ptr, dims.rank())),
            Private,
        );
        let array = dims.alloc_array(&target, array_type);

        array.root(target)
    }

    /// Create a new n-dimensional Julia array of dimensions `dims` that borrows data from Rust.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. Because
    /// the data is borrowed from Rust, operations that can change the size of the array (e.g.
    /// `push!`) will fail.
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    pub fn from_slice<'target: 'current, 'current: 'borrow, 'borrow, T, D, Tgt>(
        target: Tgt,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<ArrayResult<'target, 'data, Tgt>>
    where
        T: IntoJulia + ConstructType,
        D: DimsExt,
        Tgt: Target<'target>,
    {
        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        // Safety: The array type is rooted until the array has been constructed, all C API
        // functions are called with valid data. The data-lifetime ensures the data can't be
        // used from Rust after the borrow ends.
        unsafe {
            let callback = || {
                let array_type = D::ArrayContructor::<T>::construct_type(&target).as_value();
                dims.alloc_array_with_data(&target, array_type, data.as_mut_ptr().cast())
                    .ptr()
            };

            let exc = |err: Value| err.unwrap_non_null(Private);
            let res = match catch_exceptions(callback, exc) {
                Ok(array_ptr) => Ok(array_ptr),
                Err(e) => Err(e),
            };

            Ok(target.result_from_ptr(res, Private))
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
    pub unsafe fn from_slice_unchecked<'target, 'current, 'borrow, T, D, Tgt>(
        target: Tgt,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<ArrayData<'target, 'data, Tgt>>
    where
        T: IntoJulia + ConstructType,
        D: DimsExt,
        Tgt: Target<'target>,
    {
        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let array_type = D::ArrayContructor::<T>::construct_type(&target).as_value();
        Ok(dims.alloc_array_with_data(target, array_type, data.as_mut_ptr().cast()))
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
    pub fn from_vec<'target, 'current, 'borrow, T, D, Tgt>(
        target: Tgt,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<ArrayResult<'target, 'static, Tgt>>
    where
        T: IntoJulia + ConstructType,
        D: DimsExt,
        Tgt: Target<'target>,
    {
        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let data = Box::leak(data.into_boxed_slice());

        // Safety: The array type is rooted until the array has been constructed, all C API
        // functions are called with valid data. The data-lifetime ensures the data can't be
        // used from Rust after the borrow ends.
        unsafe {
            let callback = || {
                let array_type = D::ArrayContructor::<T>::construct_type(&target).as_value();
                let array = dims
                    .alloc_array_with_data(&target, array_type, data.as_mut_ptr().cast())
                    .ptr();

                jl_gc_add_ptr_finalizer(
                    get_tls(),
                    array.as_ptr().cast(),
                    droparray::<T> as *mut c_void,
                );

                array
            };

            let exc = |err: Value| err.unwrap_non_null(Private);
            let res = match catch_exceptions(callback, exc) {
                Ok(array_ptr) => Ok(array_ptr),
                Err(e) => Err(e),
            };

            Ok(target.result_from_ptr(res, Private))
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
    pub unsafe fn from_vec_unchecked<'target, 'current, 'borrow, T, D, Tgt>(
        target: Tgt,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<ArrayData<'target, 'static, Tgt>>
    where
        T: IntoJulia + ConstructType,
        D: DimsExt,
        Tgt: Target<'target>,
    {
        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let data = Box::leak(data.into_boxed_slice());
        let array_type = D::ArrayContructor::<T>::construct_type(&target).as_value();
        let array = dims
            .alloc_array_with_data(&target, array_type, data.as_mut_ptr().cast())
            .ptr();

        jl_gc_add_ptr_finalizer(
            get_tls(),
            array.as_ptr().cast(),
            droparray::<T> as *mut c_void,
        );
        Ok(target.data_from_ptr(array, Private))
    }

    /// Convert a string to a Julia array.
    #[inline]
    pub fn from_string<'target, A, Tgt>(target: Tgt, data: A) -> ArrayData<'target, 'static, Tgt>
    where
        A: AsRef<str>,
        Tgt: Target<'target>,
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

    #[inline]
    pub(crate) fn data_ptr(self) -> *mut c_void {
        // Safety: the pointer points to valid data.
        unsafe { self.unwrap_non_null(Private).as_ref().data }
    }
}

impl<'scope, 'data> Array<'scope, 'data> {
    /// Returns the array's dimensions.
    // TODO safety
    #[inline]
    pub unsafe fn dimensions(self) -> ArrayDimensions<'scope> {
        ArrayDimensions::new(self)
    }

    /// Returns the type of this array's elements.
    #[inline]
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
    #[inline]
    pub fn element_size(self) -> usize {
        // Safety: the pointer points to valid data.
        unsafe { self.unwrap_non_null(Private).as_ref().elsize as usize }
    }

    /// Returns `true` if the layout of the elements is compatible with `T`.
    #[inline]
    pub fn contains<T: ValidField>(self) -> bool {
        // Safety: C API function is called valid arguments.
        T::valid_field(self.element_type())
    }

    /// Returns `true` if the layout of the elements is compatible with `T` and these elements are
    /// stored inline.
    #[inline]
    pub fn contains_inline<T: ValidField>(self) -> bool {
        self.contains::<T>() && self.is_inline_array()
    }

    /// Returns `true` if the elements of the array are stored inline.
    #[inline]
    pub fn is_inline_array(self) -> bool {
        // Safety: the pointer points to valid data.
        unsafe { self.unwrap_non_null(Private).as_ref().flags.ptrarray() == 0 }
    }

    /// Returns `true` if the elements of the array are stored inline and the element type is a
    /// union type.
    #[inline]
    pub fn is_union_array(self) -> bool {
        self.is_inline_array() && self.element_type().is::<Union>()
    }

    /// Returns true if the elements of the array are stored inline and at least one of the fields
    /// of the inlined type is a pointer.
    #[inline]
    pub fn has_inlined_pointers(self) -> bool {
        // Safety: the pointer points to valid data.
        unsafe {
            let flags = self.unwrap_non_null(Private).as_ref().flags;
            self.is_inline_array() && flags.hasptr() != 0
        }
    }

    /// Returns `true` if elements of this array are zero-initialized.
    #[inline]
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
    #[inline]
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

    /// Convert this array to a [`TypedValue`].
    pub fn as_typed_value<'target, T: ConstructType, Tgt: Target<'target>, const N: isize>(
        self,
        target: &Tgt,
    ) -> JlrsResult<TypedValue<'scope, 'data, ArrayType<T, N>>> {
        unsafe {
            let ty = T::construct_type(target).as_value();
            let elty = self.element_type();
            if ty != elty {
                // err
                Err(TypeError::IncompatibleType {
                    element_type: elty.display_string_or("<Cannot display type>"),
                    value_type: ty.display_string_or("<Cannot display type>"),
                })?;
            }

            let rank = self.dimensions().rank();
            if rank != N as _ {
                Err(ArrayLayoutError::RankMismatch {
                    found: rank as isize,
                    provided: N,
                })?;
            }

            Ok(TypedValue::<ArrayType<T, N>>::from_value_unchecked(
                self.as_value(),
            ))
        }
    }

    /// Convert this array to a [`TypedValue`] without checking if the layout is compatible.
    #[inline]
    pub unsafe fn as_typed_value_unchecked<T: ConstructType, const N: isize>(
        self,
    ) -> TypedValue<'scope, 'data, ArrayType<T, N>> {
        TypedValue::<ArrayType<T, N>>::from_value_unchecked(self.as_value())
    }

    /// Convert this array to a [`RankedArray`].
    pub fn try_as_ranked<const N: isize>(self) -> JlrsResult<RankedArray<'scope, 'data, N>> {
        unsafe {
            if self.dimensions().rank() == N as usize {
                Ok(RankedArray(self.0, PhantomData, PhantomData))
            } else {
                let value_type = self.element_type().display_string_or(CANNOT_DISPLAY_TYPE);
                Err(AccessError::InvalidLayout { value_type })?
            }
        }
    }

    /// Convert this array to a [`TypedRankedArray`].
    pub fn try_as_typed_ranked<U: ValidField, const N: isize>(
        self,
    ) -> JlrsResult<TypedRankedArray<'scope, 'data, U, N>> {
        unsafe {
            if self.dimensions().rank() == N as usize && self.contains::<U>() {
                Ok(TypedRankedArray(
                    self.0,
                    PhantomData,
                    PhantomData,
                    PhantomData,
                ))
            } else {
                let value_type = self.element_type().display_string_or(CANNOT_DISPLAY_TYPE);
                Err(AccessError::InvalidLayout { value_type })?
            }
        }
    }

    /// Convert this untyped array to a [`TypedArray`] without checking if this conversion is
    /// valid.
    ///
    /// Safety: `T` must be a valid representation of the data stored in the array.
    #[inline]
    pub unsafe fn as_typed_unchecked<T>(self) -> TypedArray<'scope, 'data, T>
    where
        T: ValidField,
    {
        TypedArray::wrap_non_null(self.unwrap_non_null(Private), Private)
    }

    /// Track this array.
    ///
    /// While an array is tracked, it can't be exclusively tracked.
    #[inline]
    pub fn track_shared<'borrow>(
        &'borrow self,
    ) -> JlrsResult<TrackedArray<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow_shared(self.as_value())?;
        unsafe { Ok(TrackedArray::new(self)) }
    }

    /// Exclusively track this array.
    ///
    /// While an array is exclusively tracked, it can't be tracked otherwise.
    #[inline]
    pub unsafe fn track_exclusive<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<TrackedArrayMut<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow_exclusive(self.as_value())?;
        unsafe { Ok(TrackedArrayMut::new(self)) }
    }

    /// Copy the data of an inline array to Rust.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline or `AccessError::InvalidLayout`
    /// if the type of the elements is incorrect.
    pub unsafe fn copy_inline_data<T>(&self) -> JlrsResult<CopiedArray<T>>
    where
        T: 'static + ValidField + Unbox,
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

    /// Immutably access the contents of this array. The elements must have an `isbits` type.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline, `ArrayLayoutError::NotBits`
    /// if the type is not an `isbits` type, or `AccessError::InvalidLayout` if `T` is not a valid
    /// layout for the array elements.
    ///
    /// Safety: it's not checked if the content of this array are already borrowed by Rust code.
    #[inline]
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
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline, `ArrayLayoutError::NotBits`
    /// if the type is not an `isbits` type, or `AccessError::InvalidLayout` if `T` is not a valid
    /// layout for the array elements.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub unsafe fn managed_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ManagedRef<'scope, 'data>,
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
    #[inline]
    pub unsafe fn managed_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ManagedRef<'scope, 'data>,
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub unsafe fn indeterminate_data_mut<'borrow>(
        &'borrow mut self,
    ) -> IndeterminateArrayAccessor<'borrow, 'scope, 'data, Mutable<'borrow, u8>> {
        ArrayAccessor::new(self)
    }

    #[inline]
    pub unsafe fn into_slice_unchecked<T>(self) -> &'scope [T] {
        let len = self.dimensions().size();
        let data = self.data_ptr().cast::<T>();
        std::slice::from_raw_parts(data, len)
    }

    #[inline]
    pub unsafe fn as_slice_unchecked<'borrow, T>(&'borrow self) -> &'borrow [T] {
        let len = self.dimensions().size();
        let data = self.data_ptr().cast::<T>();
        std::slice::from_raw_parts(data, len)
    }

    #[inline]
    pub unsafe fn into_mut_slice_unchecked<T>(self) -> &'scope mut [T] {
        let len = self.dimensions().size();
        let data = self.data_ptr().cast::<T>();
        std::slice::from_raw_parts_mut(data, len)
    }

    #[inline]
    pub unsafe fn as_mut_slice_unchecked<'borrow, T>(&'borrow mut self) -> &'borrow mut [T] {
        let len = self.dimensions().size();
        let data = self.data_ptr().cast::<T>();
        std::slice::from_raw_parts_mut(data, len)
    }

    /// Reshape the array, a new array is returned that has dimensions `dims`. The new array and
    /// `self` share their data.
    ///
    /// This method returns an exception if the old and new array have a different number of
    /// elements.
    pub unsafe fn reshape<'target, 'current, 'borrow, D, Tgt>(
        &self,
        target: Tgt,
        dims: D,
    ) -> ArrayResult<'target, 'data, Tgt>
    where
        D: DimsExt,
        Tgt: Target<'target>,
    {
        target
            .with_local_scope::<_, _, 1>(|target, mut frame| {
                let elty_ptr = self.element_type().unwrap(Private);

                // Safety: The array type is rooted until the array has been constructed, all C API
                // functions are called with valid data. If an exception is thrown it's caught.
                let callback = || {
                    let array_type = jl_apply_array_type(elty_ptr, dims.rank());

                    let tuple = sized_dim_tuple(&mut frame, &dims);

                    jl_reshape_array(array_type, self.unwrap(Private), tuple.unwrap(Private))
                };

                let exc = |err: Value| err.unwrap_non_null(Private);
                let res = match catch_exceptions(callback, exc) {
                    Ok(array_ptr) => Ok(NonNull::new_unchecked(array_ptr)),
                    Err(e) => Err(e),
                };

                Ok(target.result_from_ptr(res, Private))
            })
            .unwrap()
    }

    /// Reshape the array, a new array is returned that has dimensions `dims`. The new array and
    /// `self` share their data.
    ///
    /// Safety: If the dimensions are incompatible with the array size, Julia will throw an error.
    /// This error is not caught, which is UB from a `ccall`ed function.
    pub unsafe fn reshape_unchecked<'target, 'current, 'borrow, D, Tgt>(
        &self,
        target: Tgt,
        dims: D,
    ) -> ArrayData<'target, 'data, Tgt>
    where
        D: DimsExt,
        Tgt: Target<'target>,
    {
        target
            .with_local_scope::<_, _, 1>(|target, mut frame| {
                let elty_ptr = self.element_type().unwrap(Private);
                let array_type = jl_apply_array_type(elty_ptr.cast(), dims.rank());
                let tuple = sized_dim_tuple(&mut frame, &dims);

                let res = jl_reshape_array(array_type, self.unwrap(Private), tuple.unwrap(Private));
                Ok(target.data_from_ptr(NonNull::new_unchecked(res), Private))
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
        T: ManagedRef<'fr, 'da>,
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
    /*
        TODO
        jl_array_ptr_1d_push
        jl_array_ptr_1d_append
    */

    /// Insert `inc` elements at the end of the array.
    ///
    /// The array must be 1D and not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    pub unsafe fn grow_end<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> TargetException<'target, 'static, (), S>
    where
        S: Target<'target>,
    {
        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.

        let callback = || jl_array_grow_end(self.unwrap(Private), inc);

        let exc = |err: Value| err.unwrap_non_null(Private);

        let res = match catch_exceptions(callback, exc) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };

        target.exception_from_ptr(res, Private)
    }

    /// Insert `inc` elements at the end of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    #[inline]
    pub unsafe fn grow_end_unchecked(&mut self, inc: usize) {
        jl_array_grow_end(self.unwrap(Private), inc);
    }

    /// Remove `dec` elements from the end of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    pub unsafe fn del_end<'target, S>(
        &mut self,
        target: S,
        dec: usize,
    ) -> TargetException<'target, 'static, (), S>
    where
        S: Target<'target>,
    {
        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.
        let callback = || jl_array_del_end(self.unwrap(Private), dec);

        let exc = |err: Value| err.unwrap_non_null(Private);

        let res = match catch_exceptions(callback, exc) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };

        target.exception_from_ptr(res, Private)
    }

    /// Remove `dec` elements from the end of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    #[inline]
    pub unsafe fn del_end_unchecked(&mut self, dec: usize) {
        jl_array_del_end(self.unwrap(Private), dec);
    }

    /// Insert `inc` elements at the beginning of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    pub unsafe fn grow_begin<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> TargetException<'target, 'static, (), S>
    where
        S: Target<'target>,
    {
        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.
        let callback = || jl_array_grow_beg(self.unwrap(Private), inc);
        let exc = |err: Value| err.unwrap_non_null(Private);

        let res = match catch_exceptions(callback, exc) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };

        target.exception_from_ptr(res, Private)
    }

    /// Insert `inc` elements at the beginning of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    #[inline]
    pub unsafe fn grow_begin_unchecked(&mut self, inc: usize) {
        jl_array_grow_beg(self.unwrap(Private), inc);
    }

    /// Remove `dec` elements from the beginning of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    pub unsafe fn del_begin<'target, S>(
        &mut self,
        target: S,
        dec: usize,
    ) -> TargetException<'target, 'static, (), S>
    where
        S: Target<'target>,
    {
        // Safety: the C API function is called with valid data. If an exception is thrown it's caught.
        let callback = || jl_array_del_beg(self.unwrap(Private), dec);
        let exc = |err: Value| err.unwrap_non_null(Private);

        let res = match catch_exceptions(callback, exc) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };

        target.exception_from_ptr(res, Private)
    }

    /// Remove `dec` elements from the beginning of the array.
    ///
    /// Safety: the array must be 1D and not contain data borrowed or moved from Rust, otherwise
    /// Julia throws an exception. This error is not exception, which is UB from a `ccall`ed
    /// function.
    #[inline]
    pub unsafe fn del_begin_unchecked(&mut self, dec: usize) {
        jl_array_del_beg(self.unwrap(Private), dec);
    }
}

unsafe impl<'scope, 'data> Typecheck for Array<'scope, 'data> {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        // Safety: Array is a UnionAll. so check if the typenames match
        unsafe { t.type_name() == TypeName::of_array(&Unrooted::new()) }
    }
}

impl_debug!(Array<'_, '_>);

impl<'scope, 'data> ManagedPriv<'scope, 'data> for Array<'scope, 'data> {
    type Wraps = jl_array_t;
    type TypeConstructorPriv<'target, 'da> = Array<'target, 'da>;
    const NAME: &'static str = "Array";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_ccall_arg_managed!(Array, 2);

/// Exactly the same as [`Array`], except it has an explicit element type `T`.
#[repr(transparent)]
pub struct TypedArray<'scope, 'data, T>(
    NonNull<jl_array_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
    PhantomData<T>,
);

impl<'scope, 'data, T> TypedArray<'scope, 'data, T> {
    /// Returns the array's dimensions.
    #[inline]
    pub unsafe fn dimensions(self) -> ArrayDimensions<'scope> {
        self.as_array().dimensions()
    }

    /// Returns the type of this array's elements.
    #[inline]
    pub fn element_type(self) -> Value<'scope, 'static> {
        self.as_array().element_type()
    }

    /// Returns the size of this array's elements.
    #[inline]
    pub fn element_size(self) -> usize {
        self.as_array().element_size()
    }

    /// Convert `self` to `Array`.
    #[inline]
    pub fn as_array(self) -> Array<'scope, 'data> {
        unsafe { Array::wrap_non_null(self.0, Private) }
    }
}

impl<'scope, 'data, U: ValidField> TypedArray<'scope, 'data, U> {
    /// Track this array.
    ///
    /// While an array is tracked, it can't be exclusively tracked.
    #[inline]
    pub fn track_shared<'borrow>(
        &'borrow self,
    ) -> JlrsResult<TrackedArray<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow_shared(self.as_value())?;
        unsafe { Ok(TrackedArray::new(self)) }
    }

    /// Exclusively track this array.
    ///
    /// While an array is exclusively tracked, it can't be tracked otherwise.
    #[inline]
    pub unsafe fn track_exclusive<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<TrackedArrayMut<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow_exclusive(self.as_value())?;
        unsafe { Ok(TrackedArrayMut::new(self)) }
    }
}

impl<'scope, 'data, T: ConstructType> TypedArray<'scope, 'data, T> {
    /// Convert this array to a [`TypedValue`].
    pub fn as_typed_value<const N: isize>(
        self,
    ) -> JlrsResult<TypedValue<'scope, 'data, ArrayType<T, N>>> {
        unsafe {
            let ptr = self.0;
            let rank = self.dimensions().rank();
            if rank != N as _ {
                Err(ArrayLayoutError::RankMismatch {
                    found: rank as isize,
                    provided: N,
                })?;
            }

            Ok(TypedValue::<ArrayType<T, N>>::from_value_unchecked(
                Value::wrap_non_null(ptr.cast(), Private),
            ))
        }
    }

    /// Convert this array to a [`TypedValue`] without checking if the layout is compatible.
    #[inline]
    pub unsafe fn as_typed_value_unchecked<const N: isize>(
        self,
    ) -> TypedValue<'scope, 'data, ArrayType<T, N>> {
        TypedValue::<ArrayType<T, N>>::from_value_unchecked(Value::wrap_non_null(
            self.0.cast(),
            Private,
        ))
    }
}

impl<'scope, 'data, T> Clone for TypedArray<'scope, 'data, T>
where
    T: ValidField,
{
    #[inline]
    fn clone(&self) -> Self {
        unsafe { TypedArray::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }
}

impl<'scope, 'data, T> Copy for TypedArray<'scope, 'data, T> where T: ValidField {}

impl<'data, T> TypedArray<'_, 'data, T>
where
    T: ValidField + IntoJulia + ConstructType,
{
    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. If you
    /// want to create an array for a type that doesn't implement this trait you must use
    /// [`Array::new_for`].
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    #[inline]
    pub fn new<'target, 'current, 'borrow, D, Tgt>(
        target: Tgt,
        dims: D,
    ) -> TypedArrayResult<'target, 'static, Tgt, T>
    where
        D: DimsExt,
        Tgt: Target<'target>,
    {
        unsafe {
            let res = match Array::new::<T, _, _>(&target, dims) {
                Ok(arr) => Ok(arr
                    .as_managed()
                    .as_typed_unchecked::<T>()
                    .unwrap_non_null(Private)),
                Err(e) => Err(e.as_managed().unwrap_non_null(Private)),
            };

            target.result_from_ptr(res, Private)
        }
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method is equivalent to [`Array::new`] except that Julia exceptions are not caught.
    ///
    /// Safety: If the array size is too large, Julia will throw an error. This error is not
    /// caught, which is UB from a `ccall`ed function.
    #[inline]
    pub unsafe fn new_unchecked<'target, 'current, 'borrow, D, Tgt>(
        target: Tgt,
        dims: D,
    ) -> TypedArrayData<'target, 'data, Tgt, T>
    where
        D: DimsExt,
        Tgt: Target<'target>,
    {
        let res = Array::new_unchecked::<T, _, _>(&target, dims)
            .as_managed()
            .as_typed_unchecked::<T>();

        target.data_from_ptr(res.unwrap_non_null(Private), Private)
    }

    /// Create a new n-dimensional Julia array of dimensions `dims` that borrows data from Rust.
    ///
    /// This method can only be used in combination with types that implement `IntoJulia`. Because
    /// the data is borrowed from Rust, operations that can change the size of the array (e.g.
    /// `push!`) will fail.
    ///
    /// If the array size is too large, Julia will throw an error. This error is caught and
    /// returned.
    #[inline]
    pub fn from_slice<'target: 'current, 'current: 'borrow, 'borrow, D, Tgt>(
        target: Tgt,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<TypedArrayResult<'target, 'data, Tgt, T>>
    where
        T: IntoJulia,
        D: DimsExt,
        Tgt: Target<'target>,
    {
        unsafe {
            let res = match Array::from_slice::<T, _, _>(&target, data, dims)? {
                Ok(arr) => Ok(arr
                    .as_managed()
                    .as_typed_unchecked::<T>()
                    .unwrap_non_null(Private)),
                Err(e) => Err(e.as_managed().unwrap_non_null(Private)),
            };

            Ok(target.result_from_ptr(res, Private))
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
    #[inline]
    pub unsafe fn from_slice_unchecked<'target, 'current, 'borrow, D, Tgt>(
        target: Tgt,
        data: &'data mut [T],
        dims: D,
    ) -> JlrsResult<TypedArrayData<'target, 'data, Tgt, T>>
    where
        T: IntoJulia,
        D: DimsExt,
        Tgt: Target<'target>,
    {
        let res = Array::from_slice_unchecked::<T, _, _>(&target, data, dims)?
            .as_managed()
            .as_typed_unchecked::<T>();

        Ok(target.data_from_ptr(res.unwrap_non_null(Private), Private))
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
    #[inline]
    pub fn from_vec<'target, 'current, 'borrow, D, Tgt>(
        target: Tgt,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<TypedArrayResult<'target, 'static, Tgt, T>>
    where
        T: IntoJulia,
        D: DimsExt,
        Tgt: Target<'target>,
    {
        unsafe {
            let res = match Array::from_vec::<T, _, _>(&target, data, dims)? {
                Ok(arr) => Ok(arr
                    .as_managed()
                    .as_typed_unchecked::<T>()
                    .unwrap_non_null(Private)),
                Err(e) => Err(e.as_managed().unwrap_non_null(Private)),
            };

            Ok(target.result_from_ptr(res, Private))
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
    #[inline]
    pub unsafe fn from_vec_unchecked<'target, 'current, 'borrow, D, Tgt>(
        target: Tgt,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<TypedArrayData<'target, 'static, Tgt, T>>
    where
        T: IntoJulia,
        D: DimsExt,
        Tgt: Target<'target>,
    {
        let res = Array::from_vec_unchecked::<T, _, _>(&target, data, dims)?
            .as_managed()
            .as_typed_unchecked::<T>();

        Ok(target.data_from_ptr(res.unwrap_non_null(Private), Private))
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
    pub fn new_for<'target, 'current, 'borrow, D, Tgt>(
        target: Tgt,
        dims: D,
        ty: Value,
    ) -> JlrsResult<TypedArrayResult<'target, 'static, Tgt, T>>
    where
        D: DimsExt,
        Tgt: Target<'target>,
    {
        if !T::valid_field(ty) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        unsafe {
            let res = match Array::new_for(&target, dims, ty) {
                Ok(arr) => Ok(arr
                    .as_managed()
                    .as_typed_unchecked::<T>()
                    .unwrap_non_null(Private)),
                Err(e) => Err(e.as_managed().unwrap_non_null(Private)),
            };

            Ok(target.result_from_ptr(res, Private))
        }
    }

    /// Allocate a new n-dimensional Julia array of dimensions `dims` for data of type `T`.
    ///
    /// This method is equivalent to [`Array::new_for`] except that Julia exceptions are not
    /// caught.
    ///
    /// Safety: If the array size is too large or if the type is invalid, Julia will throw an
    /// error. This error is not caught, which is UB from a `ccall`ed function.
    pub unsafe fn new_for_unchecked<'target, 'current, 'borrow, D, Tgt>(
        target: Tgt,
        dims: D,
        ty: Value,
    ) -> JlrsResult<TypedArrayData<'target, 'static, Tgt, T>>
    where
        D: DimsExt,
        Tgt: Target<'target>,
    {
        if !T::valid_field(ty) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        let res = Array::new_for_unchecked(&target, dims, ty)
            .as_managed()
            .as_typed_unchecked::<T>();

        Ok(target.data_from_ptr(res.unwrap_non_null(Private), Private))
    }
}

impl<'data> TypedArray<'_, 'data, u8> {
    /// Convert a string to a Julia array.
    #[inline]
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
    /// Returns `true` if the elements of the array are stored inline.
    #[inline]
    pub fn is_inline_array(self) -> bool {
        self.as_array().is_inline_array()
    }

    /// Returns true if the elements of the array are stored inline and at least one of the fields
    /// of the inlined type is a pointer.
    #[inline]
    pub fn has_inlined_pointers(self) -> bool {
        self.as_array().has_inlined_pointers()
    }

    /// Returns `true` if elements of this array are zero-initialized.
    #[inline]
    pub fn zero_init(self) -> bool {
        self.as_array().zero_init()
    }

    /// Returns true if the elements of the array are stored as [`Value`]s.
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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

    /// Convert this array to a [`RankedArray`].
    pub fn try_as_ranked<const N: isize>(self) -> JlrsResult<RankedArray<'scope, 'data, N>> {
        unsafe {
            if self.dimensions().rank() == N as usize {
                Ok(RankedArray(self.0, PhantomData, PhantomData))
            } else {
                let value_type = self.element_type().display_string_or(CANNOT_DISPLAY_TYPE);
                Err(AccessError::InvalidLayout { value_type })?
            }
        }
    }

    /// Convert this array to a [`TypedRankedArray`].
    pub fn try_as_typed_ranked<U: ValidField, const N: isize>(
        self,
    ) -> JlrsResult<TypedRankedArray<'scope, 'data, U, N>> {
        unsafe {
            if self.dimensions().rank() == N as usize {
                Ok(TypedRankedArray(
                    self.0,
                    PhantomData,
                    PhantomData,
                    PhantomData,
                ))
            } else {
                let value_type = self.element_type().display_string_or(CANNOT_DISPLAY_TYPE);
                Err(AccessError::InvalidLayout { value_type })?
            }
        }
    }

    /// Convert `self` to `Array`.
    #[inline]
    pub fn as_array_ref(&self) -> &Array<'scope, 'data> {
        unsafe { std::mem::transmute(self) }
    }

    /// Reshape the array, a new array is returned that has dimensions `dims`. The new array and
    /// `self` share their data.
    ///
    /// This method returns an exception if the old and new array have a different number of
    /// elements.
    #[inline]
    pub unsafe fn reshape<'target, 'current, 'borrow, D, Tgt>(
        &self,
        target: Tgt,
        dims: D,
    ) -> TypedArrayResult<'target, 'data, Tgt, T>
    where
        D: DimsExt,
        Tgt: Target<'target>,
    {
        let res = match self.as_array().reshape(&target, dims) {
            Ok(arr) => Ok(arr
                .as_managed()
                .as_typed_unchecked::<T>()
                .unwrap_non_null(Private)),
            Err(e) => Err(e.as_managed().unwrap_non_null(Private)),
        };

        target.result_from_ptr(res, Private)
    }

    /// Reshape the array, a new array is returned that has dimensions `dims`. The new array and
    /// `self` share their data.
    ///
    /// Safety: If the dimensions are incompatible with the array size, Julia will throw an error.
    /// This error is not caught, which is UB from a `ccall`ed function.
    #[inline]
    pub unsafe fn reshape_unchecked<'target, 'current, 'borrow, D, Tgt>(
        self,
        target: Tgt,
        dims: D,
    ) -> TypedArrayData<'target, 'data, Tgt, T>
    where
        D: DimsExt,
        Tgt: Target<'target>,
    {
        let res = self
            .as_array()
            .reshape_unchecked(&target, dims)
            .as_managed()
            .as_typed_unchecked::<T>()
            .unwrap_non_null(Private);

        target.data_from_ptr(res, Private)
    }

    /// Immutably access the contents of this array.
    ///
    /// You can borrow data from multiple arrays at the same time.
    #[inline]
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
    #[inline]
    pub unsafe fn indeterminate_data_mut<'borrow>(
        &'borrow mut self,
    ) -> IndeterminateArrayAccessor<'borrow, 'scope, 'data, Mutable<'borrow, u8>> {
        // Safety: layouts are compatible, access is immutable.
        ArrayAccessor::new(self.as_array_ref())
    }
}

impl<'scope, 'data, T> TypedArray<'scope, 'data, Option<T>>
where
    T: ManagedRef<'scope, 'data>,
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
    #[inline]
    pub unsafe fn managed_data<'borrow>(
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
    #[inline]
    pub unsafe fn managed_data_mut<'borrow>(
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub unsafe fn grow_end<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> TargetException<'target, 'static, (), S>
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
    #[inline]
    pub unsafe fn grow_end_unchecked(&mut self, inc: usize) {
        self.as_array().grow_end_unchecked(inc)
    }

    /// Remove `dec` elements from the end of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[inline]
    pub unsafe fn del_end<'target, S>(
        &mut self,
        target: S,
        dec: usize,
    ) -> TargetException<'target, 'static, (), S>
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
    #[inline]
    pub unsafe fn del_end_unchecked(&mut self, dec: usize) {
        self.as_array().del_end_unchecked(dec)
    }

    /// Insert `inc` elements at the beginning of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[inline]
    pub unsafe fn grow_begin<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> TargetException<'target, 'static, (), S>
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
    #[inline]
    pub unsafe fn grow_begin_unchecked(&mut self, inc: usize) {
        self.as_array().grow_begin_unchecked(inc)
    }

    /// Remove `dec` elements from the beginning of the array.
    ///
    /// The array must be 1D, not contain data borrowed or moved from Rust, otherwise an exception
    /// is returned.
    #[inline]
    pub unsafe fn del_begin<'target, S>(
        &mut self,
        target: S,
        dec: usize,
    ) -> TargetException<'target, 'static, (), S>
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
    #[inline]
    pub unsafe fn del_begin_unchecked(&mut self, dec: usize) {
        self.as_array().del_begin_unchecked(dec)
    }
}

unsafe impl<'scope, 'data, T: ValidField> Typecheck for TypedArray<'scope, 'data, T> {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        // Safety: borrow is only temporary
        unsafe {
            t.is::<Array>()
                && T::valid_field(t.parameters().data().as_slice()[0].unwrap().as_value())
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

impl<'scope, 'data, T: ValidField> ManagedPriv<'scope, 'data> for TypedArray<'scope, 'data, T> {
    type Wraps = jl_array_t;
    type TypeConstructorPriv<'target, 'da> = TypedArray<'target, 'da, T>;
    const NAME: &'static str = "Array";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it. T must be correct
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub(self) struct AssumeThreadsafe<T>(T);

unsafe impl<T> Send for AssumeThreadsafe<T> {}
unsafe impl<T> Sync for AssumeThreadsafe<T> {}

#[inline]
pub(self) fn sized_dim_tuple<'target, D, Tgt>(
    target: Tgt,
    dims: &D,
) -> ValueData<'target, 'static, Tgt>
where
    D: DimsExt,
    Tgt: Target<'target>,
{
    unsafe {
        let rank = dims.rank();
        let dims_type = dims.dimension_object(&target).as_managed();
        let tuple = jl_new_struct_uninit(dims_type.unwrap(Private));

        {
            let slice = std::slice::from_raw_parts_mut(tuple as *mut MaybeUninit<usize>, rank);
            dims.fill_tuple(slice, Private);
        }

        Value::wrap_non_null(NonNull::new_unchecked(tuple), Private).root(target)
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

/// A reference to a [`Array`] that has not been explicitly rooted.
pub type ArrayRef<'scope, 'data> = Ref<'scope, 'data, Array<'scope, 'data>>;

/// An [`Array`] with static lifetimes.
///
/// This is a useful shorthand for signatures of `ccall`able functions that take a [`Array`].
///
/// See [`TypedArrayUnbound`] for more information.
pub type ArrayUnbound = Array<'static, 'static>;

impl ArrayUnbound {
    /// Track this array.
    ///
    /// While an array is tracked, it can't be exclusively tracked.
    #[inline]
    pub fn track_shared_unbound(self) -> JlrsResult<TrackedArray<'static, 'static, 'static, Self>> {
        Ledger::try_borrow_shared(self.as_value())?;
        unsafe { Ok(TrackedArray::new_from_owned(self)) }
    }

    /// Exclusively track this array.
    ///
    /// While an array is exclusively tracked, it can't be tracked otherwise.
    #[inline]
    pub unsafe fn track_exclusive_unbound(
        self,
    ) -> JlrsResult<TrackedArrayMut<'static, 'static, 'static, Self>> {
        Ledger::try_borrow_exclusive(self.as_value())?;
        unsafe { Ok(TrackedArrayMut::new_from_owned(self)) }
    }
}

/// A [`ArrayRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`Array`].
pub type ArrayRet = Ref<'static, 'static, Array<'static, 'static>>;

unsafe impl ConstructType for Array<'_, '_> {
    #[inline]
    fn construct_type_uncached<'target, Tgt>(
        target: Tgt,
    ) -> super::value::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        UnionAll::array_type(&target).as_value().root(target)
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(UnionAll::array_type(&target).as_value())
    }

    type Static = Array<'static, 'static>;
}

unsafe impl ValidLayout for ArrayRef<'_, '_> {
    #[inline]
    fn valid_layout(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            dt.is::<Array>()
        } else if v.is::<UnionAll>() {
            let ua = unsafe { v.cast_unchecked::<UnionAll>() };
            ua.base_type().is::<Array>()
        } else {
            false
        }
    }

    #[inline]
    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        UnionAll::array_type(target).as_value()
    }

    const IS_REF: bool = true;
}

unsafe impl ValidField for Option<ArrayRef<'_, '_>> {
    #[inline]
    fn valid_field(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            dt.is::<Array>()
        } else if v.is::<UnionAll>() {
            let ua = unsafe { v.cast_unchecked::<UnionAll>() };
            ua.base_type().is::<Array>()
        } else {
            false
        }
    }
}

/// A reference to an [`TypedArray`] that has not been explicitly rooted.
pub type TypedArrayRef<'scope, 'data, T> = Ref<'scope, 'data, TypedArray<'scope, 'data, T>>;

/// A [`TypedArray`] with static lifetimes.
///
/// This is a useful shorthand for signatures of `ccall`able functions that take a [`TypedArray`]
/// and operate on its contents in another thread. Example:
///
/// ```
/// use jlrs::{ccall::AsyncCallback, data::managed::array::TypedArrayUnbound, prelude::*};
///
/// fn sum_dispatched(data: TypedArrayUnbound<f32>) -> JlrsResult<impl AsyncCallback<f32>> {
///     let tracked = data.track_shared_unbound()?;
///     Ok(move || Ok(tracked.as_slice().iter().sum()))
/// }
///
/// julia_module! {
///     become init_fn_name;
///
///     async fn sum_dispatched(
///         data: TypedArrayUnbound<f32>
///     ) -> JlrsResult<impl AsyncCallback<f32>>;
/// }
/// ```
///
/// In order for `tracked` to be moved to another thread, all lifetimes of `data` must be
/// `'static`. The generated Julia function guarantees that the array won't be freed by the GC
/// until the dispatched callback has completed.
pub type TypedArrayUnbound<T> = TypedArray<'static, 'static, T>;

impl<T: ValidField> TypedArrayUnbound<T> {
    /// Track this array.
    ///
    /// While an array is tracked, it can't be exclusively tracked.
    #[inline]
    pub fn track_shared_unbound(self) -> JlrsResult<TrackedArray<'static, 'static, 'static, Self>> {
        Ledger::try_borrow_shared(self.as_value())?;
        unsafe { Ok(TrackedArray::new_from_owned(self)) }
    }

    /// Exclusively track this array.
    ///
    /// While an array is exclusively tracked, it can't be tracked otherwise.
    #[inline]
    pub unsafe fn track_exclusive_unbound(
        self,
    ) -> JlrsResult<TrackedArrayMut<'static, 'static, 'static, Self>> {
        Ledger::try_borrow_exclusive(self.as_value())?;
        unsafe { Ok(TrackedArrayMut::new_from_owned(self)) }
    }
}

/// A [`TypedArrayRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`TypedArray`].
pub type TypedArrayRet<T> = Ref<'static, 'static, TypedArray<'static, 'static, T>>;

unsafe impl<T: ValidField> ValidLayout for TypedArrayRef<'_, '_, T> {
    #[inline]
    fn valid_layout(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            dt.is::<TypedArray<T>>()
        } else if v.is::<UnionAll>() {
            let ua = unsafe { v.cast_unchecked::<UnionAll>() };
            ua.base_type().is::<TypedArray<T>>()
        } else {
            false
        }
    }

    #[inline]
    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        UnionAll::array_type(target).as_value()
    }

    const IS_REF: bool = true;
}

unsafe impl<T: ValidField> ValidField for Option<TypedArrayRef<'_, '_, T>> {
    #[inline]
    fn valid_field(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            dt.is::<TypedArray<T>>()
        } else if v.is::<UnionAll>() {
            let ua = unsafe { v.cast_unchecked::<UnionAll>() };
            ua.base_type().is::<TypedArray<T>>()
        } else {
            false
        }
    }
}

use crate::memory::target::TargetType;

/// `Array` or `ArrayRef`, depending on the target type `T`.
pub type ArrayData<'target, 'data, T> =
    <T as TargetType<'target>>::Data<'data, Array<'target, 'data>>;

/// `JuliaResult<Array>` or `JuliaResultRef<ArrayRef>`, depending on the target type `T`.
pub type ArrayResult<'target, 'data, T> = TargetResult<'target, 'data, Array<'target, 'data>, T>;

/// `TypedArray<U>` or `TypedArrayRef<U>`, depending on the target type `T`.
pub type TypedArrayData<'target, 'data, T, U> =
    <T as TargetType<'target>>::Data<'data, TypedArray<'target, 'data, U>>;

/// `JuliaResult<TypedArray<U>>` or `JuliaResultRef<TypedArrayRef<U>>`, depending on the target
/// type `T`.
pub type TypedArrayResult<'target, 'data, T, U> =
    TargetResult<'target, 'data, TypedArray<'target, 'data, U>, T>;

unsafe impl<'scope, 'data, T: ValidField + ConstructType> CCallArg
    for TypedArray<'scope, 'data, T>
{
    type CCallArgType = Value<'scope, 'data>;
    type FunctionArgType =
        TypedValue<'scope, 'static, ArrayTypeConstructor<T, TypeVarConstructor<Name<'N'>>>>;
}

unsafe impl<T: ValidField + ConstructType> CCallReturn for TypedArrayRet<T> {
    type CCallReturnType = Value<'static, 'static>;
    type FunctionReturnType =
        TypedValue<'static, 'static, ArrayTypeConstructor<T, TypeVarConstructor<Name<'N'>>>>;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}

/// An array with a definite rank.
#[repr(transparent)]
pub struct RankedArray<'scope, 'data, const N: isize>(
    NonNull<jl_array_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data mut ()>,
);

impl<'scope, 'data, const N: isize> RankedArray<'scope, 'data, N> {
    /// Convert `self` to `Array`.
    #[inline]
    pub fn as_array(self) -> Array<'scope, 'data> {
        unsafe { Array::wrap_non_null(self.0, Private) }
    }

    /// Convert this array to a [`TypedValue`].
    pub fn as_typed_value<'target, T: ConstructType, Tgt: Target<'target>>(
        self,
        target: &Tgt,
    ) -> JlrsResult<TypedValue<'scope, 'data, ArrayType<T, N>>> {
        target.local_scope::<_, _, 1>(|frame| {
            let ty = T::construct_type(frame);
            let arr = self.as_array();
            let elty = arr.element_type();
            if ty != elty {
                // err
                Err(TypeError::IncompatibleType {
                    element_type: elty.display_string_or("<Cannot display type>"),
                    value_type: ty.display_string_or("<Cannot display type>"),
                })?;
            }

            unsafe {
                Ok(TypedValue::<ArrayType<T, N>>::from_value_unchecked(
                    self.as_value(),
                ))
            }
        })
    }

    /// Convert this array to a [`TypedValue`] without checking if the layout is compatible.
    #[inline]
    pub unsafe fn as_typed_value_unchecked<T: ConstructType>(
        self,
    ) -> TypedValue<'scope, 'data, ArrayType<T, N>> {
        TypedValue::<ArrayType<T, N>>::from_value_unchecked(self.as_value())
    }
}

impl<const N: isize> Clone for RankedArray<'_, '_, N> {
    #[inline]
    fn clone(&self) -> Self {
        RankedArray(self.0, PhantomData, PhantomData)
    }
}

impl<const N: isize> Copy for RankedArray<'_, '_, N> {}

impl<'scope, 'data, const N: isize> ManagedPriv<'scope, 'data> for RankedArray<'scope, 'data, N> {
    type Wraps = jl_array_t;
    type TypeConstructorPriv<'target, 'da> = RankedArray<'target, 'da, N>;
    const NAME: &'static str = "Array";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

unsafe impl<const N: isize> Typecheck for RankedArray<'_, '_, N> {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        // Safety: Array is a UnionAll. so check if the typenames match
        unsafe {
            if !Array::typecheck(t) {
                return false;
            }

            let unrooted = t.unrooted_target();
            if let Some(param) = t.parameter(unrooted, 1) {
                if let Ok(rank) = param.as_value().unbox::<isize>() {
                    rank == N
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

impl<const N: isize> Debug for RankedArray<'_, '_, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.display_string() {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "<Cannot display value: {}>", e),
        }
    }
}

/// A reference to an [`RankedArray`] that has not been explicitly rooted.
pub type RankedArrayRef<'scope, 'data, const N: isize> =
    Ref<'scope, 'data, RankedArray<'scope, 'data, N>>;

/// A [`RankedArrayRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`RankedArray`].
pub type RankedArrayRet<const N: isize> = Ref<'static, 'static, RankedArray<'static, 'static, N>>;

unsafe impl<const N: isize> ValidLayout for RankedArrayRef<'_, '_, N> {
    #[inline]
    fn valid_layout(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            dt.is::<RankedArray<N>>()
        } else if v.is::<UnionAll>() {
            let ua = unsafe { v.cast_unchecked::<UnionAll>() };
            ua.base_type().is::<RankedArray<N>>()
        } else {
            false
        }
    }

    #[inline]
    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        UnionAll::array_type(target).as_value()
    }

    const IS_REF: bool = true;
}

unsafe impl<const N: isize> ValidField for Option<RankedArrayRef<'_, '_, N>> {
    #[inline]
    fn valid_field(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            dt.is::<RankedArray<N>>()
        } else if v.is::<UnionAll>() {
            let ua = unsafe { v.cast_unchecked::<UnionAll>() };
            ua.base_type().is::<RankedArray<N>>()
        } else {
            false
        }
    }
}

unsafe impl<'scope, 'data, const N: isize> CCallArg for RankedArray<'scope, 'data, N> {
    type CCallArgType = Value<'scope, 'data>;
    type FunctionArgType = ArrayTypeConstructor<TypeVarConstructor<Name<'T'>>, ConstantIsize<N>>;
}

unsafe impl<'scope, 'data, const N: isize> CCallReturn for RankedArray<'scope, 'data, N> {
    type CCallReturnType = Value<'static, 'static>;
    type FunctionReturnType = ArrayTypeConstructor<TypeVarConstructor<Name<'T'>>, ConstantIsize<N>>;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}

/// An array with a set element type and a definite rank.
#[repr(transparent)]
pub struct TypedRankedArray<'scope, 'data, U, const N: isize>(
    NonNull<jl_array_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data mut ()>,
    PhantomData<U>,
);

impl<'scope, 'data, U: ConstructType, const N: isize> TypedRankedArray<'scope, 'data, U, N> {
    #[inline]
    pub fn as_typed(self) -> TypedArray<'scope, 'data, U> {
        TypedArray(self.0, PhantomData, PhantomData, PhantomData)
    }

    /// Convert `self` to `Array`.
    #[inline]
    pub fn as_array(self) -> Array<'scope, 'data> {
        unsafe { Array::wrap_non_null(self.0, Private) }
    }

    /// Convert this array to a [`TypedValue`].
    pub fn as_typed_value<'target, Tgt: Target<'target>>(
        self,
        target: &Tgt,
    ) -> JlrsResult<TypedValue<'scope, 'data, ArrayType<U, N>>> {
        target.local_scope::<_, _, 1>(|frame| {
            let ty = U::construct_type(frame);
            let arr = self.as_array();
            let elty = arr.element_type();
            if ty != elty {
                // err
                Err(TypeError::IncompatibleType {
                    element_type: elty.display_string_or("<Cannot display type>"),
                    value_type: ty.display_string_or("<Cannot display type>"),
                })?;
            }

            unsafe {
                Ok(TypedValue::<ArrayType<U, N>>::from_value_unchecked(
                    arr.as_value(),
                ))
            }
        })
    }

    /// Convert this array to a [`TypedValue`] without checking if the layout is compatible.
    #[inline]
    pub unsafe fn as_typed_value_unchecked(self) -> TypedValue<'scope, 'data, ArrayType<U, N>> {
        let arr = self.as_array();
        TypedValue::<ArrayType<U, N>>::from_value_unchecked(arr.as_value())
    }
}

impl<U, const N: isize> Clone for TypedRankedArray<'_, '_, U, N> {
    #[inline]
    fn clone(&self) -> Self {
        TypedRankedArray(self.0, PhantomData, PhantomData, PhantomData)
    }
}

impl<U, const N: isize> Copy for TypedRankedArray<'_, '_, U, N> {}

unsafe impl<U: ConstructType + ValidField, const N: isize> ConstructType
    for TypedRankedArray<'_, '_, U, N>
{
    type Static = ArrayTypeConstructor<U::Static, ConstantIsize<N>>;

    // TODO
    #[inline]
    fn construct_type_uncached<'target, Tgt>(
        target: Tgt,
    ) -> super::value::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target
            .with_local_scope::<_, _, 4>(|target, mut frame| {
                let ua = UnionAll::array_type(&target);
                unsafe {
                    let inner = ua.body();
                    let ty_var = ua.var();
                    let elem_ty = U::construct_type(&mut frame);
                    let rank = Value::new(&mut frame, N);
                    let with_rank = inner.apply_type_unchecked(&mut frame, [rank]);

                    let rewrap = UnionAll::new_unchecked(&mut frame, ty_var, with_rank);
                    let ty = rewrap
                        .apply_type_unchecked(&target, [elem_ty])
                        .as_value()
                        .root(target);
                    Ok(ty)
                }
            })
            .unwrap()
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(UnionAll::array_type(&target).as_value())
    }
}

unsafe impl<U: ValidField, const N: isize> Typecheck for TypedRankedArray<'_, '_, U, N> {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        // Safety: Array is a UnionAll. so check if the typenames match
        unsafe {
            if !TypedArray::<U>::typecheck(t) {
                return false;
            }

            let unrooted = t.unrooted_target();
            if let Some(param) = t.parameter(unrooted, 1) {
                if let Ok(rank) = param.as_value().unbox::<isize>() {
                    rank == N
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

impl<U: ValidField, const N: isize> Debug for TypedRankedArray<'_, '_, U, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.display_string() {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "<Cannot display value: {}>", e),
        }
    }
}

impl<'scope, 'data, U: ValidField, const N: isize> ManagedPriv<'scope, 'data>
    for TypedRankedArray<'scope, 'data, U, N>
{
    type Wraps = jl_array_t;
    type TypeConstructorPriv<'target, 'da> = TypedRankedArray<'target, 'da, U, N>;
    const NAME: &'static str = "Array";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it. T must be correct
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// A reference to an [`RankedArray`] that has not been explicitly rooted.
pub type TypedRankedArrayRef<'scope, 'data, U, const N: isize> =
    Ref<'scope, 'data, TypedRankedArray<'scope, 'data, U, N>>;

/// A [`TypedRankedArrayRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`TypedRankedArray`].
pub type TypedRankedArrayRet<U, const N: isize> =
    Ref<'static, 'static, TypedRankedArray<'static, 'static, U, N>>;

/// A [`TypedRankedArray`] with static lifetimes.
///
/// This is a useful shorthand for signatures of `ccall`able functions that take a [`Array`].
///
/// See [`TypedArrayUnbound`] for more information.
pub type TypedRankedArrayUnbound<U, const N: isize> = TypedRankedArray<'static, 'static, U, N>;

unsafe impl<U: ValidField, const N: isize> ValidLayout for TypedRankedArrayRef<'_, '_, U, N> {
    #[inline]
    fn valid_layout(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            dt.is::<TypedRankedArray<U, N>>()
        } else if v.is::<UnionAll>() {
            let ua = unsafe { v.cast_unchecked::<UnionAll>() };
            ua.base_type().is::<TypedRankedArray<U, N>>()
        } else {
            false
        }
    }

    #[inline]
    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        UnionAll::array_type(target).as_value()
    }

    const IS_REF: bool = true;
}

unsafe impl<U: ValidField, const N: isize> ValidField
    for Option<TypedRankedArrayRef<'_, '_, U, N>>
{
    #[inline]
    fn valid_field(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<Array>()
        } else if let Ok(ua) = v.cast::<UnionAll>() {
            ua.base_type().is::<TypedRankedArray<U, N>>()
        } else {
            false
        }
    }
}

unsafe impl<'scope, 'data, U: ValidField + ConstructType, const N: isize> CCallArg
    for TypedRankedArray<'scope, 'data, U, N>
{
    type CCallArgType = TypedRankedArray<'scope, 'data, U, N>;
    type FunctionArgType = TypedRankedArray<'scope, 'data, U, N>;
}

unsafe impl<'scope, 'data, U: ValidField + ConstructType, const N: isize> CCallReturn
    for TypedRankedArrayRef<'scope, 'data, U, N>
{
    type CCallReturnType = Value<'static, 'static>;
    type FunctionReturnType = TypedRankedArray<'scope, 'data, U, N>;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}

// TODO: conversions

/// Alias for `ArrayTypeConstructor<T, ConstantIsize<N>>`.
pub type ArrayType<T, const N: isize> = ArrayTypeConstructor<T, ConstantIsize<N>>;
