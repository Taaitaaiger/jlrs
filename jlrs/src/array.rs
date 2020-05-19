//! Support for n-dimensional arrays and their dimensions.
//!
//! When working with arrays, it's important to be aware of the `isbits` optimization in Julia.
//! Simple types in Julia, eg `UInt64` and `Float32`, are stored in terms of their raw bits. This
//! extends to structs and tuples containing combinations of those types, their data is stored
//! inline (and compatible with a struct in C containing those types).
//!
//! Since Julia 1.4 this optimization has been extended to arrays: the raw data itself is stored
//! inline inside the array's backing storage in a C-compatible way. In order to support that
//! optimization, many types that derive either `JuliaTuple` or `JuliaStruct` can also derive
//! `ArrayDatatype`.

use crate::datatype::JuliaType;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Frame;
use crate::value::Value;
use jl_sys::{
    jl_array_dim, jl_array_dims, jl_array_eltype, jl_array_ndims, jl_array_nrows, jl_array_t,
    jl_gc_wb, jl_typeof, jl_array_data
};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Array<'frame, 'data>(
    *mut jl_array_t,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
);

impl<'frame, 'data> Array<'frame, 'data> {
    pub(crate) unsafe fn wrap(array: *mut jl_array_t) -> Self {
        Array(array, PhantomData, PhantomData)
    }

    pub(crate) unsafe fn ptr(self) -> *mut jl_array_t {
        self.0
    }

    pub fn dimensions(self) -> Dimensions {
        unsafe {
            let ptr = self.ptr();
            match jl_array_ndims(ptr) {
                0 => Into::into(()),
                1 => Into::into(jl_array_nrows(ptr) as usize),
                2 => Into::into((jl_array_dim(ptr, 0), jl_array_dim(ptr, 1))),
                3 => Into::into((
                    jl_array_dim(ptr, 0),
                    jl_array_dim(ptr, 1),
                    jl_array_dim(ptr, 2),
                )),
                ndims => Into::into(jl_array_dims(ptr, ndims as _)),
            }
        }
    }

    pub fn contains<T: JuliaType>(self) -> bool {
        unsafe { jl_array_eltype(self.ptr().cast()).cast() == T::julia_type() }
    }

    pub fn contains_inline<T: JuliaType>(self) -> bool {
        self.contains::<T>() && !self.is_inline_array()
    }

    pub fn is_inline_array(self) -> bool {
        unsafe { (&*self.ptr()).flags.ptrarray() == 0 }
    }

    pub fn has_inlined_pointers(self) -> bool {
        unsafe {
            let flags = (&*self.ptr()).flags;
            self.is_inline_array() && flags.hasptr() != 0
        }
    }

    pub fn is_value_array(self) -> bool {
        !self.is_inline_array()
    }

    //pub fn try_unbox_array<T>()

