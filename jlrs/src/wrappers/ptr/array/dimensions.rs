//! N-dimensional indexing.
//!
//! In order to access the data of an n-dimensional array, you'll need to use an n-dimensional
//! index. This functionality is provided by the [`Dims`] trait, any implementor of this trait
//! can be used as an n-dimensional index. The most important implementations are tuples (up to
//! and including four dimensions), and arrays and array slices of any number of dimensions. So,
//! if you want to access the third column of the second row of an array, you can use both
//! `[1, 2]` or `(1, 2)`. Note that unlike Julia, array indexing starts at 0.

use crate::{
    error::{JlrsError, JlrsResult},
    private::Private,
    wrappers::ptr::{array::Array, private::Wrapper},
};
use jl_sys::{jl_array_dims_ptr, jl_array_ndims};
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    marker::PhantomData,
};

/// Trait implemented by types that can be used as n-dimensional indices.
pub trait Dims: Sized + Debug {
    /// Returns the number of dimensions.
    fn n_dimensions(&self) -> usize;

    /// Returns the number of elements of the nth dimension. Indexing starts at 0.
    fn n_elements(&self, dimension: usize) -> usize;

    /// The total number of elements in the arry, ie the product of the number of elements of each
    /// dimension.
    fn size(&self) -> usize {
        let mut acc = 1;
        for i in 0..self.n_dimensions() {
            acc *= self.n_elements(i);
        }

        acc
    }

