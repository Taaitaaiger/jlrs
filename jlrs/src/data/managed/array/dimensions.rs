//! N-dimensional indexing.
//!
//! In order to access the data of an n-dimensional array, you'll need to use an n-dimensional
//! index. This functionality is provided by the [`Dims`] trait, any implementor of this trait
//! can be used as an n-dimensional index. The most important implementations are tuples (up to
//! and including four dimensions), and arrays and array slices of any number of dimensions. So,
//! if you want to access the third column of the second row of an array, you can use both
//! `[1, 2]` or `(1, 2)`. Note that unlike Julia, array indexing starts at 0.

// TODO: IntoDimensions traiit
// TODO: clean up
use std::{
    ffi::c_void,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
};

use jl_sys::{
    jl_alloc_array_1d, jl_alloc_array_2d, jl_alloc_array_3d, jl_array_dims_ptr, jl_array_ndims,
    jl_array_t, jl_new_array, jl_ptr_to_array, jl_ptr_to_array_1d, jl_value_t, jlrs_dimtuple_type,
};

use self::private::DimsPriv;
use super::ArrayData;
use crate::{
    data::{
        managed::{array::Array, datatype::DataTypeData, private::ManagedPriv as _},
        types::construct_type::{ArrayTypeConstructor, ConstantIsize, ConstantSize, ConstructType},
    },
    error::{AccessError, JlrsResult},
    prelude::{DataType, Managed, NTuple, Target, Tuple0, Tuple1, Tuple2, Tuple3, Tuple4, Value},
    private::Private,
};

pub trait Dims: Sized + Debug {
    /// Returns the rank if this index.
    fn rank(&self) -> usize;

    /// Returns the number of elements of the nth dimension. Indexing starts at 0.
    fn n_elements(&self, dimension: usize) -> usize;

    /// The total number of elements in the arry, i.e. the product of the number of elements of
    /// each dimension.
    #[inline]
    fn size(&self) -> usize {
        (0..self.rank()).map(|i| self.n_elements(i)).product()
    }

    /// Calculate the linear index for `dim_index` in an array with dimensions `self`.
    ///
    /// The default implementation must not be overridden.
    fn index_of<D: Dims>(&self, dim_index: &D) -> JlrsResult<usize> {
        if self.rank() != dim_index.rank() {
            Err(AccessError::InvalidIndex {
                idx: dim_index.into_dimensions(),
                sz: self.into_dimensions(),
            })?;
        }

        let n_dims = self.rank();
        if self.rank() == 0 {
            return Ok(0);
        }

        for dim in 0..n_dims {
            if self.n_elements(dim) <= dim_index.n_elements(dim) {
                Err(AccessError::InvalidIndex {
                    idx: dim_index.into_dimensions(),
                    sz: self.into_dimensions(),
                })?;
            }
        }

        let init = dim_index.n_elements(n_dims - 1);
        let idx = (0..n_dims - 1).rev().fold(init, |idx_acc, dim| {
            idx_acc * self.n_elements(dim) + dim_index.n_elements(dim)
        });

        Ok(idx)
    }

    /// Convert `Self` to `Dimensions`.
    ///
    /// The default implementation must not be overridden.
    fn into_dimensions(&self) -> Dimensions {
        Dimensions::from_dims(self)
    }
}

/// Trait implemented by types that can be used as n-dimensional indices.
pub trait DimsExt: DimsPriv + Dims {
    /// The rank of array that can use this type as an index.
    ///
    /// This constant is -1 if the rank is not known at compile-time.
    const RANK: isize;

    type DimTupleConstructor: ConstructType;

    /// The type constructor for an array type with these dimesions.
    ///
    /// This constructor may only be used if `Self::Rank` is not equal to `-1`.
    type ArrayContructor<T: ConstructType>: ConstructType;

    #[doc(hidden)]
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

