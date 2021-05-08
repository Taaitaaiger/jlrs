use crate::error::{JlrsError, JlrsResult};
use jl_sys::{jl_array_dim, jl_array_dims, jl_array_ndims, jl_array_nrows, jl_array_t};
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    usize,
};

/// The dimensions of an n-dimensional array, they represent either the shape of an array or an
/// index. Functions that need `Dimensions` as an input, which is currently limited to just
/// indexing this data, are generic and accept any type that implements `Into<Dimensions>`.
///
/// For a single dimension, you can use a `usize` value. For 0 up to and including 8 dimensions,
/// you can use tuples of `usize`. In general, you can use slices of `usize`:
///
/// ```
/// # use jlrs::value::array::dimensions::Dimensions;
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
    pub(crate) unsafe fn from_array(array: *mut jl_array_t) -> Self {
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

impl Debug for Dimensions {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut f = f.debug_tuple("");

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
