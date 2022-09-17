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
pub trait NdArrayView<'view, T>: private::NdArrayPriv {
    /// Borrow the data in the array as an `ArrayView`.
    fn array_view(&'view self) -> ArrayView<'view, T, Dim<IxDynImpl>>;
}

/// Trait to borrow Julia arrays with inline data as `ndarray`'s `ArrayView` and `ArrayViewMut`.
pub trait NdArrayViewMut<'view, T>: NdArrayView<'view, T> {
    /// Mutably borrow the data in the array as an `ArrayViewMut`.
    fn array_view_mut(&'view mut self) -> ArrayViewMut<'view, T, Dim<IxDynImpl>>;
}

impl<'borrow: 'view, 'view, 'array, 'data, T, M> NdArrayView<'view, T>
    for BitsArrayAccessor<'borrow, 'array, 'data, T, M>
where
    M: Mutability,
    T: ValidLayout + Clone,
{
    fn array_view(&'view self) -> ArrayView<'view, T, Dim<IxDynImpl>> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }
}

impl<'borrow: 'view, 'view, 'array, 'data, T, M> NdArrayView<'view, T>
    for InlinePtrArrayAccessor<'borrow, 'array, 'data, T, M>
where
    M: Mutability,
    T: ValidLayout + Clone,
{
    fn array_view(&'view self) -> ArrayView<'view, T, Dim<IxDynImpl>> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }
}

impl<'borrow: 'view, 'view, 'array, 'data, T> NdArrayViewMut<'view, T>
    for BitsArrayAccessor<'borrow, 'array, 'data, T, Mutable<'borrow, T>>
where
    T: ValidLayout + Clone,
{
    fn array_view_mut(&'view mut self) -> ArrayViewMut<'view, T, Dim<IxDynImpl>> {
        // Safety: while the array is borrowed nothing can be pushed or popped from it.
        let shape = unsafe { self.dimensions().as_slice().into_dimension().f() };
        ArrayViewMut::from_shape(shape, self.as_mut_slice()).unwrap()
    }
}

impl<'view, T> NdArrayView<'view, T> for CopiedArray<T>
where
    T: ValidLayout + Clone,
{
    fn array_view(&'view self) -> ArrayView<'view, T, Dim<IxDynImpl>> {
        let shape = self.dimensions().as_slice().into_dimension().f();
        ArrayView::from_shape(shape, self.as_slice()).unwrap()
    }
}

impl<'view, T> NdArrayViewMut<'view, T> for CopiedArray<T>
where
    T: ValidLayout + Clone,
{
    fn array_view_mut(&'view mut self) -> ArrayViewMut<'view, T, Dim<IxDynImpl>> {
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
    use crate::util::test::JULIA;
    use crate::wrappers::ptr::array::{Array, TypedArray};

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

                    let data = unsafe { borrowed.bits_data::<usize>()? };
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
                    let mut borrowed = Array::from_slice_unchecked(&mut frame, slice, (3, 2))?;

                    let mut inline = borrowed.bits_data_mut::<usize>()?;
                    let x = inline[(2, 1)];

                    inline[(2, 1)] = x + 1;

                    let mut array = inline.array_view_mut();
                    assert_eq!(array[[2, 1]], x + 1);
                    array[[2, 1]] -= 1;

                    std::mem::drop(inline);
                    let inline = borrowed.bits_data_mut::<usize>()?;
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

                    let data = unsafe { borrowed.inline_data::<usize>()? };
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
                    let copied = unsafe { borrowed.copy_inline_data()? };

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
                    let mut borrowed = Array::from_slice_unchecked(&mut frame, slice, (3, 2))?;
                    let mut copied = unsafe {borrowed.copy_inline_data()?};
                    let x = copied[(2, 1)];

                    copied[(2, 1)] = x + 1;

                    let mut array = copied.array_view_mut();
                    assert_eq!(array[[2, 1]], x + 1);
                    array[[2, 1]] -= 1;

                    let inline = borrowed.bits_data_mut::<usize>()?;
                    assert_eq!(inline[(2, 1)], x);
                    Ok(())
                })
                .unwrap();
        });
    }
}