        let arr = match self.rank() {
            1 => jl_alloc_array_1d(array_type, self.n_elements(0)),
            2 => jl_alloc_array_2d(array_type, self.n_elements(0), self.n_elements(1)),
            3 => jl_alloc_array_3d(
                array_type,
                self.n_elements(0),
                self.n_elements(1),
                self.n_elements(2),
            ),
            _ => self.alloc_large(array_type, &target),
        };

        Array::wrap_non_null(NonNull::new_unchecked(arr), Private).root(target)
    }

    #[cold]
    #[doc(hidden)]
    #[inline(never)]
    unsafe fn alloc_large<'target, Tgt>(
        &self,
        array_type: *mut jl_value_t,
        target: &Tgt,
    ) -> *mut jl_array_t
    where
        Tgt: Target<'target>,
    {
        target
            .local_scope::<_, _, 1>(|mut frame| {
                let tuple = super::sized_dim_tuple(&frame, self);
                tuple.root(&mut frame);
                Ok(jl_new_array(array_type, tuple.ptr().as_ptr()))
            })
            .unwrap_unchecked()
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
            1 => jl_ptr_to_array_1d(array_type, data, self.n_elements(0), 0),
            _ => target
                .local_scope::<_, _, 1>(|frame| {
                    let tuple = super::sized_dim_tuple(frame, self);
                    Ok(jl_ptr_to_array(array_type, data, tuple.unwrap(Private), 0))
                })
                .unwrap_unchecked(),
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
            let raw = jlrs_dimtuple_type(rank as _);
            target.data_from_ptr(NonNull::new_unchecked(raw), Private)
        }
    }
}

/// Dimensions of a Julia array.
#[derive(Copy, Clone, Debug)]
pub struct ArrayDimensions<'scope> {
    n: usize,
    ptr: *mut usize,
    _marker: PhantomData<&'scope [usize]>,
}

impl<'scope> ArrayDimensions<'scope> {
    #[inline]
    pub(crate) fn new(array: Array<'scope, '_>) -> Self {
        let array_ptr = array.unwrap(Private);
        // Safety: The array's dimensions exists as long as the array does.
        unsafe {
            let ptr = jl_array_dims_ptr(array_ptr);
            let n = jl_array_ndims(array_ptr) as usize;

            ArrayDimensions {
                ptr,
                n,
                _marker: PhantomData,
            }
        }
    }

    /// Returns the dimensions as a slice.
    ///
    /// Safety: don't push new elements to a 1-dimensional array while borrowing its dimensions
    /// as a slice.
    pub unsafe fn as_slice<'borrow>(&'borrow self) -> &'borrow [usize] {
        std::slice::from_raw_parts(self.ptr, self.n)
    }
}

impl<'scope> Dims for ArrayDimensions<'scope> {
    #[inline]
    fn rank(&self) -> usize {
        self.n
    }

    #[inline]
    fn n_elements(&self, dimension: usize) -> usize {
        if dimension >= self.n {
            return 0;
        }

        // Safety: the dimension is in bounds
        unsafe { self.ptr.add(dimension).read() }
    }
}

impl DimsExt for () {
    const RANK: isize = 0;

    type DimTupleConstructor = Tuple0;

    type ArrayContructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<0>>;

    #[inline]
    fn fill_tuple(&self, _tup: &mut [MaybeUninit<usize>], _: Private) {}

    #[inline]
    unsafe fn alloc_array<'target, Tgt>(
        &self,
        target: Tgt,
        array_type: Value,
    ) -> ArrayData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            target
                .with_local_scope::<_, _, 1>(|target, mut frame| {
                    let array_type = array_type.unwrap(Private);
                    let tuple = super::sized_dim_tuple(&target, self);
                    tuple.root(&mut frame);
                    let arr = jl_new_array(array_type, tuple.ptr().as_ptr());
                    Ok(Array::wrap_non_null(NonNull::new_unchecked(arr), Private).root(target))
                })
                .unwrap()
        }
    }

    #[inline]
    unsafe fn alloc_array_with_data<'target, 'data, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        array_type: Value,
        data: *mut c_void,
    ) -> ArrayData<'target, 'data, Tgt> {
        let array_type = array_type.unwrap(Private);
        let tuple = Value::emptytuple(&target);
        let arr = jl_ptr_to_array(array_type, data, tuple.unwrap(Private), 0);
        target.data_from_ptr(NonNull::new_unchecked(arr), Private)
    }

    #[inline]
    fn dimension_object<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> DataTypeData<'target, Tgt> {
        DataType::emptytuple_type(&target).root(target)
    }
}

