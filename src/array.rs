//! Support for n-dimensional arrays and their dimensions.

/// An n-dimensional array whose contents have been copied from Julia to Rust. You can create this
/// struct by calling [`Value::try_unbox`]. In order to unbox arrays that contain `bool`s or
/// `char`s, you can unbox them as `Array<i8>` and `Array<u32>` respectively. The data has a
/// column-major layout.
///
/// [`Value::try_unbox`]: ../value/struct.Value.html#method.try_unbox
pub struct Array<T> {
    data: Vec<T>,
    dimensions: Dimensions,
}

impl<T> Array<T> {
    pub(crate) fn new(data: Vec<T>, dimensions: Dimensions) -> Self {
        Array { data, dimensions }
    }

    /// Turn the array into a tuple containing its data in column-major order and its dimensions.
    pub fn splat(self) -> (Vec<T>, Dimensions) {
        (self.data, self.dimensions)
    }
}

/// The dimensions of an n-dimensional array.
#[derive(Debug, Clone)]
pub enum Dimensions {
    #[doc(hidden)]
    Few([u64; 4]),
    #[doc(hidden)]
    Many(Box<[u64]>),
}

impl Dimensions {
    /// Returns the number of dimensions.
    pub fn n_dimensions(&self) -> u64 {
        match self {
            Dimensions::Few([n, _, _, _]) => *n,
            Dimensions::Many(ref dims) => dims[0],
        }
    }

    /// Returns the number of elements of the nth dimension. Indexing starts at 0.
    pub fn n_elements(&self, dimension: u64) -> u64 {
        let dims = match self {
            Dimensions::Few(ref dims) => dims,
            Dimensions::Many(ref dims) => dims.as_ref(),
        };
        dims[dimension as usize + 1]
    }

    /// The product of the number of elements of each dimension.
    pub fn size(&self) -> u64 {
        let dims = match self {
            Dimensions::Few(ref dims) => &dims[1..dims[0] as usize + 1],
            Dimensions::Many(ref dims) => &dims[1..dims[0] as usize + 1],
        };
        dims.iter().product()
    }

    pub(crate) fn as_slice(&self) -> &[u64] {
        match self {
            Dimensions::Few(ref v) => &v[1..v[0] as usize + 1],
            Dimensions::Many(ref v) => &v[1..],
        }
    }
}

impl Into<Dimensions> for u64 {
    fn into(self) -> Dimensions {
        Dimensions::Few([1, self, 0, 0])
    }
}

impl Into<Dimensions> for (u64,) {
    fn into(self) -> Dimensions {
        Dimensions::Few([1, self.0, 0, 0])
    }
}

impl Into<Dimensions> for (u64, u64) {
    fn into(self) -> Dimensions {
        Dimensions::Few([2, self.0, self.1, 0])
    }
}

impl Into<Dimensions> for (u64, u64, u64) {
    fn into(self) -> Dimensions {
        Dimensions::Few([3, self.0, self.1, self.2])
    }
}

impl Into<Dimensions> for (u64, u64, u64, u64) {
    fn into(self) -> Dimensions {
        Dimensions::Many(Box::new([4, self.0, self.1, self.2, self.3]))
    }
}

impl Into<Dimensions> for (u64, u64, u64, u64, u64) {
    fn into(self) -> Dimensions {
        Dimensions::Many(Box::new([5, self.0, self.1, self.2, self.3, self.4]))
    }
}

impl Into<Dimensions> for (u64, u64, u64, u64, u64, u64) {
    fn into(self) -> Dimensions {
        Dimensions::Many(Box::new([
            6, self.0, self.1, self.2, self.3, self.4, self.5,
        ]))
    }
}

impl Into<Dimensions> for (u64, u64, u64, u64, u64, u64, u64) {
    fn into(self) -> Dimensions {
        Dimensions::Many(Box::new([
            7, self.0, self.1, self.2, self.3, self.4, self.5, self.6,
        ]))
    }
}

impl Into<Dimensions> for (u64, u64, u64, u64, u64, u64, u64, u64) {
    fn into(self) -> Dimensions {
        Dimensions::Many(Box::new([
            8, self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7,
        ]))
    }
}

impl Into<Dimensions> for &[u64] {
    fn into(self) -> Dimensions {
        let nd = self.len();
        let mut v: Vec<u64> = Vec::with_capacity(nd + 1);
        v.push(nd as _);
        v.extend_from_slice(self);
        Dimensions::Many(v.into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::Dimensions;
    #[test]
    fn convert_u64() {
        let d: Dimensions = 4.into();
        assert_eq!(d.n_dimensions(), 1);
        assert_eq!(d.n_elements(0), 4);
        assert_eq!(d.size(), 4);
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
