//! N-dimensional indexing.
//!
//! See [`Dims`] for more information.

use std::{
    cell::Cell,
    ffi::c_void,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};

#[julia_version(since = "1.11")]
use jl_sys::jl_alloc_array_nd;
#[julia_version(until = "1.10")]
use jl_sys::jl_new_array;
use jl_sys::{
    jl_alloc_array_1d, jl_alloc_array_2d, jl_alloc_array_3d, jl_apply_array_type, jl_array_t,
    jl_ptr_to_array, jl_ptr_to_array_1d, jl_value_t, jlrs_dimtuple_type,
};
use jlrs_macros::julia_version;

use super::{sized_dim_tuple, unsized_dim_tuple, ArrayData};
use crate::{
    data::{
        managed::{datatype::DataTypeData, private::ManagedPriv as _},
        types::construct_type::{ArrayTypeConstructor, ConstantIsize, ConstantSize, ConstructType},
    },
    memory::scope::LocalScope,
    prelude::{Array, Managed, Target, Value, ValueData},
    private::Private,
};

/// Trait implemented by n-dimensional indices.
///
/// Instances of types that implement this trait can be used to index into n-dimensional Julia
/// arrays like [`Array`] and [`TypedRankedArray`]. Indexing starts at 0.
///
/// When indexing into an array, the rank of the index must match the rank of the array. A 0d
/// array can only be accessed with a 0d index, a 1d array with a 1d index, and so on. If the rank
/// of the index and array are both known at compile time, this will be checked at compile time,
/// otherwise this is checked at runtime.
///
/// The following index types have a known rank at compile time:
///
/// - `usize` (rank 1)
///
/// - `()`, `(usize,)`, ..., `(usize, usize, usize, usize)` (rank 0..=4)
///
/// - `[usize; N]`, `&[usize; N]` (rank `N`)
///
/// These index types require a runtime rank check:
///
/// - `&[usize]`
///
/// - [`ArrayDimensions`]
///
/// - [`Dimensions`]
///
/// Safety: implementations of this trait must only implement `RANK`, `SlicingType`, `to_slicer`,
/// `rank` and `n_elements_unchecked`, default implementations must not be overridden.
///
/// [`TypedRankedArray`]: crate::data::managed::array::TypedRankedArray
pub unsafe trait Dims: Sized + Debug {
    /// The rank of `Self` if `RANK >= 0`, or `-1` if the rank is not known at compile-time.
    const RANK: isize;

    /// This constant is `true` if the rank of these dimensions are unknown at compile-time, i.e.
    /// if `Self::RANK == -1`.
    const UNKNOWN_RANK: bool = Self::RANK == -1;

    /// A type that can express these dimensions as a contiguous slice.
    type SlicingType<'s>: DimSlice
    where
        Self: 's;

    /// Convert `self` to `Dimensions`.
    #[inline]
    fn to_dimensions(&self) -> Dimensions {
        Dimensions::from_dims(self)
    }

    /// Calculate the linear index for `dim_index` in an array with dimensions `self`.
    fn index_of<D: Dims>(&self, dim_index: &D) -> Option<usize> {
        // Assert that the indices are compatible
        // Check necessary if:
        // - the rank of Self is -1
        // - the rank of dim_index is -1
        // Not sure if N matters here
        let _: () = <(Self, D) as CompatibleIndices<Self, D>>::ASSERT_COMPATIBLE;

        if Self::UNKNOWN_RANK || D::UNKNOWN_RANK {
            let n_dims = self.rank();
            if n_dims != dim_index.rank() {
                return None;
            }

            if n_dims == 0 {
                return Some(0);
            }

            unsafe {
                for dim in 0..n_dims {
                    if self.n_elements_unchecked(dim) <= dim_index.n_elements_unchecked(dim) {
                        return None;
                    }
                }

                let init = dim_index.n_elements_unchecked(n_dims - 1);
                let idx = (0..n_dims - 1).rev().fold(init, |idx_acc, dim| {
                    idx_acc * self.n_elements_unchecked(dim) + dim_index.n_elements_unchecked(dim)
                });

                Some(idx)
            }
        } else {
            if Self::RANK == 0 {
                return Some(0);
            }

            unsafe {
                for dim in 0..Self::RANK as usize {
                    if self.n_elements_unchecked(dim) <= dim_index.n_elements_unchecked(dim) {
                        return None;
                    }
                }

                let init = dim_index.n_elements_unchecked(Self::RANK as usize - 1);
                let idx = (0..Self::RANK as usize - 1)
                    .rev()
                    .fold(init, |idx_acc, dim| {
                        idx_acc * self.n_elements_unchecked(dim)
                            + dim_index.n_elements_unchecked(dim)
                    });

                Some(idx)
            }
        }
    }

