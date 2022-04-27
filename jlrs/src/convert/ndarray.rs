//! Borrow data from Julia arrays as `ndarray`'s `ArrayView` and `ArrayViewMut`.
//!
//! This module defines a single trait, `NdArray`, that provides methods that return an immutable
//! or a mutable view of the array data and is implemented by `Array` and `TypedArray` from jlrs.
//! It's easier to use this trait with `TypedArray`, you'll likely have to provide type
//! annotations with `Array`. To make this trait available you must enable the `jlrs-ndarray`
//! feature.

use crate::{
    error::{JlrsError, JlrsResult},
    layout::valid_layout::ValidLayout,
    memory::frame::Frame,
    wrappers::ptr::array::dimensions::Dims,
    wrappers::ptr::array::{Array, TypedArray},
};
use ndarray::{ArrayView, ArrayViewMut, Dim, IntoDimension, IxDynImpl, ShapeBuilder};
use std::fmt::Debug;

/// Trait to borrow Julia arrays with inline data as `ndarray`'s `ArrayView` and `ArrayViewMut`.
pub trait NdArray<'borrow, T>: private::NdArray {
    /// Borrow the data in the array as an `ArrayView`. Returns an error if the wrong type is
    /// provided or the data is not stored inline.
    fn array_view<'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ArrayView<'borrow, T, Dim<IxDynImpl>>>
    where
        F: Frame<'frame>,
        T: ValidLayout + Clone;

    /// Mutably borrow the data in the array as an `ArrayViewMut`. Returns an error if the wrong
    /// type is provided or the data is not stored inline.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    unsafe fn array_view_mut<'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ArrayViewMut<'borrow, T, Dim<IxDynImpl>>>
    where
        F: Frame<'frame>,
        T: ValidLayout + Clone;
}

impl<'frame, 'data: 'borrow, 'borrow, T: ValidLayout + Clone> NdArray<'borrow, T>
    for Array<'frame, 'data>
{
    fn array_view<'fr, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ArrayView<'borrow, T, Dim<IxDynImpl>>>
    where
        F: Frame<'fr>,
        T: ValidLayout + Clone,
    {
        let data = self.inline_data::<T, _>(&*frame)?;
        let shape = data
            .dimensions()
            .into_dimensions()
            .as_slice()
            .into_dimension()
            .f();
        match ArrayView::from_shape(shape, data.into_slice()) {
            Ok(arr) => Ok(arr),
            Err(e) => Err(JlrsError::other(e))?,
        }
    }

    unsafe fn array_view_mut<'fr, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ArrayViewMut<'borrow, T, Dim<IxDynImpl>>>
    where
        F: Frame<'fr>,
        T: ValidLayout + Clone,
    {
        let data = self.inline_data_mut::<T, _>(&mut *frame)?;
        let shape = data
            .dimensions()
            .into_dimensions()
            .as_slice()
            .into_dimension()
            .f();
        let raw = data.into_mut_slice();
        match ArrayViewMut::from_shape(shape, raw) {
            Ok(arr) => Ok(arr),
            Err(e) => Err(JlrsError::other(e))?,
        }
    }
}

impl<'frame: 'borrow, 'data: 'borrow, 'borrow, T: ValidLayout + Clone + Debug> NdArray<'borrow, T>
    for TypedArray<'frame, 'data, T>
{
    fn array_view<'fr, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ArrayView<'borrow, T, Dim<IxDynImpl>>>
    where
        F: Frame<'fr>,
        T: ValidLayout,
    {
        let data = self.inline_data(&*frame)?;
        let shape = data
            .dimensions()
            .into_dimensions()
            .as_slice()
            .into_dimension()
            .f();
        match ArrayView::from_shape(shape, data.into_slice()) {
            Ok(arr) => Ok(arr),
            Err(e) => Err(JlrsError::other(e))?,
        }
    }

    unsafe fn array_view_mut<'fr, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ArrayViewMut<'borrow, T, Dim<IxDynImpl>>>
    where
        F: Frame<'fr>,
        T: ValidLayout,
    {
        let data = self.inline_data_mut(&mut *frame)?;
        let shape = data
            .dimensions()
            .into_dimensions()
            .as_slice()
            .into_dimension()
            .f();
        let raw = data.into_mut_slice();
        match ArrayViewMut::from_shape(shape, raw) {
            Ok(arr) => Ok(arr),
            Err(e) => Err(JlrsError::other(e))?,
        }
    }
}

