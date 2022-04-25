//! Inline array data borrowed from Julia.

use crate::{
    memory::frame::Frame,
    wrappers::ptr::array::{
        dimensions::{ArrayDimensions, Dims},
        Array,
    },
};
use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice,
};

/// Immutably borrowed inline array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements [`Dims`].
#[repr(transparent)]
#[derive(Clone)]
pub struct InlineArrayData<'borrow, 'array, 'data, T> {
    array: Array<'array, 'data>,
    _marker: PhantomData<&'borrow [T]>,
}

impl<'borrow, 'array, 'data, T> InlineArrayData<'borrow, 'array, 'data, T> {
    // Safety: The representation of T and the element type must match
    pub(crate) unsafe fn new<'frame, F>(array: Array<'array, 'data>, _: &'borrow F) -> Self
    where
        F: Frame<'frame>,
    {
        InlineArrayData {
            array,
            _marker: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D>(&self, index: D) -> Option<&T>
    where
        D: Dims,
    {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index).ok()?;
            self.array.data_ptr().cast::<T>().add(idx).as_ref()
        }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let len = dims.size();
            let data = self.array.data_ptr().cast::<T>();
            slice::from_raw_parts(data, len)
        }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn into_slice(self) -> &'borrow [T] {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let len = dims.size();
            let data = self.array.data_ptr().cast::<T>();
            slice::from_raw_parts(data, len)
        }
    }

    /// Returns the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        ArrayDimensions::new(self.array)
    }
}

impl<'borrow, 'array, 'data, T, D> Index<D> for InlineArrayData<'borrow, 'array, 'data, T>
where
    D: Dims,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index).unwrap();
            self.array.data_ptr().cast::<T>().add(idx).as_ref().unwrap()
        }
    }
}

/// Mutably borrowed inline array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements [`Dims`].
#[repr(transparent)]
pub struct InlineArrayDataMut<'borrow, 'array, 'data, T> {
    array: Array<'array, 'data>,
    _marker: PhantomData<&'borrow mut [T]>,
}

impl<'borrow, 'array, 'data, T> InlineArrayDataMut<'borrow, 'array, 'data, T> {
    // Safety: The representation of T and the element type must match
    pub(crate) unsafe fn new<'frame, F>(array: Array<'array, 'data>, _: &'borrow mut F) -> Self
    where
        F: Frame<'frame>,
    {
        InlineArrayDataMut {
            array,
            _marker: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D>(&self, index: D) -> Option<&T>
    where
        D: Dims,
    {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index).ok()?;
            self.array.data_ptr().cast::<T>().add(idx).as_ref()
        }
    }

    /// Get a mutable reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get_mut<D>(&mut self, index: D) -> Option<&mut T>
    where
        D: Dims,
    {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index).ok()?;
            self.array.data_ptr().cast::<T>().add(idx).as_mut()
        }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let len = dims.size();
            let data = self.array.data_ptr().cast::<T>();
            slice::from_raw_parts(data, len)
        }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn into_slice(self) -> &'borrow [T] {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let len = dims.size();
            let data = self.array.data_ptr().cast::<T>();
            slice::from_raw_parts(data, len)
        }
    }

    /// Returns the array's data as a mutable slice, the data is in column-major order.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let len = dims.size();
            let data = self.array.data_ptr().cast::<T>();
            slice::from_raw_parts_mut(data, len)
        }
    }

    /// Returns the array's data as a mutable slice, the data is in column-major order.
    pub fn into_mut_slice(self) -> &'borrow mut [T] {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let len = dims.size();
            let data = self.array.data_ptr().cast::<T>();
            slice::from_raw_parts_mut(data, len)
        }
    }

    /// Returns a reference to the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        ArrayDimensions::new(self.array)
    }
}

/// Mutably borrowed inline array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements [`Dims`].
impl<'borrow, 'array, 'data, T, D> Index<D> for InlineArrayDataMut<'borrow, 'array, 'data, T>
where
    D: Dims,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index).unwrap();
            self.array.data_ptr().cast::<T>().add(idx).as_ref().unwrap()
        }
    }
}

impl<'borrow, 'array, 'data, T, D> IndexMut<D> for InlineArrayDataMut<'borrow, 'array, 'data, T>
where
    D: Dims,
{
    fn index_mut(&mut self, index: D) -> &mut Self::Output {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index).unwrap();
            self.array.data_ptr().cast::<T>().add(idx).as_mut().unwrap()
        }
    }
}

/// Mutably borrowed inline array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements [`Dims`].
#[repr(transparent)]
pub struct UnrestrictedInlineArrayDataMut<'borrow, 'array, 'data, T> {
    array: Array<'array, 'data>,
    _marker: PhantomData<&'borrow mut [T]>,
}

impl<'borrow, 'array, 'data, T> UnrestrictedInlineArrayDataMut<'borrow, 'array, 'data, T> {
    // Safety: The representation of T and the element type must match
    pub(crate) unsafe fn new<'frame, F>(array: Array<'array, 'data>, _: &'borrow F) -> Self
    where
        F: Frame<'frame>,
    {
        UnrestrictedInlineArrayDataMut {
            array,
            _marker: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D>(&self, index: D) -> Option<&T>
    where
        D: Dims,
    {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index).ok()?;
            self.array.data_ptr().cast::<T>().add(idx).as_ref()
        }
    }

    /// Get a mutable reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get_mut<D>(&mut self, index: D) -> Option<&mut T>
    where
        D: Dims,
    {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index).ok()?;
            self.array.data_ptr().cast::<T>().add(idx).as_mut()
        }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let len = dims.size();
            let data = self.array.data_ptr().cast::<T>();
            slice::from_raw_parts(data, len)
        }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn into_slice(self) -> &'borrow [T] {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let len = dims.size();
            let data = self.array.data_ptr().cast::<T>();
            slice::from_raw_parts(data, len)
        }
    }

    /// Returns the array's data as a mutable slice, the data is in column-major order.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let len = dims.size();
            let data = self.array.data_ptr().cast::<T>();
            slice::from_raw_parts_mut(data, len)
        }
    }

    /// Returns the array's data as a mutable slice, the data is in column-major order.
    pub fn into_mut_slice(self) -> &'borrow mut [T] {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let len = dims.size();
            let data = self.array.data_ptr().cast::<T>();
            slice::from_raw_parts_mut(data, len)
        }
    }

    /// Returns a reference to the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        ArrayDimensions::new(self.array)
    }
}

impl<'borrow, 'array, 'data, T, D> Index<D>
    for UnrestrictedInlineArrayDataMut<'borrow, 'array, 'data, T>
where
    D: Dims,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index).unwrap();
            self.array.data_ptr().cast::<T>().add(idx).as_ref().unwrap()
        }
    }
}

impl<'borrow, 'array, 'data, T, D> IndexMut<D>
    for UnrestrictedInlineArrayDataMut<'borrow, 'array, 'data, T>
where
    D: Dims,
{
    fn index_mut(&mut self, index: D) -> &mut Self::Output {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index).unwrap();
            self.array.data_ptr().cast::<T>().add(idx).as_mut().unwrap()
        }
    }
}
