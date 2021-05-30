//! Value array data borrowed from Julia.

use crate::{
    error::{JlrsError, JlrsResult},
    memory::traits::frame::Frame,
    private::Private,
    wrappers::ptr::{
        array::{
            dimensions::{ArrayDimensions, Dims},
            Array,
        },
        private::Wrapper as _,
        value::Value,
        Ref, ValueRef, Wrapper,
    },
};
use jl_sys::{jl_array_data, jl_array_ptr_set};
use std::{marker::PhantomData, ops::Index, ptr::null_mut, slice};

/// Immutably borrowed value array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements [`Dims`].
#[repr(transparent)]
pub struct ValueArrayData<'borrow, 'array, 'data, T = Value<'array, 'data>>
where
    T: Wrapper<'array, 'data>,
{
    array: Array<'array, 'data>,
    _marker: PhantomData<&'borrow [Ref<'array, 'data, T>]>,
}

impl<'borrow, 'array, 'data, T> ValueArrayData<'borrow, 'array, 'data, T>
where
    T: Wrapper<'array, 'data>,
{
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
    pub fn get<D>(&self, index: D) -> Option<&Ref<'array, 'data, T>>
    where
        D: Dims,
    {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).ok()?;
            jl_array_data(self.array.unwrap(Private).cast())
                .cast::<Ref<T>>()
                .add(idx)
                .as_ref()
        }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[Ref<'array, 'data, T>] {
        unsafe {
            let arr_data = jl_array_data(self.array.unwrap(Private).cast()).cast::<Ref<T>>();

            let dims = ArrayDimensions::new(self.array);
            let n_elems = dims.size();
            slice::from_raw_parts(arr_data, n_elems)
        }
    }

    /// Returns the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        unsafe { ArrayDimensions::new(self.array) }
    }
}

impl<'borrow, 'array, 'data, D, T> Index<D> for ValueArrayData<'borrow, 'array, 'data, T>
where
    D: Dims,
    T: Wrapper<'array, 'data>,
{
    type Output = Ref<'array, 'data, T>;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).unwrap();
            jl_array_data(self.array.unwrap(Private).cast())
                .cast::<Ref<T>>()
                .add(idx)
                .as_ref()
                .unwrap()
        }
    }
}

/// Mutably borrowed value array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements [`Dims`].
#[repr(transparent)]
pub struct ValueArrayDataMut<'borrow, 'array, 'data, T = Value<'array, 'data>>
where
    T: Wrapper<'array, 'data>,
{
    array: Array<'array, 'data>,
    _marker: PhantomData<&'borrow [Ref<'array, 'data, T>]>,
}

impl<'borrow, 'array, 'data, T> ValueArrayDataMut<'borrow, 'array, 'data, T>
where
    T: Wrapper<'array, 'data>,
{
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
    pub fn get<D>(&self, index: D) -> Option<&Ref<'array, 'data, T>>
    where
        D: Dims,
    {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).ok()?;
            jl_array_data(self.array.unwrap(Private).cast())
                .cast::<Ref<T>>()
                .add(idx)
                .as_ref()
        }
    }

    /// Set the value at `index` to `value` if `value` has a type that's compatible with this array.
    pub fn set<'va, 'da: 'data, D>(&mut self, index: D, value: ValueRef<'va, 'da>) -> JlrsResult<()>
    where
        D: Dims,
    {
        unsafe {
            let ptr = self.array.unwrap(Private);
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index)?;

            let data_ptr = if let Some(value) = value.wrapper() {
                if !self
                    .array
                    .element_type()
                    .subtype(value.datatype().as_value())
                {
                    Err(JlrsError::InvalidArrayType)?;
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
    pub fn as_slice(&self) -> &[Ref<'array, 'data, T>] {
        unsafe {
            let arr_data = jl_array_data(self.array.unwrap(Private).cast()).cast::<Ref<T>>();
            let dims = ArrayDimensions::new(self.array);
            let n_elems = dims.size();
            slice::from_raw_parts(arr_data, n_elems)
        }
    }

    /// Returns the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        unsafe { ArrayDimensions::new(self.array) }
    }
}

impl<'borrow, 'array, 'data, D, T> Index<D> for ValueArrayDataMut<'borrow, 'array, 'data, T>
where
    D: Dims,
    T: Wrapper<'array, 'data>,
{
    type Output = ValueRef<'array, 'data>;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).unwrap();
            jl_array_data(self.array.unwrap(Private).cast())
                .cast::<ValueRef>()
                .add(idx)
                .as_ref()
                .unwrap()
        }
    }
}

/// Mutably borrowed value array data from Julia. The data has a column-major order and can be
/// indexed with anything that implements [`Dims`].
#[repr(transparent)]
pub struct UnrestrictedValueArrayDataMut<'borrow, 'array, 'data, T = Value<'array, 'data>>
where
    T: Wrapper<'array, 'data>,
{
    array: Array<'array, 'data>,
    _marker: PhantomData<&'borrow [Ref<'array, 'data, T>]>,
}

impl<'borrow, 'array, 'data, T> UnrestrictedValueArrayDataMut<'borrow, 'array, 'data, T>
where
    T: Wrapper<'array, 'data>,
{
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
    pub fn get<D>(&self, index: D) -> Option<&Ref<'array, 'data, T>>
    where
        D: Dims,
    {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).ok()?;
            jl_array_data(self.array.unwrap(Private).cast())
                .cast::<Ref<T>>()
                .add(idx)
                .as_ref()
        }
    }

    /// Set the value at `index` to `value` if `value` has a type that's compatible with this array.
    pub fn set<'va, 'da: 'data, D>(&mut self, index: D, value: ValueRef<'va, 'da>) -> JlrsResult<()>
    where
        D: Dims,
    {
        unsafe {
            let ptr = self.array.unwrap(Private);
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index)?;

            let data_ptr = if let Some(value) = value.wrapper() {
                if !self
                    .array
                    .element_type()
                    .subtype(value.datatype().as_value())
                {
                    Err(JlrsError::InvalidArrayType)?;
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
    pub fn as_slice(&self) -> &[Ref<'array, 'data, T>] {
        unsafe {
            let arr_data = jl_array_data(self.array.unwrap(Private).cast()).cast::<Ref<T>>();
            let dims = ArrayDimensions::new(self.array);
            let n_elems = dims.size();
            slice::from_raw_parts(arr_data, n_elems)
        }
    }

    /// Returns the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        unsafe { ArrayDimensions::new(self.array) }
    }
}

impl<'borrow, 'array, 'data, D, T> Index<D>
    for UnrestrictedValueArrayDataMut<'borrow, 'array, 'data, T>
where
    D: Dims,
    T: Wrapper<'array, 'data>,
{
    type Output = Ref<'array, 'data, T>;
    fn index(&self, index: D) -> &Self::Output {
        unsafe {
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(index).unwrap();
            jl_array_data(self.array.unwrap(Private).cast())
                .cast::<Ref<T>>()
                .add(idx)
                .as_ref()
                .unwrap()
        }
    }
}