    /// Calculate the linear index for `dim_index` in an array with dimensions `self` without
    /// checking any bounds.
    ///
    /// The default implementation should not be overridden.
    ///
    /// Safety: `dim_index` must be in-bounds of `self`.
    unsafe fn index_of_unchecked<D: Dims>(&self, dim_index: &D) -> usize {
        // Assert that the indices are compatible
        let _: () = <(Self, D) as CompatibleIndices<Self, D>>::ASSERT_COMPATIBLE;

        let rank = self.rank();
        if rank == 0 {
            return 0;
        }

        let init = dim_index.n_elements_unchecked(rank - 1);
        let idx = (0..rank - 1).rev().fold(init, |idx_acc, dim| {
            idx_acc * self.n_elements_unchecked(dim) + dim_index.n_elements_unchecked(dim)
        });

        idx
    }

    /// Returns the rank of `self`.
    fn rank(&self) -> usize;

    /// Returns the number of elements of the nth dimension. Indexing starts at 0.
    #[inline]
    fn n_elements(&self, dimension: usize) -> Option<usize> {
        if dimension >= self.rank() {
            if dimension == 0 {
                return Some(0);
            }

            return None;
        }

        unsafe { Some(self.n_elements_unchecked(dimension)) }
    }

    /// Returns the number of elements of the nth dimension without checkings bounds. Indexing
    /// starts at 0.
    ///
    /// Safety: the dimension must be in-bounds, implementations of this function should not do
    /// any bounds checking because that is handled by the default implementation of `n_elements`.
    unsafe fn n_elements_unchecked(&self, dimension: usize) -> usize;

    /// The total number of elements in the array, i.e. the product of the number of elements of
    /// each dimension.
    fn size(&self) -> usize {
        (0..self.rank())
            .map(|i| unsafe { self.n_elements_unchecked(i) })
            .product()
    }

    /// Convert `self` to a type that can be sliced.
    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s>;
}

unsafe impl Dims for usize {
    const RANK: isize = 1;

    type SlicingType<'s> = [usize; 1];

    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s> {
        [*self]
    }

    #[inline]
    fn rank(&self) -> usize {
        1
    }

    #[inline]
    unsafe fn n_elements_unchecked(&self, _dimension: usize) -> usize {
        *self
    }
}

unsafe impl Dims for () {
    const RANK: isize = 0;

    type SlicingType<'s> = Self;

    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s> {
        *self
    }

    #[inline]
    fn rank(&self) -> usize {
        0
    }

    #[inline]
    unsafe fn n_elements_unchecked(&self, _dimension: usize) -> usize {
        0
    }
}

unsafe impl Dims for (usize,) {
    const RANK: isize = 1;

    type SlicingType<'s> = [usize; 1];

    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s> {
        [self.0]
    }

    #[inline]
    fn rank(&self) -> usize {
        1
    }

    #[inline]
    unsafe fn n_elements_unchecked(&self, _dimension: usize) -> usize {
        self.0
    }
}

unsafe impl Dims for (usize, usize) {
    const RANK: isize = 2;

