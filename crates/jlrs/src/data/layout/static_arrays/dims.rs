pub trait Dims<const R: usize>: 'static + Clone {
    const PARAMS: [usize; R];
}

#[derive(Clone)]
pub struct Dims1D<const N: usize>;

impl<const N: usize> Dims<1> for Dims1D<N> {
    const PARAMS: [usize; 1] = [N];
}

#[derive(Clone)]
pub struct Dims2D<const ROWS: usize, const COLS: usize>;

impl<const ROWS: usize, const COLS: usize> Dims<2> for Dims2D<ROWS, COLS> {
    const PARAMS: [usize; 2] = [ROWS, COLS];
}

#[derive(Clone)]
pub struct Dims3D<const ROWS: usize, const COLS: usize, const Z: usize>;

impl<const ROWS: usize, const COLS: usize, const Z: usize> Dims<3> for Dims3D<ROWS, COLS, Z> {
    const PARAMS: [usize; 3] = [ROWS, COLS, Z];
}

// TODO: macro
