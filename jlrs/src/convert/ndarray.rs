//! Borrow data from Julia arrays as `ndarray`'s `ArrayView` and `ArrayViewMut`.

use ndarray::{ArrayView, ArrayViewMut, IntoDimension, IxDyn, Shape, ShapeBuilder};

use super::compatible::{Compatible, CompatibleCast};
use crate::data::{
    layout::{is_bits::IsBits, valid_layout::ValidField},
    managed::array::data::{
        accessor::{Accessor, BitsAccessor, BitsAccessorMut, InlineAccessor},
        copied::CopiedArray,
    },
};

fn into_shape<'scope, 'data, T, A: Accessor<'scope, 'data, T, N>, const N: isize>(
    a: &A,
) -> Shape<IxDyn> {
    let array = a.array();
    let dims = array.dimensions();
    let slice = dims.as_slice();
    if N == 1 || slice.len() == 1 {
        let slice = unsafe { &[slice[0].dims_cell.get()][..] };
        slice.into_dimension().f()
    } else {
        let n = slice.len();
        let ptr = slice.as_ptr().cast::<usize>();
        let slice = unsafe { std::slice::from_raw_parts(ptr, n) };
        slice.into_dimension().f()
    }
}

/// Trait to borrow Julia arrays with inline data as `ndarray`'s `ArrayView`.
pub trait NdArrayView<'view, T>: private::NdArrayPriv {
    /// Borrow the data in the array as an `ArrayView`.
    fn array_view(&'view self) -> ArrayView<'view, T, IxDyn>;

    /// Borrow the data in the array as an `ArrayView` of a compatible type `U`.
    fn compatible_array_view<U>(&'view self) -> ArrayView<'view, U, IxDyn>
    where
        T: Compatible<U>;
}

/// Trait to borrow Julia arrays with inline data as `ndarray`'s `ArrayViewMut`.
pub trait NdArrayViewMut<'view, T>: NdArrayView<'view, T> {
    /// Mutably borrow the data in the array as an `ArrayViewMut`.
    fn array_view_mut(&'view mut self) -> ArrayViewMut<'view, T, IxDyn>;

    /// Mutably borrow the data in the array as an `ArrayViewMut` of a compatible type `U`.
    fn compatible_array_view_mut<U>(&'view mut self) -> ArrayViewMut<'view, U, IxDyn>
    where
        T: Compatible<U>;
}

impl<'borrow: 'view, 'view, 'array, 'data, T, L, const N: isize> NdArrayView<'view, L>
    for BitsAccessor<'borrow, 'array, 'data, T, L, N>
where
    L: IsBits + ValidField,
{
    fn array_view(&'view self) -> ArrayView<'view, L, IxDyn> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = into_shape(self);
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }

    fn compatible_array_view<U>(&'view self) -> ArrayView<'view, U, IxDyn>
    where
        L: Compatible<U>,
    {
        let shape = into_shape(self);
        ArrayView::from_shape(shape, self.as_slice().compatible_cast()).unwrap()
    }
}

impl<'borrow: 'view, 'view, 'array, 'data, T, L, const N: isize> NdArrayView<'view, L>
    for BitsAccessorMut<'borrow, 'array, 'data, T, L, N>
where
    L: IsBits + ValidField,
{
    fn array_view(&'view self) -> ArrayView<'view, L, IxDyn> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = into_shape(self);
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }

    fn compatible_array_view<U>(&'view self) -> ArrayView<'view, U, IxDyn>
    where
        L: Compatible<U>,
    {
        let shape = into_shape(self);
        ArrayView::from_shape(shape, self.as_slice().compatible_cast()).unwrap()
    }
}

impl<'borrow: 'view, 'view, 'array, 'data, T, L, const N: isize> NdArrayView<'view, L>
    for InlineAccessor<'borrow, 'array, 'data, T, L, N>
where
    L: ValidField,
{
    fn array_view(&'view self) -> ArrayView<'view, L, IxDyn> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = into_shape(self);
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }

    fn compatible_array_view<U>(&'view self) -> ArrayView<'view, U, IxDyn>
    where
        L: Compatible<U>,
    {
        let shape = into_shape(self);
        ArrayView::from_shape(shape, self.as_slice().compatible_cast()).unwrap()
    }
}

impl<'borrow: 'view, 'view, 'array, 'data, T, L, const N: isize> NdArrayViewMut<'view, L>
    for BitsAccessorMut<'borrow, 'array, 'data, T, L, N>
where
    L: ValidField + IsBits,
{
    fn array_view_mut(&'view mut self) -> ArrayViewMut<'view, L, IxDyn> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = into_shape(self);
        ArrayViewMut::from_shape(shape, self.as_mut_slice()).unwrap()
    }

    fn compatible_array_view_mut<U>(&'view mut self) -> ArrayViewMut<'view, U, IxDyn>
    where
        L: Compatible<U>,
    {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = into_shape(self);
        ArrayViewMut::from_shape(shape, self.as_mut_slice().compatible_cast_mut()).unwrap()
    }
}

impl<'view, T> NdArrayView<'view, T> for CopiedArray<T> {
    fn array_view(&'view self) -> ArrayView<'view, T, IxDyn> {
        let shape = self.dimensions().as_slice().into_dimension().f();
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }

    fn compatible_array_view<U>(&'view self) -> ArrayView<'view, U, IxDyn>
    where
        T: Compatible<U>,
    {
        let shape = self.dimensions().as_slice().into_dimension().f();
        ArrayView::from_shape(shape, self.as_slice().compatible_cast()).unwrap()
    }
}

impl<'view, T> NdArrayViewMut<'view, T> for CopiedArray<T> {
    fn array_view_mut(&'view mut self) -> ArrayViewMut<'view, T, IxDyn> {
        let shape = self.dimensions().as_slice().into_dimension().f();
        ArrayViewMut::from_shape(shape, self.as_mut_slice()).unwrap()
    }

    fn compatible_array_view_mut<U>(&'view mut self) -> ArrayViewMut<'view, U, IxDyn>
    where
        T: Compatible<U>,
    {
        let shape = self.dimensions().as_slice().into_dimension().f();
        ArrayViewMut::from_shape(shape, self.as_mut_slice().compatible_cast_mut()).unwrap()
    }
}

mod private {
    use crate::data::managed::array::data::{
        accessor::{BitsAccessor, BitsAccessorMut, InlineAccessor},
        copied::CopiedArray,
    };

    pub trait NdArrayPriv {}
    impl<'borrow, 'array, 'data, T, L, const N: isize> NdArrayPriv
        for InlineAccessor<'borrow, 'array, 'data, T, L, N>
    {
    }

    impl<'borrow, 'array, 'data, T, L, const N: isize> NdArrayPriv
        for BitsAccessor<'borrow, 'array, 'data, T, L, N>
    {
    }

    impl<'borrow, 'array, 'data, T, L, const N: isize> NdArrayPriv
        for BitsAccessorMut<'borrow, 'array, 'data, T, L, N>
    {
    }

    impl<T> NdArrayPriv for CopiedArray<T> {}
}