    type SlicingType<'s> = [usize; 2];

    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s> {
        [self.0, self.1]
    }

    #[inline]
    fn rank(&self) -> usize {
        2
    }

    #[inline]
    unsafe fn n_elements_unchecked(&self, dimension: usize) -> usize {
        if dimension == 0 {
            self.0
        } else {
            self.1
        }
    }
}

unsafe impl Dims for (usize, usize, usize) {
    const RANK: isize = 3;

    type SlicingType<'s> = [usize; 3];

    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s> {
        [self.0, self.1, self.2]
    }

    #[inline]
    fn rank(&self) -> usize {
        3
    }

    #[inline]
    unsafe fn n_elements_unchecked(&self, dimension: usize) -> usize {
        match dimension {
            0 => self.0,
            1 => self.1,
            _ => self.2,
        }
    }
}

unsafe impl Dims for (usize, usize, usize, usize) {
    const RANK: isize = 4;

    type SlicingType<'s> = [usize; 4];

    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s> {
        [self.0, self.1, self.2, self.3]
    }

    #[inline]
    fn rank(&self) -> usize {
        4
    }

    #[inline]
    unsafe fn n_elements_unchecked(&self, dimension: usize) -> usize {
        match dimension {
            0 => self.0,
            1 => self.1,
            2 => self.2,
            _ => self.3,
        }
    }
}

unsafe impl<const N: usize> Dims for [usize; N] {
    const RANK: isize = N as isize;

    type SlicingType<'s> = &'s Self;

    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s> {
        self
    }

    #[inline]
    fn rank(&self) -> usize {
        N
    }

    #[inline]
    unsafe fn n_elements_unchecked(&self, dimension: usize) -> usize {
        *self.get_unchecked(dimension)
    }
}

unsafe impl<const N: usize> Dims for &[usize; N] {
    const RANK: isize = N as isize;

    type SlicingType<'s> = Self where Self: 's;

    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s> {
        self
    }

    #[inline]
    fn rank(&self) -> usize {
        N
    }

    #[inline]
    unsafe fn n_elements_unchecked(&self, dimension: usize) -> usize {
        *self.get_unchecked(dimension)
    }
}

unsafe impl Dims for &[usize] {
    const RANK: isize = -1;

    type SlicingType<'s> = Self where Self: 's;

    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s> {
        self
    }

    #[inline]
    fn rank(&self) -> usize {
        self.len()
    }

    #[inline]
    unsafe fn n_elements_unchecked(&self, dimension: usize) -> usize {
        *self.get_unchecked(dimension)
    }
}

unsafe impl<const N: isize> Dims for ArrayDimensions<'_, N> {
    const RANK: isize = N;

    type SlicingType<'s> = &'s [usize] where Self: 's;

    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s> {
        let slice = self.as_slice();
        let len = slice.len();
        let ptr = slice.as_ptr();
        unsafe { std::slice::from_raw_parts(ptr.cast(), len) }
    }

    #[inline]
    fn rank(&self) -> usize {
        if N >= 0 {
            return N as usize;
        }

        self.as_slice().len()
    }

    #[inline]
    unsafe fn n_elements_unchecked(&self, dimension: usize) -> usize {
        if N == 1 || self.rank() == 1 {
            self.as_slice().get_unchecked(dimension).dims_cell.get()
        } else {
            self.as_slice().get_unchecked(dimension).dims_usize
        }
    }
}

unsafe impl Dims for Dimensions {
    const RANK: isize = -1;

    type SlicingType<'s> = &'s Self;

    fn to_slicer<'s>(&'s self) -> Self::SlicingType<'s> {
        self
    }

    #[inline]
    fn rank(&self) -> usize {
        self.as_slice().len()
    }

    #[inline]
    unsafe fn n_elements_unchecked(&self, dimension: usize) -> usize {
        *self.as_slice().get_unchecked(dimension)
    }
}