    pub fn inline_array_data<'borrow, 'fr, T, F>(
        self,
        frame: &'borrow F
    ) -> JlrsResult<ArrayData<'borrow, 'fr, T, F>>
    where
        T: JuliaType,
        F: Frame<'fr>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType)?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe {
            let jl_data = jl_array_data(self.ptr().cast()).cast();
            let dimensions= Dimensions::from_array(self.ptr().cast());
            let data = std::slice::from_raw_parts(jl_data, dimensions.size());
            Ok(ArrayData::new(data, dimensions, frame))
        }
    }

    pub fn inline_array_data_mut<'borrow, 'fr, T, F>(
        self,
        frame: &'borrow mut F
    ) -> JlrsResult<InlineArrayDataMut<'borrow, 'fr, T, F>>
    where
        T: JuliaType,
        F: Frame<'fr>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType)?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe {
            let jl_data = jl_array_data(self.ptr().cast()).cast();
            let dimensions= Dimensions::from_array(self.ptr().cast());
            let data = std::slice::from_raw_parts_mut(jl_data, dimensions.size());
            Ok(InlineArrayDataMut::new(data, dimensions, frame))
        }
    }

    pub fn value_array_data<'borrow, 'fr, F>(
        self,
        frame: &'borrow F
    ) -> JlrsResult<ArrayData<'borrow, 'fr, Value<'frame, 'data>, F>>
    where
        F: Frame<'fr>,
    {
        if !self.is_value_array() {
            Err(JlrsError::WrongType)?;
        }

        unsafe {
            let jl_data = jl_array_data(self.ptr().cast()).cast();
            let dimensions= Dimensions::from_array(self.ptr().cast());
            let data = std::slice::from_raw_parts(jl_data, dimensions.size());
            Ok(ArrayData::new(data, dimensions, frame))
        }
    }

    pub fn value_array_data_mut<'borrow, 'fr, F>(
        self,
        frame: &'borrow mut F
    ) -> JlrsResult<ValueArrayDataMut<'borrow, 'frame, 'data, 'fr, F>>
    where
        F: Frame<'fr>,
    {
        if !self.is_value_array() {
            Err(JlrsError::WrongType)?;
        }

        unsafe {
            let jl_data = jl_array_data(self.ptr().cast()).cast();
            let dimensions= Dimensions::from_array(self.ptr().cast());
            let data = std::slice::from_raw_parts_mut(jl_data, dimensions.size());
            Ok(ValueArrayDataMut::new(self, data, dimensions, frame))
        }
    }
}

/// An n-dimensional array whose contents have been copied from Julia to Rust. You can create this
/// struct by calling [`Value::try_unbox`]. In order to unbox arrays that contain `bool`s or
/// `char`s, you must unbox them as `CopiedArray<i8>` and `CopiedArray<u32>` respectively because these arrays
/// containt uninitialized values. The data has a column-major order and can be indexed with
/// anything that implements `Into<Dimensions>`; see [`Dimensions`] for more information.
///
/// [`Value::try_unbox`]: ../value/struct.Value.html#method.try_unbox
/// [`Dimensions`]: struct.Dimensions.html
pub struct CopiedArray<T> {
    data: Vec<T>,
    dimensions: Dimensions,
}

impl<T> CopiedArray<T> {
    pub(crate) fn new(data: Vec<T>, dimensions: Dimensions) -> Self {
        CopiedArray { data, dimensions }
    }

    /// Turn the array into a tuple containing its data in column-major order and its dimensions.
    pub fn splat(self) -> (Vec<T>, Dimensions) {
        (self.data, self.dimensions)
    }

    /// Returns a reference to the element at the given n-dimensional index if the index is valid,
    /// `None` otherwise.
    pub fn get<D: Into<Dimensions>>(&self, idx: D) -> Option<&T> {
        Some(&self.data[self.dimensions.index_of(idx).ok()?])
    }

    /// Returns a mutable reference to the element at the given n-dimensional index if the index
    /// is valid, `None` otherwise.
    pub fn get_mut<D: Into<Dimensions>>(&mut self, idx: D) -> Option<&mut T> {
        Some(&mut self.data[self.dimensions.index_of(idx).ok()?])
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns the array's data as a mutable slice, the data is in column-major order.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Returns a reference to the array's dimensions.
    pub fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }
}

impl<T, D: Into<Dimensions>> Index<D> for CopiedArray<T> {
    type Output = T;
    fn index(&self, idx: D) -> &T {
        &self.data[self.dimensions.index_of(idx).unwrap()]
    }
}

impl<T, D: Into<Dimensions>> IndexMut<D> for CopiedArray<T> {
    fn index_mut(&mut self, idx: D) -> &mut T {
        &mut self.data[self.dimensions.index_of(idx).unwrap()]
    }
}

