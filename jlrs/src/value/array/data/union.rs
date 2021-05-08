use super::super::{Array, Dimensions};
use crate::{
    error::{JlrsError, JlrsResult},
    layout::valid_layout::ValidLayout,
    memory::traits::frame::Frame,
    value::{
        datatype::DataType,
        union::{find_union_component, nth_union_component},
        Value,
    },
};
use jl_sys::jl_array_typetagdata;
use std::marker::PhantomData;

/// Immutably borrowed array data from Julia where the element type is a bits-union. The data has
/// a column-major order and can be indexed with anything that implements `Into<Dimensions>`; see
/// [`Dimensions`] for more information.
#[derive(Clone, Debug)]
pub struct UnionArrayData<'borrow, 'array: 'borrow + 'frame, 'frame, F>
where
    F: Frame<'frame>,
{
    array: Array<'array, 'static>,
    dims: Dimensions,
    _notsendsync: PhantomData<*const ()>,
    _borrow: PhantomData<&'borrow F>,
    _frame: PhantomData<&'frame ()>,
}

impl<'borrow, 'array: 'borrow + 'frame, 'frame, F: Frame<'frame>>
    UnionArrayData<'borrow, 'array, 'frame, F>
{
    pub(crate) fn new(array: Array<'array, 'static>, dims: Dimensions, _: &'borrow F) -> Self {
        UnionArrayData {
            array,
            dims,
            _notsendsync: PhantomData,
            _borrow: PhantomData,
            _frame: PhantomData,
        }
    }

    /// Returns the dimensions of this array.
    pub fn dimensions(&self) -> &Dimensions {
        &self.dims
    }

    /// Returns `true` if `ty` if a value of that type can be stored in this array.
    pub fn contains(&self, ty: DataType) -> bool {
        let mut tag = 0;
        find_union_component(self.array.element_type(), ty.as_value(), &mut tag)
    }

    /// Returns the type of the element at index `idx`.
    pub fn element_type<D: Into<Dimensions>>(
        &self,
        idx: D,
    ) -> JlrsResult<Option<Value<'array, 'static>>> {
        unsafe {
            let elty = self.array.element_type();
            let idx = self.dims.index_of(idx)?;

            let tags = jl_array_typetagdata(self.array.inner().as_ptr());
            let mut tag = *tags.add(idx) as _;

            Ok(nth_union_component(elty, &mut tag))
        }
    }

    /// Get the element at index `idx`. The type `T` must be a valid layout for the type of the
    /// element stored there.
    pub fn get<T: ValidLayout + Copy, D: Into<Dimensions>>(&self, idx: D) -> JlrsResult<T> {
        unsafe {
            let elty = self.array.element_type();
            let idx = self.dims.index_of(idx)?;

            let tags = jl_array_typetagdata(self.array.inner().as_ptr());
            let mut tag = *tags.add(idx) as _;

            if let Some(ty) = nth_union_component(elty, &mut tag) {
                if T::valid_layout(ty) {
                    let offset = idx * self.array.inner().as_ref().elsize as usize;
                    let ptr = self
                        .array
                        .inner()
                        .as_ref()
                        .data
                        .cast::<i8>()
                        .add(offset)
                        .cast::<T>();
                    return Ok(*ptr);
                }
            }

            Err(JlrsError::WrongType)?
        }
    }
}

/// Mutably borrowed array data from Julia where the element type is a bits-union. The data has a
/// column-major order and can be indexed with anything that implements `Into<Dimensions>`; see
/// [`Dimensions`] for more information.
#[derive(Debug)]
pub struct UnionArrayDataMut<'borrow, 'array: 'borrow + 'frame, 'frame, F>
where
    F: Frame<'frame>,
{
    array: Array<'array, 'static>,
    dims: Dimensions,
    _notsendsync: PhantomData<*const ()>,
    _borrow: PhantomData<&'borrow mut F>,
    _f: PhantomData<&'frame ()>,
}

impl<'borrow, 'array: 'borrow + 'frame, 'frame, F: Frame<'frame>>
    UnionArrayDataMut<'borrow, 'array, 'frame, F>
{
    pub(crate) fn new(array: Array<'array, 'static>, dims: Dimensions, _: &'borrow mut F) -> Self {
        UnionArrayDataMut {
            array,
            dims,
            _notsendsync: PhantomData,
            _borrow: PhantomData,
            _f: PhantomData,
        }
    }

    /// Returns the dimensions of this array.
    pub fn dimensions(&self) -> &Dimensions {
        &self.dims
    }

    /// Returns `true` if `ty` if a value of that type can be stored in this array.
    pub fn contains(&self, ty: DataType) -> bool {
        let mut tag = 0;
        find_union_component(self.array.element_type(), ty.as_value(), &mut tag)
    }

    /// Returns the type of the element at index `idx`.
    pub fn element_type<D: Into<Dimensions>>(
        &self,
        idx: D,
    ) -> JlrsResult<Option<Value<'array, 'static>>> {
        unsafe {
            let elty = self.array.element_type();
            let idx = self.dims.index_of(idx)?;

            let tags = jl_array_typetagdata(self.array.inner().as_ptr());
            let mut tag = *tags.add(idx) as _;

            Ok(nth_union_component(elty, &mut tag))
        }
    }

    /// Get the element at index `idx`. The type `T` must be a valid layout for the type of the
    /// element stored there.
    pub fn get<T: ValidLayout + Copy, D: Into<Dimensions>>(&self, idx: D) -> JlrsResult<T> {
        unsafe {
            let elty = self.array.element_type();
            let idx = self.dims.index_of(idx)?;

            let tags = jl_array_typetagdata(self.array.inner().as_ptr());
            let mut tag = *tags.add(idx) as _;

            if let Some(ty) = nth_union_component(elty, &mut tag) {
                if T::valid_layout(ty) {
                    let offset = idx * self.array.inner().as_ref().elsize as usize;
                    let ptr = self
                        .array
                        .inner()
                        .as_ref()
                        .data
                        .cast::<i8>()
                        .add(offset)
                        .cast::<T>();
                    return Ok(*ptr);
                }
            }

            Err(JlrsError::WrongType)?
        }
    }

    /// Set the element at index `idx` to `value` with the type `ty`. The type `T` must be a valid
    /// layout for the value, and `ty` must be a member of the union of all possible element
    /// types.
    pub fn set<T: ValidLayout + Copy, D: Into<Dimensions>>(
        &mut self,
        idx: D,
        ty: DataType,
        value: T,
    ) -> JlrsResult<()> {
        unsafe {
            if !T::valid_layout(ty.as_value()) {
                Err(JlrsError::InvalidLayout)?;
            }

            let mut tag = 0;
            if !find_union_component(self.array.element_type(), ty.as_value(), &mut tag) {
                Err(JlrsError::InvalidArrayType)?;
            }

            let idx = self.dims.index_of(idx)?;
            let offset = idx * self.array.inner().as_ref().elsize as usize;
            self.array
                .inner()
                .as_ref()
                .data
                .cast::<i8>()
                .add(offset)
                .cast::<T>()
                .write(value);

            jl_array_typetagdata(self.array.inner().as_ptr())
                .add(idx)
                .write(tag as _);
        }

        Ok(())
    }
}