impl Dims for () {
    #[inline]
    fn rank(&self) -> usize {
        0
    }

    #[inline]
    fn n_elements(&self, _: usize) -> usize {
        0
    }
}

impl DimsExt for usize {
    const RANK: isize = 1;

    type DimTupleConstructor = Tuple1<usize>;

    type ArrayContructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<1>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        tup[0].write(*self);
    }

    #[inline]
    unsafe fn alloc_array<'target, Tgt>(
        &self,
        target: Tgt,
        array_type: Value,
    ) -> ArrayData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let array_type = array_type.unwrap(Private);
            let arr = jl_alloc_array_1d(array_type, *self);
            Array::wrap_non_null(NonNull::new_unchecked(arr), Private).root(target)
        }
    }

    #[inline]
    unsafe fn alloc_array_with_data<'target, 'data, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        array_type: Value,
        data: *mut c_void,
    ) -> ArrayData<'target, 'data, Tgt> {
        let array_type = array_type.unwrap(Private);
        let arr = jl_ptr_to_array_1d(array_type, data, *self, 0);
        target.data_from_ptr(NonNull::new_unchecked(arr), Private)
    }

    #[inline]
    fn dimension_object<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> DataTypeData<'target, Tgt> {
        unsafe {
            NTuple::<isize, 1>::construct_type(&target)
                .as_managed()
                .cast_unchecked::<DataType>()
                .root(target)
        }
    }
}

impl Dims for usize {
    #[inline]
    fn rank(&self) -> usize {
        1
    }

    #[inline]
    fn n_elements(&self, dimension: usize) -> usize {
        if dimension == 0 {
            *self
        } else {
            0
        }
    }
}

impl DimsExt for (usize,) {
    const RANK: isize = 1;

    type DimTupleConstructor = Tuple1<usize>;

    type ArrayContructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<1>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        tup[0].write(self.0);
    }

    #[inline]
    unsafe fn alloc_array<'target, Tgt>(
        &self,
        target: Tgt,
        array_type: Value,
    ) -> ArrayData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let array_type = array_type.unwrap(Private);
            let arr = jl_alloc_array_1d(array_type, self.0);
            Array::wrap_non_null(NonNull::new_unchecked(arr), Private).root(target)
        }
    }

    #[inline]
    unsafe fn alloc_array_with_data<'target, 'data, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        array_type: Value,
        data: *mut c_void,
    ) -> ArrayData<'target, 'data, Tgt> {
        let array_type = array_type.unwrap(Private);
        let arr = jl_ptr_to_array_1d(array_type, data, self.0, 0);
        target.data_from_ptr(NonNull::new_unchecked(arr), Private)
    }

    #[inline]
    fn dimension_object<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> DataTypeData<'target, Tgt> {
        unsafe {
            NTuple::<isize, 1>::construct_type(&target)
                .as_managed()
                .cast_unchecked::<DataType>()
                .root(target)
        }
    }
}

impl Dims for (usize,) {
    #[inline]
    fn rank(&self) -> usize {
        1
    }

    #[inline]
    fn n_elements(&self, dimension: usize) -> usize {
        if dimension == 0 {
            self.0
        } else {
            0
        }
    }
}

impl DimsExt for (usize, usize) {
    const RANK: isize = 2;

    type DimTupleConstructor = Tuple2<usize, usize>;