/// Immutably borrowed array data from Julia. The data has a column-major order and can be indexed
/// with anything that implements `Into<Dimensions>`; see [`Dimensions`] for more information.
///
/// [`Dimensions`]: struct.Dimensions.html
pub struct ArrayData<'borrow, 'frame, T, F: Frame<'frame>> {
    data: &'borrow [T],
    dimensions: Dimensions,
    _notsendsync: PhantomData<*const ()>,
    _frame: PhantomData<&'borrow &'frame F>,
}

impl<'borrow, 'frame, T, F> ArrayData<'borrow, 'frame, T, F>
where
    F: Frame<'frame>,
{
    pub(crate) unsafe fn new(data: &'borrow [T], dimensions: Dimensions, _: &'borrow F) -> Self {
        ArrayData {
            data,
            dimensions,
            _notsendsync: PhantomData,
            _frame: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D: Into<Dimensions>>(&self, index: D) -> Option<&T> {
        Some(&self.data[self.dimensions.index_of(index).ok()?])
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns a reference to the array's dimensions.
    pub fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }
}

impl<'borrow, 'frame, T, D, F> Index<D> for ArrayData<'borrow, 'frame, T, F>
where
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        &self.data[self.dimensions.index_of(index).unwrap()]
    }
}

/// Mutably borrowed array data from Julia. The data has a column-major order and can be indexed
/// with anything that implements `Into<Dimensions>`; see [`Dimensions`] for more information.
///
/// [`Dimensions`]: struct.Dimensions.html
pub struct InlineArrayDataMut<'borrow, 'frame, T, F: Frame<'frame>> {
    data: &'borrow mut [T],
    dimensions: Dimensions,
    _notsendsync: PhantomData<*const ()>,
    _frame: PhantomData<&'borrow &'frame mut F>,
}

impl<'borrow, 'frame, T, F> InlineArrayDataMut<'borrow, 'frame, T, F>
where
    F: Frame<'frame>,
{
    pub(crate) unsafe fn new(
        data: &'borrow mut [T],
        dimensions: Dimensions,
        _: &'borrow mut F,
    ) -> Self {
        InlineArrayDataMut {
            data,
            dimensions,
            _notsendsync: PhantomData,
            _frame: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D: Into<Dimensions>>(&self, index: D) -> Option<&T> {
        Some(&self.data[self.dimensions.index_of(index).ok()?])
    }

    /// Get a mutable reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get_mut<D: Into<Dimensions>>(&mut self, index: D) -> Option<&mut T> {
        Some(&mut self.data[self.dimensions.index_of(index).ok()?])
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns the array's data as a mutable slice, the data is in column-major order.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Returns a reference to the array's dimensions.
    pub fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }
}

impl<'borrow, 'frame, T, D, F> Index<D> for InlineArrayDataMut<'borrow, 'frame, T, F>
where
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        &self.data[self.dimensions.index_of(index).unwrap()]
    }
}

impl<'borrow, 'frame, T, D, F> IndexMut<D> for InlineArrayDataMut<'borrow, 'frame, T, F>
where
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    fn index_mut(&mut self, index: D) -> &mut Self::Output {
        &mut self.data[self.dimensions.index_of(index).unwrap()]
    }
}

pub struct ValueArrayDataMut<'borrow, 'value, 'data, 'frame, F: Frame<'frame>> {
    array: Array<'value, 'data>,
    data: &'borrow mut [Value<'value, 'data>],
    dimensions: Dimensions,
    _notsendsync: PhantomData<*const ()>,
    _frame: PhantomData<&'borrow &'frame mut F>,
}