/// Trait used to convert instances of implementations of `Dims` to a slice.
///
/// The trait method [`Dims::to_slicer`] returns instances of implementations of this trait.
pub trait DimSlice {
    /// Convert `self` to a slice.
    fn as_slice(&self) -> &[usize];
}

impl DimSlice for () {
    fn as_slice(&self) -> &[usize] {
        &[]
    }
}

impl<const N: usize> DimSlice for [usize; N] {
    fn as_slice(&self) -> &[usize] {
        &self[..]
    }
}

impl<const N: usize> DimSlice for &[usize; N] {
    fn as_slice(&self) -> &[usize] {
        &self[..]
    }
}

impl DimSlice for &[usize] {
    fn as_slice(&self) -> &[usize] {
        self
    }
}

impl DimSlice for &Dimensions {
    fn as_slice(&self) -> &[usize] {
        Dimensions::as_slice(self)
    }
}

/// Extension trait for implementations of `Dims` that can be used to construct new arrays.
///
/// This trait is similar to `RankedDims`, but doesn't require that the rank of the dimensions is
/// known at compile-time.
///
/// Safety: only `fill_tuple` must be implemented, and it must be implemented correctly.
pub unsafe trait DimsExt: Dims {
    /// Constructs the array type with element type `T`
    fn array_type<'target, T, Tgt>(&self, target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        T: ConstructType,
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let n = self.rank();
            let elem_ty = T::construct_type(&mut frame);
            unsafe {
                let ty = jl_apply_array_type(elem_ty.unwrap(Private), n);
                target.data_from_ptr(NonNull::new_unchecked(ty), Private)
            }
        })
    }

    /// Fill `tup` with the number of elements of each dimension. All elements of `tup` must be
    /// initialized when this function is called.
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private);

    #[doc(hidden)]
    #[inline]
    unsafe fn alloc_array<'target, Tgt>(
        &self,
        target: Tgt,
        array_type: Value,
    ) -> ArrayData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let array_type = array_type.unwrap(Private);

        let arr = match Self::RANK {
            1 => jl_alloc_array_1d(array_type, self.n_elements_unchecked(0)),
            2 => jl_alloc_array_2d(
                array_type,
                self.n_elements_unchecked(0),
                self.n_elements_unchecked(1),
            ),
            3 => jl_alloc_array_3d(
                array_type,
                self.n_elements_unchecked(0),
                self.n_elements_unchecked(1),
                self.n_elements_unchecked(2),
            ),
            _ => self.alloc_large(array_type, &target),
        };

        Array::wrap_non_null(NonNull::new_unchecked(arr), Private).root(target)
    }

    #[cold]
    #[doc(hidden)]
    #[inline(never)]
    #[julia_version(until = "1.10")]
    unsafe fn alloc_large<'target, Tgt>(
        &self,
        array_type: *mut jl_value_t,
        target: &Tgt,
    ) -> *mut jl_array_t
    where
        Tgt: Target<'target>,
    {
        target.local_scope::<_, 1>(|mut frame| {
            let tuple = unsized_dim_tuple(&frame, self);
            tuple.root(&mut frame);
            jl_new_array(array_type, tuple.ptr().as_ptr())
        })
    }

    #[cold]
    #[doc(hidden)]
    #[inline(never)]
    #[julia_version(since = "1.11")]
    unsafe fn alloc_large<'target, Tgt>(
        &self,
        array_type: *mut jl_value_t,
        _target: &Tgt,
    ) -> *mut jl_array_t
    where
        Tgt: Target<'target>,
    {
        let slicer = self.to_slicer();
        let slice = slicer.as_slice();
        let len = slice.len();
        let ptr = slice.as_ptr();

        jl_alloc_array_nd(array_type, ptr as *mut _, len)
    }

    #[doc(hidden)]
    #[inline]
    unsafe fn alloc_array_with_data<'target, 'data, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        array_type: Value,
        data: *mut c_void,
    ) -> ArrayData<'target, 'data, Tgt> {
        let array_type = array_type.unwrap(Private);

        let arr = match self.rank() {
            1 => jl_ptr_to_array_1d(array_type, data, self.n_elements_unchecked(0), 0),
            _ => target.local_scope::<_, 1>(|frame| {
                let tuple = unsized_dim_tuple(frame, self);
                jl_ptr_to_array(array_type, data, tuple.unwrap(Private), 0)
            }),
        };

        target.data_from_ptr(NonNull::new_unchecked(arr), Private)
    }

    #[doc(hidden)]
    #[inline]
    fn dimension_object<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> DataTypeData<'target, Tgt> {
        let rank = self.rank();

        unsafe {
            let raw = jlrs_dimtuple_type(rank);
            target.data_from_ptr(NonNull::new_unchecked(raw), Private)
        }
    }
}

