//! Borrow data from Julia arrays as `ndarray`'s `ArrayView` and `ArrayViewMut`.

use crate::{
    layout::valid_layout::ValidLayout,
    wrappers::ptr::array::data::{
        accessor::{BitsArrayAccessor, InlinePtrArrayAccessor, Mutability, Mutable},
        copied::CopiedArray,
    },
};
use ndarray::{ArrayView, ArrayViewMut, Dim, IntoDimension, IxDynImpl, ShapeBuilder};

/// Trait to borrow Julia arrays with inline data as `ndarray`'s `ArrayView`.
pub trait NdArrayView<'borrow, T>: private::NdArrayPriv {
    /// Borrow the data in the array as an `ArrayView`.
    fn array_view<'frame>(&'borrow self) -> ArrayView<'borrow, T, Dim<IxDynImpl>>;
}

/// Trait to borrow Julia arrays with inline data as `ndarray`'s `ArrayView` and `ArrayViewMut`.
pub trait NdArrayViewMut<'borrow, T>: NdArrayView<'borrow, T> {
    /// Mutably borrow the data in the array as an `ArrayViewMut`.
    fn array_view_mut<'frame>(&'borrow mut self) -> ArrayViewMut<'borrow, T, Dim<IxDynImpl>>;
}

impl<'borrow, 'array, 'data, T, M> NdArrayView<'borrow, T>
    for BitsArrayAccessor<'borrow, 'array, 'data, T, M>
where
    M: Mutability,
    T: ValidLayout + Clone,
{
    fn array_view<'frame>(&'borrow self) -> ArrayView<'borrow, T, Dim<IxDynImpl>> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }
}

impl<'borrow, 'array, 'data, T, M> NdArrayView<'borrow, T>
    for InlinePtrArrayAccessor<'borrow, 'array, 'data, T, M>
where
    M: Mutability,
    T: ValidLayout + Clone,
{
    fn array_view<'frame>(&'borrow self) -> ArrayView<'borrow, T, Dim<IxDynImpl>> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }
}

impl<'borrow, 'array, 'data, T> NdArrayViewMut<'borrow, T>
    for BitsArrayAccessor<'borrow, 'array, 'data, T, Mutable<'borrow, T>>
where
    T: ValidLayout + Clone,
{
    fn array_view_mut<'frame>(&'borrow mut self) -> ArrayViewMut<'borrow, T, Dim<IxDynImpl>> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayViewMut::from_shape(shape, self.as_mut_slice()).unwrap()
    }
}

impl<'borrow, T> NdArrayView<'borrow, T> for CopiedArray<T>
where
    T: ValidLayout + Clone,
{
    fn array_view<'frame>(&'borrow self) -> ArrayView<'borrow, T, Dim<IxDynImpl>> {
        let shape = self.dimensions().as_slice().into_dimension().f();
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }
}

impl<'borrow, T> NdArrayViewMut<'borrow, T> for CopiedArray<T>
where
    T: ValidLayout + Clone,
{
    fn array_view_mut<'frame>(&'borrow mut self) -> ArrayViewMut<'borrow, T, Dim<IxDynImpl>> {
        let shape = self.dimensions().as_slice().into_dimension().f();
        ArrayViewMut::from_shape(shape, self.as_mut_slice()).unwrap()
    }
}

mod private {
    use crate::{
        layout::valid_layout::ValidLayout,
        wrappers::ptr::array::data::{
            accessor::{BitsArrayAccessor, InlinePtrArrayAccessor, Mutability},
            copied::CopiedArray,
        },
    };

    pub trait NdArrayPriv {}
    impl<'borrow, 'array, 'data, T, M> NdArrayPriv
        for InlinePtrArrayAccessor<'borrow, 'array, 'data, T, M>
    where
        T: Clone + ValidLayout,
        M: Mutability,
    {
    }

    impl<'borrow, 'array, 'data, T, M> NdArrayPriv for BitsArrayAccessor<'borrow, 'array, 'data, T, M>
    where
        T: Clone + ValidLayout,
        M: Mutability,
    {
    }

    impl<T> NdArrayPriv for CopiedArray<T> where T: Clone + ValidLayout {}
}

#[cfg(test)]
#[cfg(feature = "sync-rt")]
mod tests {
    use super::{NdArrayView, NdArrayViewMut};
    use crate::wrappers::ptr::array::Array;
    use crate::{prelude::TypedArray, util::JULIA};

    #[test]
    fn bits_array_view() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, mut frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed =
                        unsafe { Array::from_slice_unchecked(&mut frame, slice, (3, 2))? };

                    let data = borrowed.bits_data::<usize, _>(&mut frame)?;
                    let x = data[(2, 1)];

                    let array = data.array_view();
                    assert_eq!(array[[2, 1]], x);

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn bits_array_view_mut() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, mut frame| unsafe {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = Array::from_slice_unchecked(&mut frame, slice, (3, 2))?;
                    let mut inline = borrowed.bits_data_mut::<usize, _>(&mut frame)?;
                    let x = inline[(2, 1)];

                    inline[(2, 1)] = x + 1;

                    let mut array = inline.array_view_mut();
                    assert_eq!(array[[2, 1]], x + 1);
                    array[[2, 1]] -= 1;

                    let inline = borrowed.bits_data_mut::<usize, _>(&mut frame)?;
                    assert_eq!(inline[(2, 1)], x);
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn inline_array_view() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, mut frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed =
                        unsafe { Array::from_slice_unchecked(&mut frame, slice, (3, 2))? };

                    let data = borrowed.inline_data::<usize, _>(&mut frame)?;
                    let x = data[(2, 1)];

                    let array = data.array_view();
                    assert_eq!(array[[2, 1]], x);

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn copied_array_view() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, mut frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed =
                        unsafe { TypedArray::from_slice_unchecked(&mut frame, slice, (3, 2))? };
                    let copied = borrowed.copy_inline_data(&frame)?;

                    let x = copied[(2, 1)];

                    let array = copied.array_view();
                    assert_eq!(array[[2, 1]], x);

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn copied_array_view_mut() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, mut frame| unsafe {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = Array::from_slice_unchecked(&mut frame, slice, (3, 2))?;
                    let mut copied = borrowed.copy_inline_data(&frame)?;
                    let x = copied[(2, 1)];

                    copied[(2, 1)] = x + 1;

                    let mut array = copied.array_view_mut();
                    assert_eq!(array[[2, 1]], x + 1);
                    array[[2, 1]] -= 1;

                    let inline = borrowed.bits_data_mut::<usize, _>(&mut frame)?;
                    assert_eq!(inline[(2, 1)], x);
                    Ok(())
                })
                .unwrap();
        });
    }
}