impl<'borrow, 'frame, 'data, 'fr, F> ValueArrayDataMut<'borrow, 'frame, 'data, 'fr, F>
where
    F: Frame<'fr>,
{
    pub(crate) unsafe fn new(
        array: Array<'frame, 'data>,
        data: &'borrow mut [Value<'frame, 'data>],
        dimensions: Dimensions,
        _: &'borrow mut F,
    ) -> Self {
        ValueArrayDataMut {
            array,
            data,
            dimensions,
            _notsendsync: PhantomData,
            _frame: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D: Into<Dimensions>>(&self, index: D) -> Option<&Value<'frame, 'data>> {
        Some(&self.data[self.dimensions.index_of(index).ok()?])
    }

    pub unsafe fn set<'va, 'da: 'data, D: Into<Dimensions>>(
        &mut self,
        index: D,
        value: Value<'frame, 'da>,
    ) -> JlrsResult<()> {
        let ptr = self.array.ptr();
        let eltype = jl_array_eltype(ptr.cast());

        if eltype != jl_typeof(value.ptr().cast()).cast() {
            Err(JlrsError::InvalidArrayType)?;
        }

        self.data[self.dimensions.index_of(index)?] = value;
        jl_gc_wb(self.array.ptr().cast(), value.ptr().cast());

        Ok(())
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[Value<'frame, 'data>] {
        &self.data
    }

    /// Returns a reference to the array's dimensions.
    pub fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }
}

impl<'borrow, 'value, 'data, 'frame, D, F> Index<D>
    for ValueArrayDataMut<'borrow, 'value, 'data, 'frame, F>
where
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    type Output = Value<'value, 'data>;
    fn index(&self, index: D) -> &Self::Output {
        &self.data[self.dimensions.index_of(index).unwrap()]
    }
}

/// The dimensions of an n-dimensional array, they represent either the shape of an array or an
/// index. Functions that need `Dimensions` as an input, which is currently limited to just
/// indexing this data, are generic and accept any type that implements `Into<Dimensions>`.
///
/// For a single dimension, you can use a `usize` value. For 0 up to and including 8 dimensions,
/// you can use tuples of `usize`. In general, you can use slices of `usize`:
///
/// ```
/// # use jlrs::array::Dimensions;
/// # fn main() {
/// let _0d: Dimensions = ().into();
/// let _1d_value: Dimensions = 42.into();
/// let _1d_tuple: Dimensions = (42,).into();
/// let _2d: Dimensions = (42, 6).into();
/// let _nd: Dimensions = [42, 6, 12, 3].as_ref().into();
/// # }
/// ```
#[derive(Clone)]
pub enum Dimensions {
    #[doc(hidden)]
    Few([usize; 4]),
    #[doc(hidden)]
    Many(Box<[usize]>),
}

impl Dimensions {
    pub(crate) unsafe fn from_array(array: *mut jl_array_t) -> Self{
        let n_dims = jl_array_ndims(array);
        match n_dims {
            0 => Into::into(()),
            1 => Into::into(jl_array_nrows(array) as usize),
            2 => Into::into((jl_array_dim(array, 0), jl_array_dim(array, 1))),
            3 => Into::into((
                jl_array_dim(array, 0),
                jl_array_dim(array, 1),
                jl_array_dim(array, 2),
            )),
            ndims => Into::into(jl_array_dims(array, ndims as _)),
        }
    }

    /// Returns the number of dimensions.
    pub fn n_dimensions(&self) -> usize {
        match self {
            Dimensions::Few([n, _, _, _]) => *n,
            Dimensions::Many(ref dims) => dims[0],
        }
    }

    /// Returns the number of elements of the nth dimension. Indexing starts at 0.
    pub fn n_elements(&self, dimension: usize) -> usize {
        if self.n_dimensions() == 0 && dimension == 0 {
            return 0;
        }

        assert!(dimension < self.n_dimensions());

        let dims = match self {
            Dimensions::Few(ref dims) => dims,
            Dimensions::Many(ref dims) => dims.as_ref(),
        };

        dims[dimension as usize + 1]
    }

    /// The product of the number of elements of each dimension.
    pub fn size(&self) -> usize {
        if self.n_dimensions() == 0 {
            return 0;
        }

        let dims = match self {
            Dimensions::Few(ref dims) => &dims[1..dims[0] as usize + 1],
            Dimensions::Many(ref dims) => &dims[1..dims[0] as usize + 1],
        };
        dims.iter().product()
    }