mod private {
    use crate::{
        layout::valid_layout::ValidLayout,
        wrappers::ptr::array::{Array, TypedArray},
    };
    use std::fmt::Debug;

    pub trait NdArray {}
    impl<'frame, 'data> NdArray for Array<'frame, 'data> {}
    impl<'frame, 'data, T> NdArray for TypedArray<'frame, 'data, T> where T: Clone + ValidLayout + Debug {}
}

#[cfg(test)]
#[cfg(feature = "sync-rt")]
mod tests {
    use super::NdArray;
    use crate::util::JULIA;
    use crate::wrappers::ptr::array::Array;
    use ndarray::{ArrayView, ArrayViewMut, IxDyn};

    #[test]
    fn array_view() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = Array::from_slice(&mut *frame, slice, (3, 2))?;
                    let x = borrowed.inline_data::<usize, _>(&mut *frame)?[(1, 0)];

                    let array: ArrayView<usize, _> = borrowed.array_view(&mut *frame)?;
                    assert_eq!(array[IxDyn(&[1, 0])], x);

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn array_view_wrong_type() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = Array::from_slice(&mut *frame, slice, (3, 2))?;

                    let view: Result<ArrayView<isize, _>, _> = borrowed.array_view(&mut *frame);
                    assert!(view.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn array_view_mut() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| unsafe {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = Array::from_slice(&mut *frame, slice, (3, 2))?;
                    let mut inline = borrowed.inline_data_mut::<usize, _>(&mut *frame)?;
                    let x = inline[(1, 0)];

                    inline[(1, 0)] = x + 1;

                    let mut array: ArrayViewMut<usize, _> = borrowed.array_view_mut(&mut *frame)?;
                    assert_eq!(array[IxDyn(&[1, 0])], x + 1);
                    array[IxDyn(&[1, 0])] -= 1;

                    let inline = borrowed.inline_data_mut::<usize, _>(&mut *frame)?;
                    assert_eq!(inline[(1, 0)], x);
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn array_view_mut_wrong_type() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| unsafe {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = Array::from_slice(&mut *frame, slice, (3, 2))?;

                    let view: Result<ArrayViewMut<isize, _>, _> =
                        borrowed.array_view_mut(&mut *frame);
                    assert!(view.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn typed_array_view() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = Array::from_slice(&mut *frame, slice, (3, 2))?;

                    let x = borrowed.inline_data(&mut *frame)?[(1, 0)];

                    let array: ArrayView<usize, _> = borrowed.array_view(&mut *frame)?;
                    assert_eq!(array[IxDyn(&[1, 0])], x);

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn typed_array_view_mut() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| unsafe {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = Array::from_slice(&mut *frame, slice, (3, 2))?;

                    let mut inline = borrowed.inline_data_mut::<usize, _>(&mut *frame)?;
                    let x = inline[(1, 0)];

                    inline[(1, 0)] = x + 1;

                    let mut array: ArrayViewMut<usize, _> = borrowed.array_view_mut(&mut *frame)?;
                    assert_eq!(array[IxDyn(&[1, 0])], x + 1);
                    array[IxDyn(&[1, 0])] -= 1;

                    let inline = borrowed.inline_data_mut::<usize, _>(&mut *frame)?;
                    assert_eq!(inline[(1, 0)], x);
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn example() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .scope(|_global, frame| {
                    let mut data = vec![1u64, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = Array::from_slice(&mut *frame, slice, (3, 2))?;

                    let _array = borrowed.try_as_typed::<u64>()?.array_view(&mut *frame)?;

                    Ok(())
                })
                .unwrap();
        });
    }
}