unsafe impl<D: RankedDims> DimsExt for D {
    #[inline]
    fn array_type<'target, T, Tgt>(&self, target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        T: ConstructType,
        Tgt: Target<'target>,
    {
        <<Self as RankedDims>::ArrayConstructor<T>>::construct_type(target)
    }

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        <Self as RankedDims>::fill_tuple(self, tup, Private)
    }
}

unsafe impl DimsExt for &[usize] {
    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        let n = self.len();
        let ptr = self.as_ptr().cast::<MaybeUninit<usize>>();
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, n);
            tup.copy_from_slice(slice);
        }
    }
}

unsafe impl DimsExt for Dimensions {
    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        let n = self.rank();
        let slice = self.as_slice();
        let ptr = slice.as_ptr().cast::<MaybeUninit<usize>>();
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, n);
            tup.copy_from_slice(slice);
        }
    }
}

/// Extension trait for implementations of `Dims` with a known rank at compile-time.
///
/// Functions that take an implementation of this trait instead of `Dims` require that the rank is
/// known at compile-time.
///
/// Safety: only `ArrayConstructor` and `fill_tuple` must be implemented, no default
/// implementations must be overridden.
pub unsafe trait RankedDims: Dims {
    /// This constant only exists if `Self::RANK != -1`, otherwise using this constant results in
    /// a compile-time error.
    const ASSERT_RANKED: () = assert!(Self::RANK != -1);

    /// The type constructor for an array type with these dimesions.
    type ArrayConstructor<T: ConstructType>: ConstructType;

