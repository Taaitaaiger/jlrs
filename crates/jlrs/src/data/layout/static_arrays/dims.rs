//! Static dimensions

/// Dimensions for a rank-R array.
///
/// Implementations of this trait should be created with [`define_dims`].
pub trait Dims<const R: usize>: 'static + Clone {
    /// The size of each dimension
    const PARAMS: [usize; R];
    // The total number of elements
    const N: usize;
}

/// Dimension of a rank-1 array
#[derive(Clone)]
pub struct Dims1D<const N: usize>;

impl<const N: usize> Dims<1> for Dims1D<N> {
    const PARAMS: [usize; 1] = [N];
    const N: usize = N;
}

/// Dimension of a rank-2 array
#[derive(Clone)]
pub struct Dims2D<const ROWS: usize, const COLS: usize>;

impl<const ROWS: usize, const COLS: usize> Dims<2> for Dims2D<ROWS, COLS> {
    const PARAMS: [usize; 2] = [ROWS, COLS];
    const N: usize = ROWS * COLS;
}

/// Dimension of a rank-3 array
#[derive(Clone)]
pub struct Dims3D<const ROWS: usize, const COLS: usize, const Z: usize>;

impl<const ROWS: usize, const COLS: usize, const Z: usize> Dims<3> for Dims3D<ROWS, COLS, Z> {
    const PARAMS: [usize; 3] = [ROWS, COLS, Z];
    const N: usize = ROWS * COLS * Z;
}

#[doc(hidden)]
#[macro_export]
macro_rules! product {
    ($t:ident, $($x:ident),+) => {
        $t * $crate::product!($($x),+)
    };
    ($t:ident) => {
        $t
    };
}

/// Macro to generate new implementations of `Dims`
///
/// Example for rank-4 arrays:
///
/// ```
/// jlrs::define_dims!(pub Dims4D<A, B, C, D; 4>);
/// ```
///
/// This creates `Dims4D<const A: usize, const B: usize, const C: usize, const D: usize>`
/// and implements `Dims<4>` for it.
#[macro_export]
macro_rules! define_dims {
    ($vis:vis $name:ident<$($n:ident),+; $r:literal>) => {
        #[derive(Clone)]
        $vis struct $name<$(const $n: usize),+>;
        impl <$(const $n: usize),+> $crate::data::layout::static_arrays::dims::Dims<$r> for $name<$($n),+> {
            const PARAMS: [usize; $r] = [$($n),+];
            const N: usize = $crate::product!($($n),+);
        }
    };
}
