use super::super::{Array, Dimensions};
use crate::memory::traits::frame::Frame;
use jl_sys::jl_array_data;
use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice,
};

/// Mutably borrowed inline array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements `Into<Dimensions>`; see [`Dimensions`] for more
/// information.
pub struct InlineArrayDataMut<'borrow, 'array, 'frame, 'data, T, F>
where
    'array: 'borrow + 'frame,
    F: Frame<'frame>,
{
    array: Array<'array, 'data>,
    dimensions: Dimensions,
    _notsendsync: PhantomData<*const ()>,
    _borrow: PhantomData<&'borrow F>,
    _frame: PhantomData<&'frame ()>,
    _type: PhantomData<T>,
}

impl<'borrow, 'array, 'frame, 'data, T, F> InlineArrayDataMut<'borrow, 'array, 'frame, 'data, T, F>
where
    'array: 'borrow + 'frame,
    'borrow: 'data,
    F: Frame<'frame>,
{
    pub(crate) unsafe fn new(
        array: Array<'array, 'data>,
        dimensions: Dimensions,
        _: &'borrow mut F,
    ) -> Self {
        InlineArrayDataMut {
            array,
            dimensions,
            _notsendsync: PhantomData,
            _frame: PhantomData,
            _borrow: PhantomData,
            _type: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D: Into<Dimensions>>(&self, index: D) -> Option<&T> {
        unsafe {
            let idx = self.dimensions.index_of(index).ok()?;
            jl_array_data(self.array.inner().as_ptr().cast())
                .cast::<T>()
                .add(idx)
                .as_ref()
        }
    }

    /// Get a mutable reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get_mut<D: Into<Dimensions>>(&mut self, index: D) -> Option<&mut T> {
        unsafe {
            let idx = self.dimensions.index_of(index).ok()?;
            jl_array_data(self.array.inner().as_ptr().cast())
                .cast::<T>()
                .add(idx)
                .as_mut()
        }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let len = self.dimensions.size();
            let data = jl_array_data(self.array.inner().as_ptr().cast()).cast::<T>();
            slice::from_raw_parts(data, len)
        }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn into_slice(self) -> &'borrow [T] {
        unsafe {
            let len = self.dimensions.size();
            let data = jl_array_data(self.array.inner().as_ptr().cast()).cast::<T>();
            slice::from_raw_parts(data, len)
        }
    }

    /// Returns the array's data as a mutable slice, the data is in column-major order.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            let len = self.dimensions.size();
            let data = jl_array_data(self.array.inner().as_ptr().cast()).cast::<T>();
            slice::from_raw_parts_mut(data, len)
        }
    }

    /// Returns the array's data as a mutable slice, the data is in column-major order.
    pub fn into_mut_slice(self) -> &'borrow mut [T] {
        unsafe {
            let len = self.dimensions.size();
            let data = jl_array_data(self.array.inner().as_ptr().cast()).cast::<T>();
            slice::from_raw_parts_mut(data, len)
        }
    }

    /// Returns a reference to the array's dimensions.
    pub fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }
}

/// Mutably borrowed value array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements `Into<Dimensions>`; see [`Dimensions`] for more
/// information.
impl<'borrow, 'array, 'frame, 'data, T, F, D> Index<D>
    for InlineArrayDataMut<'borrow, 'array, 'frame, 'data, T, F>
where
    'array: 'borrow + 'frame,
    'borrow: 'data,
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let idx = self.dimensions.index_of(index).unwrap();
            jl_array_data(self.array.inner().as_ptr().cast())
                .cast::<T>()
                .add(idx)
                .as_ref()
                .unwrap()
        }
    }
}

impl<'borrow, 'array, 'frame, 'data, T, F, D> IndexMut<D>
    for InlineArrayDataMut<'borrow, 'array, 'frame, 'data, T, F>
where
    'array: 'borrow + 'frame,
    'borrow: 'data,
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    fn index_mut(&mut self, index: D) -> &mut Self::Output {
        unsafe {
            let idx = self.dimensions.index_of(index).unwrap();
            jl_array_data(self.array.inner().as_ptr().cast())
                .cast::<T>()
                .add(idx)
                .as_mut()
                .unwrap()
        }
    }
}

/// Mutably borrowed inline array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements `Into<Dimensions>`; see [`Dimensions`] for more
/// information.
pub struct UnrestrictedInlineArrayDataMut<'borrow, 'frame, T, F: Frame<'frame>> {
    data: &'borrow mut [T],
    dimensions: Dimensions,
    _notsendsync: PhantomData<*const ()>,
    _borrow: PhantomData<&'borrow F>,
    _frame: PhantomData<&'frame ()>,
}

impl<'borrow, 'frame, T, F> UnrestrictedInlineArrayDataMut<'borrow, 'frame, T, F>
where
    F: Frame<'frame>,
{
    pub(crate) unsafe fn new(
        data: &'borrow mut [T],
        dimensions: Dimensions,
        _: &'borrow F,
    ) -> Self {
        UnrestrictedInlineArrayDataMut {
            data,
            dimensions,
            _notsendsync: PhantomData,
            _frame: PhantomData,
            _borrow: PhantomData,
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

/// Mutably borrowed value array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements `Into<Dimensions>`; see [`Dimensions`] for more
/// information.
impl<'borrow, 'frame, T, D, F> Index<D> for UnrestrictedInlineArrayDataMut<'borrow, 'frame, T, F>
where
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        &self.data[self.dimensions.index_of(index).unwrap()]
    }
}

impl<'borrow, 'frame, T, D, F> IndexMut<D> for UnrestrictedInlineArrayDataMut<'borrow, 'frame, T, F>
where
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    fn index_mut(&mut self, index: D) -> &mut Self::Output {
        &mut self.data[self.dimensions.index_of(index).unwrap()]
    }
}