    /// Calculates the linear index for `dim_index` in an array with dimensions `self`. The
    /// default implementation is generally correct and should not be overridden.
    fn index_of<D: Dims>(&self, dim_index: D) -> JlrsResult<usize> {
        if self.n_dimensions() != dim_index.n_dimensions() {
            Err(JlrsError::InvalidIndex {
                idx: dim_index.into_dimensions(),
                sz: self.into_dimensions(),
            })?;
        }

        if self.n_dimensions() == 0 {
            return Ok(0);
        }

        let n_dims = self.n_dimensions();
        for dim in 0..n_dims {
            if self.n_elements(dim) <= dim_index.n_elements(dim) {
                Err(JlrsError::InvalidIndex {
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

    fn into_dimensions(&self) -> Dimensions {
        Dimensions::from_dims(self)
    }
}

/// Dimensions of a Julia array.
#[derive(Copy, Clone, Debug)]
pub struct ArrayDimensions<'scope> {
    n: usize,
    ptr: *mut usize,
    _marker: PhantomData<&'scope ()>,
}

impl<'scope> ArrayDimensions<'scope> {
    pub(crate) unsafe fn new(array: Array<'scope, '_>) -> Self {
        let array_ptr = array.unwrap(Private);
        let ptr = jl_array_dims_ptr(array_ptr);
        let n = jl_array_ndims(array_ptr) as usize;
        ArrayDimensions {
            ptr,
            n,
            _marker: PhantomData,
        }
    }
}

impl<'scope> Dims for ArrayDimensions<'scope> {
    fn n_dimensions(&self) -> usize {
        self.n
    }

    fn n_elements(&self, dimension: usize) -> usize {
        if dimension >= self.n {
            return 0;
        }

        unsafe { self.ptr.add(dimension).read() }
    }
}

impl Dims for () {
    fn n_dimensions(&self) -> usize {
        0
    }

    fn n_elements(&self, _: usize) -> usize {
        0
    }
}

impl Dims for usize {
    fn n_dimensions(&self) -> usize {
        1
    }

    fn n_elements(&self, dimension: usize) -> usize {
        if dimension == 0 {
            *self
        } else {
            0
        }
    }
}

impl Dims for (usize,) {
    fn n_dimensions(&self) -> usize {
        1
    }

    fn n_elements(&self, dimension: usize) -> usize {
        if dimension == 0 {
            self.0
        } else {
            0
        }
    }
}

impl Dims for (usize, usize) {
    fn n_dimensions(&self) -> usize {
        2
    }

    fn n_elements(&self, dimension: usize) -> usize {
        match dimension {
            0 => self.0,
            1 => self.1,
            _ => 0,
        }
    }
}

impl Dims for (usize, usize, usize) {
    fn n_dimensions(&self) -> usize {
        3
    }

    fn n_elements(&self, dimension: usize) -> usize {
        match dimension {
            0 => self.0,
            1 => self.1,
            2 => self.2,
            _ => 0,
        }
    }
}

impl Dims for (usize, usize, usize, usize) {
    fn n_dimensions(&self) -> usize {
        4
    }

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

impl<const N: usize> Dims for &[usize; N] {
    fn n_dimensions(&self) -> usize {
        N
    }

    fn n_elements(&self, dim: usize) -> usize {
        if dim < N {
            self[dim]
        } else {
            0
        }
    }
}

impl<const N: usize> Dims for [usize; N] {
    fn n_dimensions(&self) -> usize {
        N
    }

    fn n_elements(&self, dim: usize) -> usize {
        if dim < N {
            self[dim]
        } else {
            0
        }
    }
}

impl Dims for &[usize] {
    fn n_dimensions(&self) -> usize {
        self.len()
    }

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
    pub fn from_dims<D: Dims>(dims: &D) -> Self {
        match dims.n_dimensions() {
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

    /// Returns the raw dimensions as a slice.
    pub fn as_slice(&self) -> &[usize] {
        match self {
            Dimensions::Few(ref v) => &v[1..v[0] as usize + 1],
            Dimensions::Many(ref v) => &v[1..],
        }
    }
}

impl Dims for Dimensions {
    fn n_dimensions(&self) -> usize {
        match self {
            Dimensions::Few([n, _, _, _]) => *n,
            Dimensions::Many(ref dims) => dims[0],
        }
    }

    fn n_elements(&self, dim: usize) -> usize {
        if dim < self.n_dimensions() {
            match self {
                Dimensions::Few(dims) => dims[dim + 1],
                Dimensions::Many(ref dims) => dims[dim + 1],
            }
        } else {
            0
        }
    }

    fn size(&self) -> usize {
        if self.n_dimensions() == 0 {
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
        <Self as Debug>::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Dimensions, Dims};
    #[test]
    fn convert_usize() {
        let d: Dimensions = 4.into_dimensions();
        assert_eq!(d.n_dimensions(), 1);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.size(), 4);
    }

    #[test]
    fn convert_tuple_0d() {
        let d: Dimensions = ().into_dimensions();
        assert_eq!(d.n_dimensions(), 0);
        assert_eq!(d.n_elements(0), 0);
        assert_eq!(d.size(), 0);
    }

    #[test]
    fn convert_tuple_1d() {
        let d: Dimensions = (4,).into_dimensions();
        assert_eq!(d.n_dimensions(), 1);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.size(), 4);
    }

    #[test]
    fn convert_tuple_2d() {
        let d: Dimensions = (4, 3).into_dimensions();
        assert_eq!(d.n_dimensions(), 2);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.size(), 12);
    }

    #[test]
    fn convert_tuple_3d() {
        let d: Dimensions = (4, 3, 2).into_dimensions();
        assert_eq!(d.n_dimensions(), 3);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.n_elements(2), 2);
        assert_eq!(d.size(), 24);
    }

    #[test]
    fn convert_tuple_4d() {
        let d: Dimensions = (4, 3, 2, 1).into_dimensions();
        assert_eq!(d.n_dimensions(), 4);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.n_elements(1), 3);
        assert_eq!(d.n_elements(2), 2);
        assert_eq!(d.n_elements(3), 1);
        assert_eq!(d.size(), 24);
    }

    #[test]
    fn convert_tuple_5d() {
        let d: Dimensions = (&[4, 3, 2, 1, 2]).into_dimensions();
        assert_eq!(d.n_dimensions(), 5);
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
        assert_eq!(d.n_dimensions(), 3);
        assert_eq!(d.n_elements(0), 1);
        assert_eq!(d.n_elements(1), 2);
        assert_eq!(d.n_elements(2), 3);
        assert_eq!(d.size(), 6);
    }
}
