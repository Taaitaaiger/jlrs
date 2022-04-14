//! Value array data borrowed from Julia.

use crate::{
    error::{JlrsError, JlrsResult, CANNOT_DISPLAY_TYPE},
    memory::frame::Frame,
    private::Private,
    wrappers::ptr::{
        array::{
            dimensions::{ArrayDimensions, Dims},
            Array,
        },
        private::Wrapper as _,
        value::Value,
        ValueRef, Wrapper, WrapperRef,
    },
};
use jl_sys::jl_array_ptr_set;
use std::{marker::PhantomData, ops::Index, ptr::null_mut, slice};

/// Immutably borrowed value array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements [`Dims`].
#[repr(transparent)]
pub struct ValueArrayData<'borrow, 'array, 'data, T = ValueRef<'array, 'data>>
where
    T: WrapperRef<'array, 'data>,
{
    array: Array<'array, 'data>,
    _marker: PhantomData<&'borrow [T]>,
}

impl<'borrow, 'array, 'data, T> ValueArrayData<'borrow, 'array, 'data, T>
where
    T: WrapperRef<'array, 'data>,
{
    // Safety: The representation of T and the element type must match
    pub(crate) unsafe fn new<'frame, F>(array: Array<'array, 'data>, _: &'borrow F) -> Self
    where
        F: Frame<'frame>,
    {
        ValueArrayData {
            array,
            _marker: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D>(&self, index: D) -> Option<T>
    where
        D: Dims,
    {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).ok()?;
            self.array.data_ptr().cast::<T>().add(idx).as_ref().cloned()
        }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let arr_data = self.array.data_ptr().cast::<T>();

            let dims = ArrayDimensions::new(self.array);
            let n_elems = dims.size();
            slice::from_raw_parts(arr_data, n_elems)
        }
    }

    /// Returns the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        ArrayDimensions::new(self.array)
    }
}

impl<'borrow, 'array, 'data, D, T> Index<D> for ValueArrayData<'borrow, 'array, 'data, T>
where
    D: Dims,
    T: WrapperRef<'array, 'data>,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).unwrap();
            self.array.data_ptr().cast::<T>().add(idx).as_ref().unwrap()
        }
    }
}

/// Mutably borrowed value array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements [`Dims`].
#[repr(transparent)]
pub struct ValueArrayDataMut<'borrow, 'array, 'data, T = ValueRef<'array, 'data>>
where
    T: WrapperRef<'array, 'data>,
{
    array: Array<'array, 'data>,
    _marker: PhantomData<&'borrow mut [T]>,
}

impl<'borrow, 'array, 'data, T> ValueArrayDataMut<'borrow, 'array, 'data, T>
where
    T: WrapperRef<'array, 'data>,
{
    // Safety: The representation of T and the element type must match
    pub(crate) unsafe fn new<'frame, F>(array: Array<'array, 'data>, _: &'borrow mut F) -> Self
    where
        F: Frame<'frame>,
    {
        ValueArrayDataMut {
            array,
            _marker: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D>(&self, index: D) -> Option<T>
    where
        D: Dims,
    {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).ok()?;
            self.array.data_ptr().cast::<T>().add(idx).as_ref().cloned()
        }
    }

    /// Set the value at `index` to `value` if `value` has a type that's compatible with this array.
    pub fn set<D>(&mut self, index: D, value: Option<Value<'_, 'data>>) -> JlrsResult<()>
    where
        D: Dims,
    {
        unsafe {
            let ptr = self.array.unwrap(Private);
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index)?;

            let data_ptr = if let Some(value) = value {
                if !self
                    .array
                    .element_type()
                    .subtype(value.datatype().as_value())
                {
                    let element_type_str = self
                        .array
                        .element_type()
                        .display_string_or(CANNOT_DISPLAY_TYPE);
                    let value_type_str = value.datatype().display_string_or(CANNOT_DISPLAY_TYPE);
                    Err(JlrsError::ElementTypeError {
                        element_type_str,
                        value_type_str,
                    })?;
                }

                value.unwrap(Private)
            } else {
                null_mut()
            };

            jl_array_ptr_set(ptr.cast(), idx, data_ptr.cast());
        }
        Ok(())
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let arr_data = self.array.data_ptr().cast::<T>();
            let dims = ArrayDimensions::new(self.array);
            let n_elems = dims.size();
            slice::from_raw_parts(arr_data, n_elems)
        }
    }

    /// Returns the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        ArrayDimensions::new(self.array)
    }
}

impl<'borrow, 'array, 'data, D, T> Index<D> for ValueArrayDataMut<'borrow, 'array, 'data, T>
where
    D: Dims,
    T: WrapperRef<'array, 'data>,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).unwrap();
            self.array.data_ptr().cast::<T>().add(idx).as_ref().unwrap()
        }
    }
}

/// Mutably borrowed value array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements [`Dims`].
#[repr(transparent)]
pub struct UnrestrictedValueArrayDataMut<'borrow, 'array, 'data, T = ValueRef<'array, 'data>>
where
    T: WrapperRef<'array, 'data>,
{
    array: Array<'array, 'data>,
    _marker: PhantomData<&'borrow mut [T]>,
}

impl<'borrow, 'array, 'data, T> UnrestrictedValueArrayDataMut<'borrow, 'array, 'data, T>
where
    T: WrapperRef<'array, 'data>,
{
    // Safety: The representation of T and the element type must match
    pub(crate) unsafe fn new<'frame, F>(array: Array<'array, 'data>, _: &'borrow F) -> Self
    where
        F: Frame<'frame>,
    {
        UnrestrictedValueArrayDataMut {
            array,
            _marker: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D>(&self, index: D) -> Option<T>
    where
        D: Dims,
    {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).ok()?;
            self.array.data_ptr().cast::<T>().add(idx).as_ref().cloned()
        }
    }

    /// Set the value at `index` to `value` if `value` has a type that's compatible with this array.
    pub fn set<D>(&mut self, index: D, value: Option<Value<'_, 'data>>) -> JlrsResult<()>
    where
        D: Dims,
    {
        unsafe {
            let ptr = self.array.unwrap(Private);
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index)?;

            let data_ptr = if let Some(value) = value {
                if !self
                    .array
                    .element_type()
                    .subtype(value.datatype().as_value())
                {
                    let element_type_str = self
                        .array
                        .element_type()
                        .display_string_or(CANNOT_DISPLAY_TYPE);
                    let value_type_str = value.datatype().display_string_or(CANNOT_DISPLAY_TYPE);
                    Err(JlrsError::ElementTypeError {
                        element_type_str,
                        value_type_str,
                    })?;
                }

                value.unwrap(Private)
            } else {
                null_mut()
            };

            jl_array_ptr_set(ptr.cast(), idx, data_ptr.cast());
        }
        Ok(())
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let arr_data = self.array.data_ptr().cast::<T>();
            let dims = ArrayDimensions::new(self.array);
            let n_elems = dims.size();
            slice::from_raw_parts(arr_data, n_elems)
        }
    }

    /// Returns the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        ArrayDimensions::new(self.array)
    }
}

impl<'borrow, 'array, 'data, D, T> Index<D>
    for UnrestrictedValueArrayDataMut<'borrow, 'array, 'data, T>
where
    D: Dims,
    T: WrapperRef<'array, 'data>,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).unwrap();
            self.array.data_ptr().cast::<T>().add(idx).as_ref().unwrap()
        }
    }
}