    /// Fill `tup` with the number of elements of each dimension. All elements of `tup` must be
    /// initialized when this function is called.
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private);

    #[doc(hidden)]
    #[inline]
    unsafe fn alloc_array<'target, Tgt>(
        &self,
        target: Tgt,
        array_type: Value,
    ) -> ArrayData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let _: () = Self::ASSERT_RANKED;
        let array_type = array_type.unwrap(Private);

        let arr = match Self::RANK {
            1 => jl_alloc_array_1d(array_type, self.n_elements_unchecked(0)),
            2 => jl_alloc_array_2d(
                array_type,
                self.n_elements_unchecked(0),
                self.n_elements_unchecked(1),
            ),
            3 => jl_alloc_array_3d(
                array_type,
                self.n_elements_unchecked(0),
                self.n_elements_unchecked(1),
                self.n_elements_unchecked(2),
            ),
            _ => self.alloc_large(array_type, &target),
        };

        Array::wrap_non_null(NonNull::new_unchecked(arr), Private).root(target)
    }

    #[cold]
    #[doc(hidden)]
    #[inline(never)]
    #[julia_version(until = "1.10")]
    unsafe fn alloc_large<'target, Tgt>(
        &self,
        array_type: *mut jl_value_t,
        target: &Tgt,
    ) -> *mut jl_array_t
    where
        Tgt: Target<'target>,
    {
        let _: () = Self::ASSERT_RANKED;
        target.local_scope::<_, 1>(|mut frame| {
            let tuple = sized_dim_tuple(&frame, self);
            tuple.root(&mut frame);
            jl_new_array(array_type, tuple.ptr().as_ptr())
        })
    }

    #[cold]
    #[doc(hidden)]
    #[inline(never)]
    #[julia_version(since = "1.11")]
    unsafe fn alloc_large<'target, Tgt>(
        &self,
        array_type: *mut jl_value_t,
        _target: &Tgt,
    ) -> *mut jl_array_t
    where
        Tgt: Target<'target>,
    {
        let _: () = Self::ASSERT_RANKED;
        let slicer = self.to_slicer();
        let slice = slicer.as_slice();
        let len = slice.len();
        let ptr = slice.as_ptr();

        jl_alloc_array_nd(array_type, ptr as *mut _, len)
    }

    #[doc(hidden)]
    #[inline]
    unsafe fn alloc_array_with_data<'target, 'data, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        array_type: Value,
        data: *mut c_void,
    ) -> ArrayData<'target, 'data, Tgt> {
        let _: () = Self::ASSERT_RANKED;

        let array_type = array_type.unwrap(Private);

        let arr = match Self::RANK {
            1 => jl_ptr_to_array_1d(array_type, data, self.n_elements_unchecked(0), 0),
            _ => target.local_scope::<_, 1>(|frame| {
                let tuple = sized_dim_tuple(frame, self);
                jl_ptr_to_array(array_type, data, tuple.unwrap(Private), 0)
            }),
        };

        target.data_from_ptr(NonNull::new_unchecked(arr), Private)
    }

    #[doc(hidden)]
    #[inline]
    fn dimension_object<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> DataTypeData<'target, Tgt> {
        let _: () = Self::ASSERT_RANKED;
        let rank = Self::RANK;

        unsafe {
            let raw = jlrs_dimtuple_type(rank as _);
            target.data_from_ptr(NonNull::new_unchecked(raw), Private)
        }
    }
}

unsafe impl RankedDims for usize {
    type ArrayConstructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<1>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        tup[0].write(*self);
    }
}

unsafe impl RankedDims for () {
    type ArrayConstructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<0>>;

    #[inline]
    fn fill_tuple(&self, _tup: &mut [MaybeUninit<usize>], _: Private) {}
}

unsafe impl RankedDims for (usize,) {
    type ArrayConstructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<1>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        tup[0].write(self.0);
    }
}

unsafe impl RankedDims for (usize, usize) {
    type ArrayConstructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<2>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        tup[0].write(self.0);
        tup[1].write(self.1);
    }
}

unsafe impl RankedDims for (usize, usize, usize) {
    type ArrayConstructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<3>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        tup[0].write(self.0);
        tup[1].write(self.1);
        tup[2].write(self.2);
    }
}

unsafe impl RankedDims for (usize, usize, usize, usize) {
    type ArrayConstructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<4>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        tup[0].write(self.0);
        tup[1].write(self.1);
        tup[2].write(self.2);
        tup[3].write(self.3);
    }
}

unsafe impl<const N: usize> RankedDims for &[usize; N] {
    type ArrayConstructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantSize<N>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        let data = unsafe { &*(*self as *const [usize; N] as *const [MaybeUninit<usize>; N]) };
        tup.copy_from_slice(data);
    }
}

unsafe impl<const N: usize> RankedDims for [usize; N] {
    type ArrayConstructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantSize<N>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        let data = unsafe { &*(self as *const [usize; N] as *const [MaybeUninit<usize>; N]) };
        tup.copy_from_slice(data);
    }
}

// If the array is one-dimensional, the size of the array can change by pushing or popping
// elements. This is not true for arrays of rank greater than 1 because their size is
// constant.
pub(crate) union Elem {
    pub(crate) dims_cell: ManuallyDrop<Cell<usize>>,
    pub(crate) dims_usize: usize,
}