    /// Calculates the linear index of `dim_index` corresponding to a multidimensional array of
    /// shape `self`. Returns an error if the index is not valid for this shape.
    pub fn index_of<D: Into<Dimensions>>(&self, dim_index: D) -> JlrsResult<usize> {
        let dim_index = dim_index.into();
        self.check_bounds(&dim_index)?;

        let idx = match self.n_dimensions() {
            0 => 0,
            _ => {
                let mut d_it = dim_index.as_slice().iter().rev();
                let acc = d_it.next().unwrap();

                d_it.zip(self.as_slice().iter().rev().skip(1))
                    .fold(*acc, |acc, (dim, sz)| dim + sz * acc)
            }
        };

        Ok(idx)
    }

    /// Returns the raw dimensions as a slice.
    pub fn as_slice(&self) -> &[usize] {
        match self {
            Dimensions::Few(ref v) => &v[1..v[0] as usize + 1],
            Dimensions::Many(ref v) => &v[1..],
        }
    }

    fn check_bounds(&self, dim_index: &Dimensions) -> JlrsResult<()> {
        if self.n_dimensions() != dim_index.n_dimensions() {
            Err(JlrsError::InvalidIndex(dim_index.clone(), self.clone()))?;
        }

        for i in 0..self.n_dimensions() {
            if self.n_elements(i) < dim_index.n_elements(i) {
                Err(JlrsError::InvalidIndex(dim_index.clone(), self.clone()))?;
            }
        }

        Ok(())
    }
}

#[cfg_attr(tarpaulin, skip)]
impl Debug for Dimensions {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut f = f.debug_tuple("");

        for d in self.as_slice() {
            f.field(&d);
        }

        f.finish()
    }
}

#[cfg_attr(tarpaulin, skip)]
impl Display for Dimensions {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut f = f.debug_tuple("");

        for d in self.as_slice() {
            f.field(&d);
        }

        f.finish()
    }
}

impl Into<Dimensions> for usize {
    fn into(self) -> Dimensions {
        Dimensions::Few([1, self, 0, 0])
    }
}

impl Into<Dimensions> for () {
    fn into(self) -> Dimensions {
        Dimensions::Few([0, 0, 0, 0])
    }
}

impl Into<Dimensions> for (usize,) {
    fn into(self) -> Dimensions {
        Dimensions::Few([1, self.0, 0, 0])
    }
}

impl Into<Dimensions> for (usize, usize) {
    fn into(self) -> Dimensions {
        Dimensions::Few([2, self.0, self.1, 0])
    }
}

impl Into<Dimensions> for (usize, usize, usize) {
    fn into(self) -> Dimensions {
        Dimensions::Few([3, self.0, self.1, self.2])
    }
}

impl Into<Dimensions> for (usize, usize, usize, usize) {
    fn into(self) -> Dimensions {
        Dimensions::Many(Box::new([4, self.0, self.1, self.2, self.3]))
    }
}

impl Into<Dimensions> for (usize, usize, usize, usize, usize) {
    fn into(self) -> Dimensions {
        Dimensions::Many(Box::new([5, self.0, self.1, self.2, self.3, self.4]))
    }
}

impl Into<Dimensions> for (usize, usize, usize, usize, usize, usize) {
    fn into(self) -> Dimensions {
        Dimensions::Many(Box::new([
            6, self.0, self.1, self.2, self.3, self.4, self.5,
        ]))
    }
}

impl Into<Dimensions> for (usize, usize, usize, usize, usize, usize, usize) {
    fn into(self) -> Dimensions {
        Dimensions::Many(Box::new([
            7, self.0, self.1, self.2, self.3, self.4, self.5, self.6,
        ]))
    }
}

impl Into<Dimensions> for (usize, usize, usize, usize, usize, usize, usize, usize) {
    fn into(self) -> Dimensions {
        Dimensions::Many(Box::new([
            8, self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7,
        ]))
    }
}