    type ArrayContructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<2>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        tup[0].write(self.0);
        tup[1].write(self.1);
    }

    #[inline]
    unsafe fn alloc_array<'target, Tgt>(
        &self,
        target: Tgt,
        array_type: Value,
    ) -> ArrayData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let array_type = array_type.unwrap(Private);
            let arr = jl_alloc_array_2d(array_type, self.0, self.1);
            Array::wrap_non_null(NonNull::new_unchecked(arr), Private).root(target)
        }
    }

    #[inline]
    unsafe fn alloc_array_with_data<'target, 'data, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        array_type: Value,
        data: *mut c_void,
    ) -> ArrayData<'target, 'data, Tgt> {
        target
            .with_local_scope::<_, _, 1>(|target, frame| {
                let array_type = array_type.unwrap(Private);
                let tuple = super::sized_dim_tuple(frame, self);
                let arr = jl_ptr_to_array(array_type, data, tuple.unwrap(Private), 0);

                Ok(target.data_from_ptr(NonNull::new_unchecked(arr), Private))
            })
            .unwrap_unchecked()
    }

    #[inline]
    fn dimension_object<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> DataTypeData<'target, Tgt> {
        unsafe {
            NTuple::<isize, 2>::construct_type(&target)
                .as_managed()
                .cast_unchecked::<DataType>()
                .root(target)
        }
    }
}

impl Dims for (usize, usize) {
    #[inline]
    fn rank(&self) -> usize {
        2
    }

    #[inline]
    fn n_elements(&self, dimension: usize) -> usize {
        match dimension {
            0 => self.0,
            1 => self.1,
            _ => 0,
        }
    }
}

impl DimsExt for (usize, usize, usize) {
    const RANK: isize = 3;

    type DimTupleConstructor = Tuple3<usize, usize, usize>;

    type ArrayContructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<3>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        tup[0].write(self.0);
        tup[1].write(self.1);
        tup[2].write(self.2);
    }

    #[inline]
    unsafe fn alloc_array<'target, Tgt>(
        &self,
        target: Tgt,
        array_type: Value,
    ) -> ArrayData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let array_type = array_type.unwrap(Private);
            let arr = jl_alloc_array_3d(array_type, self.0, self.1, self.2);
            Array::wrap_non_null(NonNull::new_unchecked(arr), Private).root(target)
        }
    }

    #[inline]
    unsafe fn alloc_array_with_data<'target, 'data, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        array_type: Value,
        data: *mut c_void,
    ) -> ArrayData<'target, 'data, Tgt> {
        target
            .with_local_scope::<_, _, 1>(|target, frame| {
                let array_type = array_type.unwrap(Private);
                let tuple = super::sized_dim_tuple(frame, self);
                let arr = jl_ptr_to_array(array_type, data, tuple.unwrap(Private), 0);

                Ok(target.data_from_ptr(NonNull::new_unchecked(arr), Private))
            })
            .unwrap_unchecked()
    }

    #[inline]
    fn dimension_object<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> DataTypeData<'target, Tgt> {
        unsafe {
            NTuple::<isize, 3>::construct_type(&target)
                .as_managed()
                .cast_unchecked::<DataType>()
                .root(target)
        }
    }
}

impl Dims for (usize, usize, usize) {
    #[inline]
    fn rank(&self) -> usize {
        3
    }

    #[inline]
    fn n_elements(&self, dimension: usize) -> usize {
        match dimension {
            0 => self.0,
            1 => self.1,
            2 => self.2,
            _ => 0,
        }
    }
}

impl DimsExt for (usize, usize, usize, usize) {
    const RANK: isize = 4;

    type DimTupleConstructor = Tuple4<usize, usize, usize, usize>;