/// Reference to the dimensions of an [array].
///
/// [array]: crate::data::managed::array::ArrayBase
pub struct ArrayDimensions<'borrow, const N: isize> {
    dims: &'borrow [Elem],
}

impl<const N: isize> Debug for ArrayDimensions<'_, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if N == 1 || self.rank() == 1 {
            let dim = unsafe { self.dims[0].dims_cell.get() };
            let dims = &[dim][..];
            f.debug_struct("ArrayDimensions")
                .field("dims", &dims)
                .finish()
        } else {
            let dims = unsafe { &*(self.dims as *const [Elem] as *const [usize]) };
            f.debug_struct("ArrayDimensions")
                .field("dims", &dims)
                .finish()
        }
    }
}

impl<'borrow, const N: isize> ArrayDimensions<'borrow, N> {
    #[inline]
    pub(crate) fn new(dims: &'borrow [Elem]) -> Self {
        ArrayDimensions { dims }
    }

    #[inline]
    pub(crate) fn as_slice(&self) -> &[Elem] {
        &self.dims
    }
}

/// The dimensions of an n-dimensional array that has been copied from Julia to Rust.
#[derive(Clone)]
pub enum Dimensions {
    #[doc(hidden)]
    Few([usize; 4]),
    #[doc(hidden)]
    Many(Box<[usize]>),
}

impl Dimensions {
    /// Convert an implementation of `Dims` to `Dimensions`.
    pub fn from_dims<D: Dims>(dims: &D) -> Self {
        unsafe {
            match dims.rank() {
                0 => Dimensions::Few([0, 0, 0, 0]),
                1 => Dimensions::Few([1, dims.n_elements_unchecked(0), 0, 0]),
                2 => Dimensions::Few([
                    2,
                    dims.n_elements_unchecked(0),
                    dims.n_elements_unchecked(1),
                    0,
                ]),
                3 => Dimensions::Few([
                    3,
                    dims.n_elements_unchecked(0),
                    dims.n_elements_unchecked(1),
                    dims.n_elements_unchecked(2),
                ]),
                n => {
                    let mut v = Vec::with_capacity(n + 1);
                    v.push(n);
                    let iter = (0..n).map(|dim| dims.n_elements_unchecked(dim));
                    v.extend(iter);

                    Dimensions::Many(v.into_boxed_slice())
                }
            }
        }
    }

    /// Returns the dimensions as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[usize] {
        match self {
            Dimensions::Few(ref v) => &v[1..v[0] as usize + 1],
            Dimensions::Many(ref v) => &v[1..],
        }
    }
}

impl Debug for Dimensions {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut f = f.debug_tuple("Dimensions");

        for d in self.as_slice() {
            f.field(&d);
        }

        f.finish()
    }
}

impl Display for Dimensions {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut f = f.debug_tuple("");

        for d in self.as_slice() {
            f.field(&d);
        }

        f.finish()
    }
}

/// Check if two dimension types are compatible at compile-time.
pub trait CompatibleIndices<A: Dims, B: Dims>: private::CompatibleIndicesPriv {
    /// This constant exists if the rank of `A` or `B` is -1, or if the ranks of `A` and `B` are
    /// equal.
    ///
    /// If the rank of either `A` or `B` is -1, the ranks have to be compared at runtime.
    const ASSERT_COMPATIBLE: () = assert!(
        A::RANK == -1 || B::RANK == -1 || A::RANK == B::RANK,
        "The rank of the dimensions is incompatible with the rank of the array"
    );
}

impl<A: Dims, B: Dims> CompatibleIndices<A, B> for (A, B) {}

/// Assert that a dimension type is compatible with the rank of the array.
pub trait DimsRankCheck<D: Dims, const N: isize> {
    /// This constant exists if the rank of `A` or `B` is -1, or if the ranks of `A` and `B` are
    /// equal.
    ///
    /// If the rank of either `A` or `B` is -1, the ranks have to be compared at runtime.
    const ASSERT_VALID_RANK: () = assert!(
        D::RANK == N || N == -1 || D::RANK == -1,
        "The rank of the dimensions is incompatible with the rank of the array"
    );