impl Into<Dimensions> for &[usize] {
    fn into(self) -> Dimensions {
        let nd = self.len();
        let mut v: Vec<usize> = Vec::with_capacity(nd + 1);
        v.push(nd);
        v.extend_from_slice(self);
        Dimensions::Many(v.into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::Dimensions;
    #[test]
    fn convert_usize() {
        let d: Dimensions = 4.into();
        assert_eq!(d.n_dimensions(), 1);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.size(), 4);
    }

    #[test]
    fn convert_tuple_0d() {
        let d: Dimensions = ().into();
        assert_eq!(d.n_dimensions(), 0);
        assert_eq!(d.n_elements(0), 0);
        assert_eq!(d.size(), 0);
    }

    #[test]
    fn convert_tuple_1d() {
        let d: Dimensions = (4,).into();
        assert_eq!(d.n_dimensions(), 1);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.size(), 4);
    }

    #[test]
    fn convert_tuple_2d() {
        let d: Dimensions = (4, 3).into();
        assert_eq!(d.n_dimensions(), 2);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.size(), 12);
    }

    #[test]
    fn convert_tuple_3d() {
        let d: Dimensions = (4, 3, 2).into();
        assert_eq!(d.n_dimensions(), 3);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.n_elements(2), 2);
        assert_eq!(d.size(), 24);
    }

    #[test]
    fn convert_tuple_4d() {
        let d: Dimensions = (4, 3, 2, 1).into();
        assert_eq!(d.n_dimensions(), 4);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.n_elements(2), 2);
        assert_eq!(d.n_elements(3), 1);
        assert_eq!(d.size(), 24);
    }

    #[test]
    fn convert_tuple_5d() {
        let d: Dimensions = (4, 3, 2, 1, 2).into();
        assert_eq!(d.n_dimensions(), 5);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.n_elements(2), 2);
        assert_eq!(d.n_elements(3), 1);
        assert_eq!(d.n_elements(4), 2);
        assert_eq!(d.size(), 48);
    }

    #[test]
    fn convert_tuple_6d() {
        let d: Dimensions = (4, 3, 2, 1, 2, 3).into();
        assert_eq!(d.n_dimensions(), 6);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.n_elements(2), 2);
        assert_eq!(d.n_elements(3), 1);
        assert_eq!(d.n_elements(4), 2);
        assert_eq!(d.n_elements(5), 3);
        assert_eq!(d.size(), 144);
    }

    #[test]
    fn convert_tuple_7d() {
        let d: Dimensions = (4, 3, 2, 1, 2, 3, 2).into();
        assert_eq!(d.n_dimensions(), 7);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.n_elements(2), 2);
        assert_eq!(d.n_elements(3), 1);
        assert_eq!(d.n_elements(4), 2);
        assert_eq!(d.n_elements(5), 3);
        assert_eq!(d.n_elements(6), 2);
        assert_eq!(d.size(), 288);
    }

    #[test]
    fn convert_tuple_8d() {
        let d: Dimensions = (4, 3, 2, 1, 2, 3, 2, 4).into();
        assert_eq!(d.n_dimensions(), 8);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.n_elements(2), 2);
        assert_eq!(d.n_elements(3), 1);
        assert_eq!(d.n_elements(4), 2);
        assert_eq!(d.n_elements(5), 3);
        assert_eq!(d.n_elements(6), 2);
        assert_eq!(d.n_elements(7), 4);
        assert_eq!(d.size(), 1152);
    }

    #[test]
    fn convert_tuple_nd() {
        let v = [1, 2, 3];
        let d: Dimensions = v.as_ref().into();
        assert_eq!(d.n_dimensions(), 3);
        assert_eq!(d.n_elements(0), 1);
        assert_eq!(d.n_elements(1), 2);
        assert_eq!(d.n_elements(2), 3);
        assert_eq!(d.size(), 6);
    }
}
