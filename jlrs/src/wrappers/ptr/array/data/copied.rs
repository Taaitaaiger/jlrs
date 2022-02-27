//! Array data that has been copied from Julia to Rust.

use crate::wrappers::ptr::array::dimensions::{Dimensions, Dims};
use std::ops::{Index, IndexMut};

/// An n-dimensional array whose contents have been copied from Julia to Rust. You can create this
/// struct by calling [`Array::copy_inline_data`]. The data has a column-major order and can be
/// indexed with anything that implements [`Dims`].
///
/// [`Array::copy_inline_data`]: crate::wrappers::ptr::array::Array::copy_inline_data
#[derive(Debug)]
pub struct CopiedArray<T> {
    data: Box<[T]>,
    dimensions: Dimensions,
}

impl<T> CopiedArray<T> {
    // Safety: dimensions must be valid for the size of data
    pub(crate) unsafe fn new(data: Box<[T]>, dimensions: Dimensions) -> Self {
        CopiedArray { data, dimensions }
    }

    /// Turn the array into a tuple containing its data in column-major order and its dimensions.
    pub fn splat(self) -> (Box<[T]>, Dimensions) {
        (self.data, self.dimensions)
    }

    /// Returns a reference to the element at the given n-dimensional index if the index is valid,
    /// `None` otherwise.
    pub fn get<D: Dims>(&self, idx: D) -> Option<&T> {
        Some(&self.data[self.dimensions.index_of(idx).ok()?])
    }

    /// Returns a mutable reference to the element at the given n-dimensional index if the index
    /// is valid, `None` otherwise.
    pub fn get_mut<D: Dims>(&mut self, idx: D) -> Option<&mut T> {
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

impl<T, D: Dims> Index<D> for CopiedArray<T> {
    type Output = T;
    fn index(&self, idx: D) -> &T {
        &self.data[self.dimensions.index_of(idx).unwrap()]
    }
}

impl<T, D: Dims> IndexMut<D> for CopiedArray<T> {
    fn index_mut(&mut self, idx: D) -> &mut T {
        &mut self.data[self.dimensions.index_of(idx).unwrap()]
    }
}
