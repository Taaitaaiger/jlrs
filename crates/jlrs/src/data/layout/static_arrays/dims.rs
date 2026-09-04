pub trait Dims<const R: usize>: 'static + Clone {
    const PARAMS: [usize; R];
    const N: usize;
}

#[derive(Clone)]
pub struct Dims1D<const N: usize>;

impl<const N: usize> Dims<1> for Dims1D<N> {
    const PARAMS: [usize; 1] = [N];
    const N: usize = N;
}

#[derive(Clone)]
pub struct Dims2D<const ROWS: usize, const COLS: usize>;

impl<const ROWS: usize, const COLS: usize> Dims<2> for Dims2D<ROWS, COLS> {
    const PARAMS: [usize; 2] = [ROWS, COLS];
    const N: usize = ROWS * COLS;
}

#[derive(Clone)]
pub struct Dims3D<const ROWS: usize, const COLS: usize, const Z: usize>;

impl<const ROWS: usize, const COLS: usize, const Z: usize> Dims<3> for Dims3D<ROWS, COLS, Z> {
    const PARAMS: [usize; 3] = [ROWS, COLS, Z];
    const N: usize = ROWS * COLS * Z;
}

#[macro_export]
macro_rules! product {
    ($t:ident, $($x:ident),+) => {
        $t * product!($($x),+)
    };
    ($t:ident) => {
        $t
    };
}

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