    type ArrayContructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantIsize<4>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        tup[0].write(self.0);
        tup[1].write(self.1);
        tup[2].write(self.2);
        tup[3].write(self.3);
    }

    #[inline]
    unsafe fn alloc_array<'target, Tgt>(
        &self,
        target: Tgt,
        array_type: Value,
    ) -> ArrayData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let ptr = self.alloc_large(array_type.unwrap(Private), &target);
        target.data_from_ptr(NonNull::new_unchecked(ptr), Private)
    }

    #[inline]
    unsafe fn alloc_array_with_data<'target, 'data, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        array_type: Value,
        data: *mut c_void,
    ) -> ArrayData<'target, 'data, Tgt> {
        target
            .with_local_scope::<_, _, 1>(|target, frame| {
                let array_type = array_type.unwrap(Private);
                let tuple = super::sized_dim_tuple(frame, self);
                let arr = jl_ptr_to_array(array_type, data, tuple.unwrap(Private), 0);

                Ok(target.data_from_ptr(NonNull::new_unchecked(arr), Private))
            })
            .unwrap_unchecked()
    }

    #[inline]
    fn dimension_object<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> DataTypeData<'target, Tgt> {
        unsafe {
            NTuple::<isize, 4>::construct_type(&target)
                .as_managed()
                .cast_unchecked::<DataType>()
                .root(target)
        }
    }
}

impl Dims for (usize, usize, usize, usize) {
    #[inline]
    fn rank(&self) -> usize {
        4
    }

    #[inline]
    fn n_elements(&self, dimension: usize) -> usize {
        match dimension {
            0 => self.0,
            1 => self.1,
            2 => self.2,
            3 => self.3,
            _ => 0,
        }
    }
}

impl<const N: usize> DimsExt for &[usize; N] {
    const RANK: isize = N as isize;

    type DimTupleConstructor = NTuple<usize, N>;

    type ArrayContructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantSize<N>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        for i in 0..N {
            tup[i].write(self[i]);
        }
    }

    #[inline]
    fn dimension_object<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> DataTypeData<'target, Tgt> {
        unsafe {
            NTuple::<isize, N>::construct_type(&target)
                .as_managed()
                .cast_unchecked::<DataType>()
                .root(target)
        }
    }
}

impl<const N: usize> Dims for &[usize; N] {
    #[inline]
    fn rank(&self) -> usize {
        N
    }

    #[inline]
    fn n_elements(&self, dim: usize) -> usize {
        if dim < N {
            self[dim]
        } else {
            0
        }
    }
}

impl<const N: usize> DimsExt for [usize; N] {
    const RANK: isize = N as isize;

    type DimTupleConstructor = NTuple<usize, N>;

    type ArrayContructor<T: ConstructType> = ArrayTypeConstructor<T, ConstantSize<N>>;

    #[inline]
    fn fill_tuple(&self, tup: &mut [MaybeUninit<usize>], _: Private) {
        for i in 0..N {
            tup[i].write(self[i]);
        }
    }

    #[inline]
    fn dimension_object<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> DataTypeData<'target, Tgt> {
        unsafe {
            NTuple::<isize, N>::construct_type(&target)
                .as_managed()
                .cast_unchecked::<DataType>()
                .root(target)
        }
    }
}

impl<const N: usize> Dims for [usize; N] {
    #[inline]
    fn rank(&self) -> usize {
        N
    }

    #[inline]
    fn n_elements(&self, dim: usize) -> usize {
        if dim < N {
            self[dim]
        } else {
            0
        }
    }
}

impl Dims for &[usize] {
    #[inline]
    fn rank(&self) -> usize {
        self.len()
    }