    /// This constant is `true` if it must be checked that the rank of `D` is `N` at runtime.
    const NEEDS_RUNTIME_RANK_CHECK: bool = N != -1 && D::RANK == -1;
}

/// Helper struct for using [`DimsRankCheck`].
pub struct DimsRankAssert<D: Dims, const N: isize>(PhantomData<D>);

impl<D: Dims, const N: isize> DimsRankCheck<D, N> for DimsRankAssert<D, N> {}

pub(crate) mod private {
    use super::Dims;

    pub trait CompatibleIndicesPriv {}
    impl<A: Dims, B: Dims> CompatibleIndicesPriv for (A, B) {}
}

#[cfg(test)]
mod tests {
    use crate::data::managed::array::dimensions::{Dimensions, Dims};

    #[test]
    fn convert_usize() {
        let d: Dimensions = 4.to_dimensions();
        assert_eq!(d.rank(), 1);
        assert_eq!(d.n_elements(0), Some(4));
        assert_eq!(d.size(), 4);
    }

    #[test]
    fn convert_tuple_0d() {
        let d: Dimensions = ().to_dimensions();
        assert_eq!(d.rank(), 0);
        assert_eq!(d.n_elements(0), Some(0));
        assert_eq!(d.size(), 1);
    }

    #[test]
    fn convert_tuple_1d() {
        let d: Dimensions = (4,).to_dimensions();
        assert_eq!(d.rank(), 1);
        assert_eq!(d.n_elements(0), Some(4));
        assert_eq!(d.size(), 4);
    }

    #[test]
    fn convert_tuple_2d() {
        let d: Dimensions = (4, 3).to_dimensions();
        assert_eq!(d.rank(), 2);
        assert_eq!(d.n_elements(0), Some(4));
        assert_eq!(d.n_elements(1), Some(3));
        assert_eq!(d.size(), 12);
    }

    #[test]
    fn convert_tuple_3d() {
        let d: Dimensions = (4, 3, 2).to_dimensions();
        assert_eq!(d.rank(), 3);
        assert_eq!(d.n_elements(0), Some(4));
        assert_eq!(d.n_elements(1), Some(3));
        assert_eq!(d.n_elements(2), Some(2));
        assert_eq!(d.size(), 24);
    }

    #[test]
    fn convert_tuple_4d() {
        let d: Dimensions = (4, 3, 2, 1).to_dimensions();
        assert_eq!(d.rank(), 4);
        assert_eq!(d.n_elements(0), Some(4));
        assert_eq!(d.n_elements(1), Some(3));
        assert_eq!(d.n_elements(2), Some(2));
        assert_eq!(d.n_elements(3), Some(1));
        assert_eq!(d.size(), 24);
    }

    #[test]
    fn convert_array_5d() {
        let d: Dimensions = (&[4, 3, 2, 1, 2]).to_dimensions();
        assert_eq!(d.rank(), 5);
        assert_eq!(d.n_elements(0), Some(4));
        assert_eq!(d.n_elements(1), Some(3));
        assert_eq!(d.n_elements(2), Some(2));
        assert_eq!(d.n_elements(3), Some(1));
        assert_eq!(d.n_elements(4), Some(2));
        assert_eq!(d.size(), 48);
    }

    #[test]
    fn convert_array_nd() {
        let v = &[1, 2, 3][..];
        let d: Dimensions = v.to_dimensions();
        assert_eq!(d.rank(), 3);
        assert_eq!(d.n_elements(0), Some(1));
        assert_eq!(d.n_elements(1), Some(2));
        assert_eq!(d.n_elements(2), Some(3));
        assert_eq!(d.size(), 6);
    }
}
