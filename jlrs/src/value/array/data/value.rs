use super::super::{Array, Dimensions};
use crate::{
    error::{JlrsError, JlrsResult},
    memory::traits::frame::Frame,
    value::wrapper_ref::ValueRef,
};
use jl_sys::{jl_array_data, jl_array_ptr_set};
use std::{marker::PhantomData, ops::Index, ptr::null_mut, slice};

pub struct ValueArrayDataMut<'borrow, 'array, 'data, 'frame, F: Frame<'frame>> {
    array: Array<'array, 'data>,
    dimensions: Dimensions,
    _notsendsync: PhantomData<*const ()>,
    _borrow: PhantomData<&'borrow mut F>,
    _frame: PhantomData<&'frame ()>,
}

impl<'borrow, 'array, 'data, 'frame, F> ValueArrayDataMut<'borrow, 'array, 'data, 'frame, F>
where
    F: Frame<'frame>,
{
    pub(crate) unsafe fn new(
        array: Array<'array, 'data>,
        dimensions: Dimensions,
        _: &'borrow mut F,
    ) -> Self {
        ValueArrayDataMut {
            array,
            dimensions,
            _notsendsync: PhantomData,
            _borrow: PhantomData,
            _frame: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D: Into<Dimensions>>(&self, index: D) -> Option<ValueRef<'array, 'data>> {
        unsafe {
            let idx = self.dimensions.index_of(index).ok()?;
            let elem = jl_array_data(self.array.inner().as_ptr().cast())
                .cast::<ValueRef>()
                .add(idx)
                .read();
            Some(elem)
        }
    }

    /// Set the value at `index` to `value` if `value` has a type that's compatible with this array.
    pub fn set<'va, 'da: 'data, D: Into<Dimensions>>(
        &mut self,
        index: D,
        value: ValueRef<'_, 'da>,
    ) -> JlrsResult<()> {
        unsafe {
            let ptr = self.array.inner().as_ptr();
            let idx = self.dimensions.index_of(index)?;

            let data_ptr = if let Some(value) = value.assume_reachable() {
                if !self
                    .array
                    .element_type()
                    .subtype(value.datatype().as_value())
                {
                    Err(JlrsError::InvalidArrayType)?;
                }

                value.inner().as_ptr()
            } else {
                null_mut()
            };

            jl_array_ptr_set(ptr.cast(), idx, data_ptr.cast());
        }
        Ok(())
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[ValueRef<'array, 'data>] {
        unsafe {
            let arr_data = jl_array_data(self.array.inner().as_ptr().cast()).cast::<ValueRef>();
            let n_elems = self.dimensions.size();
            slice::from_raw_parts(arr_data, n_elems)
        }
    }

    /// Returns a reference to the array's dimensions.
    pub fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }
}

impl<'borrow, 'value, 'data, 'frame, D, F> Index<D>
    for ValueArrayDataMut<'borrow, 'value, 'data, 'frame, F>
where
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    type Output = ValueRef<'value, 'data>;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let idx = self.dimensions.index_of(index).unwrap();
            &*(jl_array_data(self.array.inner().as_ptr().cast())
                .cast::<ValueRef>()
                .add(idx))
        }
    }
}

pub struct UnrestrictedValueArrayDataMut<'borrow, 'array, 'data, 'frame, F: Frame<'frame>> {
    array: Array<'array, 'data>,
    dimensions: Dimensions,
    _notsendsync: PhantomData<*const ()>,
    _borrow: PhantomData<&'borrow F>,
    _frame: PhantomData<&'frame ()>,
}

impl<'borrow, 'array, 'data, 'fr, F> UnrestrictedValueArrayDataMut<'borrow, 'array, 'data, 'fr, F>
where
    F: Frame<'fr>,
{
    pub(crate) unsafe fn new(
        array: Array<'array, 'data>,
        dimensions: Dimensions,
        _: &'borrow F,
    ) -> Self {
        UnrestrictedValueArrayDataMut {
            array,
            dimensions,
            _notsendsync: PhantomData,
            _borrow: PhantomData,
            _frame: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D: Into<Dimensions>>(&self, index: D) -> Option<ValueRef<'array, 'data>> {
        unsafe {
            let idx = self.dimensions.index_of(index).ok()?;
            let elem = jl_array_data(self.array.inner().as_ptr().cast())
                .cast::<ValueRef>()
                .add(idx)
                .read();
            Some(elem)
        }
    }

    /// Set the value at `index` to `value` if `value` has a type that's compatible with this array.
    pub fn set<'va, 'da: 'data, D: Into<Dimensions>>(
        &mut self,
        index: D,
        value: ValueRef<'_, 'da>,
    ) -> JlrsResult<()> {
        unsafe {
            let ptr = self.array.inner().as_ptr();
            let idx = self.dimensions.index_of(index)?;

            let data_ptr = if let Some(value) = value.assume_reachable() {
                if !self
                    .array
                    .element_type()
                    .subtype(value.datatype().as_value())
                {
                    Err(JlrsError::InvalidArrayType)?;
                }

                value.inner().as_ptr()
            } else {
                null_mut()
            };

            jl_array_ptr_set(ptr.cast(), idx, data_ptr.cast());
        }
        Ok(())
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[ValueRef<'array, 'data>] {
        unsafe {
            let arr_data = jl_array_data(self.array.inner().as_ptr().cast()).cast::<ValueRef>();
            let n_elems = self.dimensions.size();
            slice::from_raw_parts(arr_data, n_elems)
        }
    }

    /// Returns a reference to the array's dimensions.
    pub fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }
}

impl<'borrow, 'value, 'data, 'frame, D, F> Index<D>
    for UnrestrictedValueArrayDataMut<'borrow, 'value, 'data, 'frame, F>
where
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    type Output = ValueRef<'value, 'data>;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let idx = self.dimensions.index_of(index).unwrap();
            &*(jl_array_data(self.array.inner().as_ptr().cast())
                .cast::<ValueRef>()
                .add(idx))
        }
    }
}