    #[inline]
    fn n_elements(&self, dim: usize) -> usize {
        if dim < self.len() {
            self[dim]
        } else {
            0
        }
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
        match dims.rank() {
            0 => Dimensions::Few([0, 0, 0, 0]),
            1 => Dimensions::Few([1, dims.n_elements(0), 0, 0]),
            2 => Dimensions::Few([2, dims.n_elements(0), dims.n_elements(1), 0]),
            3 => Dimensions::Few([
                3,
                dims.n_elements(0),
                dims.n_elements(1),
                dims.n_elements(2),
            ]),
            n => {
                let mut v = Vec::with_capacity(n + 1);
                v.push(n);
                for dim in 0..n {
                    v.push(dims.n_elements(dim));
                }

                Dimensions::Many(v.into_boxed_slice())
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

impl Dims for Dimensions {
    #[inline]
    fn rank(&self) -> usize {
        match self {
            Dimensions::Few([n, _, _, _]) => *n,
            Dimensions::Many(ref dims) => dims[0],
        }
    }

    #[inline]
    fn n_elements(&self, dim: usize) -> usize {
        if dim < self.rank() {
            match self {
                Dimensions::Few(dims) => dims[dim + 1],
                Dimensions::Many(ref dims) => dims[dim + 1],
            }
        } else {
            0
        }
    }

    #[inline]
    fn size(&self) -> usize {
        if self.rank() == 0 {
            return 0;
        }

        let dims = match self {
            Dimensions::Few(ref dims) => &dims[1..dims[0] as usize + 1],
            Dimensions::Many(ref dims) => &dims[1..dims[0] as usize + 1],
        };
        dims.iter().product()
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

pub(crate) mod private {
    pub trait DimsPriv {}

    impl DimsPriv for () {}

    impl DimsPriv for usize {}

    impl DimsPriv for (usize,) {}

    impl DimsPriv for (usize, usize) {}

    impl DimsPriv for (usize, usize, usize) {}

    impl DimsPriv for (usize, usize, usize, usize) {}

    impl<const N: usize> DimsPriv for [usize; N] {}

    impl<const N: usize> DimsPriv for &[usize; N] {}
}

#[cfg(test)]
mod tests {
    use super::{Dimensions, Dims};
    #[test]
    fn convert_usize() {
        let d: Dimensions = 4.into_dimensions();
        assert_eq!(d.rank(), 1);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.size(), 4);
    }

    #[test]
    fn convert_tuple_0d() {
        let d: Dimensions = ().into_dimensions();
        assert_eq!(d.rank(), 0);
        assert_eq!(d.n_elements(0), 0);
        assert_eq!(d.size(), 0);
    }

    #[test]
    fn convert_tuple_1d() {
        let d: Dimensions = (4,).into_dimensions();
        assert_eq!(d.rank(), 1);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.size(), 4);
    }

    #[test]
    fn convert_tuple_2d() {
        let d: Dimensions = (4, 3).into_dimensions();
        assert_eq!(d.rank(), 2);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.size(), 12);
    }

    #[test]
    fn convert_tuple_3d() {
        let d: Dimensions = (4, 3, 2).into_dimensions();
        assert_eq!(d.rank(), 3);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.n_elements(2), 2);
        assert_eq!(d.size(), 24);
    }

    #[test]
    fn convert_tuple_4d() {
        let d: Dimensions = (4, 3, 2, 1).into_dimensions();
        assert_eq!(d.rank(), 4);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.n_elements(2), 2);
        assert_eq!(d.n_elements(3), 1);
        assert_eq!(d.size(), 24);
    }

    #[test]
    fn convert_tuple_5d() {
        let d: Dimensions = (&[4, 3, 2, 1, 2]).into_dimensions();
        assert_eq!(d.rank(), 5);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.n_elements(2), 2);
        assert_eq!(d.n_elements(3), 1);
        assert_eq!(d.n_elements(4), 2);
        assert_eq!(d.size(), 48);
    }

    #[test]
    fn convert_tuple_nd() {
        let v = [1, 2, 3];
        let d: Dimensions = v.into_dimensions();
        assert_eq!(d.rank(), 3);
        assert_eq!(d.n_elements(0), 1);
        assert_eq!(d.n_elements(1), 2);
        assert_eq!(d.n_elements(2), 3);
        assert_eq!(d.size(), 6);
    }
}
