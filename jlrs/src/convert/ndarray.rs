//! Borrow data from Julia arrays as `ndarray`'s `ArrayView` and `ArrayViewMut`.

use ndarray::{ArrayView, ArrayViewMut, Dim, IntoDimension, IxDynImpl, ShapeBuilder};

use super::compatible::{Compatible, CompatibleCast};
use crate::data::managed::array::data::{
    accessor::{BitsArrayAccessor, InlinePtrArrayAccessor, Mutability, Mutable},
    copied::CopiedArray,
};

/// Trait to borrow Julia arrays with inline data as `ndarray`'s `ArrayView`.
pub trait NdArrayView<'view, T>: private::NdArrayPriv {
    /// Borrow the data in the array as an `ArrayView`.
    fn array_view(&'view self) -> ArrayView<'view, T, Dim<IxDynImpl>>;

    /// Borrow the data in the array as an `ArrayView` of a compatible type `U`.
    fn compatible_array_view<U>(&'view self) -> ArrayView<'view, U, Dim<IxDynImpl>>
    where
        T: Compatible<U>;
}

/// Trait to borrow Julia arrays with inline data as `ndarray`'s `ArrayViewMut`.
pub trait NdArrayViewMut<'view, T>: NdArrayView<'view, T> {
    /// Mutably borrow the data in the array as an `ArrayViewMut`.
    fn array_view_mut(&'view mut self) -> ArrayViewMut<'view, T, Dim<IxDynImpl>>;

    /// Mutably borrow the data in the array as an `ArrayViewMut` of a compatible type `U`.
    fn compatible_array_view_mut<U>(&'view mut self) -> ArrayViewMut<'view, U, Dim<IxDynImpl>>
    where
        T: Compatible<U>;
}

impl<'borrow: 'view, 'view, 'array, 'data, T, M> NdArrayView<'view, T>
    for BitsArrayAccessor<'borrow, 'array, 'data, T, M>
where
    M: Mutability,
{
    fn array_view(&'view self) -> ArrayView<'view, T, Dim<IxDynImpl>> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }

    fn compatible_array_view<U>(&'view self) -> ArrayView<'view, U, Dim<IxDynImpl>>
    where
        T: Compatible<U>,
    {
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayView::from_shape(shape, self.as_slice().compatible_cast()).unwrap()
    }
}

impl<'borrow: 'view, 'view, 'array, 'data, T, M> NdArrayView<'view, T>
    for InlinePtrArrayAccessor<'borrow, 'array, 'data, T, M>
where
    M: Mutability,
{
    fn array_view(&'view self) -> ArrayView<'view, T, Dim<IxDynImpl>> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }

    fn compatible_array_view<U>(&'view self) -> ArrayView<'view, U, Dim<IxDynImpl>>
    where
        T: Compatible<U>,
    {
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayView::from_shape(shape, self.as_slice().compatible_cast()).unwrap()
    }
}

impl<'borrow: 'view, 'view, 'array, 'data, T> NdArrayViewMut<'view, T>
    for BitsArrayAccessor<'borrow, 'array, 'data, T, Mutable<'borrow, T>>
{
    fn array_view_mut(&'view mut self) -> ArrayViewMut<'view, T, Dim<IxDynImpl>> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayViewMut::from_shape(shape, self.as_mut_slice()).unwrap()
    }

    fn compatible_array_view_mut<U>(&'view mut self) -> ArrayViewMut<'view, U, Dim<IxDynImpl>>
    where
        T: Compatible<U>,
    {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayViewMut::from_shape(shape, self.as_mut_slice().compatible_cast_mut()).unwrap()
    }
}

impl<'view, T> NdArrayView<'view, T> for CopiedArray<T> {
    fn array_view(&'view self) -> ArrayView<'view, T, Dim<IxDynImpl>> {
        let shape = self.dimensions().as_slice().into_dimension().f();
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }

    fn compatible_array_view<U>(&'view self) -> ArrayView<'view, U, Dim<IxDynImpl>>
    where
        T: Compatible<U>,
    {
        let shape = self.dimensions().as_slice().into_dimension().f();
        ArrayView::from_shape(shape, self.as_slice().compatible_cast()).unwrap()
    }
}

impl<'view, T> NdArrayViewMut<'view, T> for CopiedArray<T> {
    fn array_view_mut(&'view mut self) -> ArrayViewMut<'view, T, Dim<IxDynImpl>> {
        let shape = self.dimensions().as_slice().into_dimension().f();
        ArrayViewMut::from_shape(shape, self.as_mut_slice()).unwrap()
    }

    fn compatible_array_view_mut<U>(&'view mut self) -> ArrayViewMut<'view, U, Dim<IxDynImpl>>
    where
        T: Compatible<U>,
    {
        let shape = self.dimensions().as_slice().into_dimension().f();
        ArrayViewMut::from_shape(shape, self.as_mut_slice().compatible_cast_mut()).unwrap()
    }
}

mod private {
    use crate::data::managed::array::data::{
        accessor::{BitsArrayAccessor, InlinePtrArrayAccessor, Mutability},
        copied::CopiedArray,
    };

    pub trait NdArrayPriv {}
    impl<'borrow, 'array, 'data, T, M> NdArrayPriv
        for InlinePtrArrayAccessor<'borrow, 'array, 'data, T, M>
    where
        M: Mutability,
    {
    }

    impl<'borrow, 'array, 'data, T, M> NdArrayPriv for BitsArrayAccessor<'borrow, 'array, 'data, T, M> where
        M: Mutability
    {
    }

    impl<T> NdArrayPriv for CopiedArray<T> {}
}
