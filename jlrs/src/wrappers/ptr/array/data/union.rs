//! Bits-union array data borrowed from Julia.

use crate::{
    error::{JlrsError, JlrsResult, CANNOT_DISPLAY_TYPE},
    layout::valid_layout::ValidLayout,
    memory::frame::Frame,
    private::Private,
    wrappers::ptr::{
        array::{
            dimensions::{ArrayDimensions, Dims},
            Array,
        },
        datatype::DataType,
        private::WrapperPriv,
        union::{find_union_component, nth_union_component},
        value::Value,
        Wrapper as _,
    },
};
use jl_sys::jl_array_typetagdata;
use std::marker::PhantomData;

/// Immutably borrowed array data from Julia where the element type is a bits-union.
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct UnionArrayData<'borrow, 'array> {
    array: Array<'array, 'static>,
    _marker: PhantomData<&'borrow ()>,
}

impl<'borrow, 'array> UnionArrayData<'borrow, 'array> {
    // Safety: The array must contain bits unions
    pub(crate) unsafe fn new<'frame, F>(array: Array<'array, 'static>, _: &'borrow F) -> Self
    where
        F: Frame<'frame>,
    {
        UnionArrayData {
            array,
            _marker: PhantomData,
        }
    }

    /// Returns the dimensions of this array.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        ArrayDimensions::new(self.array)
    }

    /// Returns `true` if `ty` if a value of that type can be stored in this array.
    pub fn contains(&self, ty: DataType) -> bool {
        let mut tag = 0;
        find_union_component(self.array.element_type(), ty.as_value(), &mut tag)
    }

    /// Returns the type of the element at index `idx`.
    pub fn element_type<D>(&self, index: D) -> JlrsResult<Option<Value<'array, 'static>>>
    where
        D: Dims,
    {
        unsafe {
            let elty = self.array.element_type();
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index)?;

            let tags = jl_array_typetagdata(self.array.unwrap(Private));
            let mut tag = *tags.add(idx) as _;

            Ok(nth_union_component(elty, &mut tag))
        }
    }

    /// Get the element at index `idx`. The type `T` must be a valid layout for the type of the
    /// element stored there.
    pub fn get<T, D>(&self, index: D) -> JlrsResult<T>
    where
        T: ValidLayout + Clone,
        D: Dims,
    {
        unsafe {
            let elty = self.array.element_type();
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index)?;

            let tags = jl_array_typetagdata(self.array.unwrap(Private));
            let mut tag = *tags.add(idx) as _;

            if let Some(ty) = nth_union_component(elty, &mut tag) {
                if T::valid_layout(ty) {
                    let offset = idx * self.array.unwrap_non_null(Private).as_ref().elsize as usize;
                    let ptr = self.array.data_ptr().cast::<i8>().add(offset).cast::<T>();
                    return Ok((&*ptr).clone());
                }

                Err(JlrsError::WrongType {
                    value_type: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                })?
            }

            Err(JlrsError::IllegalUnionTag {
                union_type: elty.display_string_or(CANNOT_DISPLAY_TYPE),
                tag: tag as usize,
            })?
        }
    }
}

/// Mutably borrowed array data from Julia where the element type is a bits-union.
#[derive(Debug)]
#[repr(transparent)]
pub struct UnionArrayDataMut<'borrow, 'array> {
    array: Array<'array, 'static>,
    _marker: PhantomData<&'borrow mut ()>,
}

impl<'borrow, 'array> UnionArrayDataMut<'borrow, 'array> {
    // Safety: The array must contain bits unions
    pub(crate) unsafe fn new<'frame, F>(array: Array<'array, 'static>, _: &'borrow mut F) -> Self
    where
        F: Frame<'frame>,
    {
        UnionArrayDataMut {
            array,
            _marker: PhantomData,
        }
    }

    /// Returns the dimensions of this array.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        ArrayDimensions::new(self.array)
    }

    /// Returns `true` if `ty` if a value of that type can be stored in this array.
    pub fn contains(&self, ty: DataType) -> bool {
        let mut tag = 0;
        find_union_component(self.array.element_type(), ty.as_value(), &mut tag)
    }

    /// Returns the type of the element at index `idx`.
    pub fn element_type<D>(&self, index: D) -> JlrsResult<Option<Value<'array, 'static>>>
    where
        D: Dims,
    {
        unsafe {
            let elty = self.array.element_type();
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index)?;

            let tags = jl_array_typetagdata(self.array.unwrap(Private));
            let mut tag = *tags.add(idx) as _;

            Ok(nth_union_component(elty, &mut tag))
        }
    }

    /// Get the element at index `idx`. The type `T` must be a valid layout for the type of the
    /// element stored there.
    pub fn get<T, D>(&self, index: D) -> JlrsResult<T>
    where
        T: ValidLayout + Clone,
        D: Dims,
    {
        unsafe {
            let elty = self.array.element_type();
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index)?;

            let tags = jl_array_typetagdata(self.array.unwrap(Private));
            let mut tag = *tags.add(idx) as _;

            if let Some(ty) = nth_union_component(elty, &mut tag) {
                if T::valid_layout(ty) {
                    let offset = idx * self.array.unwrap_non_null(Private).as_ref().elsize as usize;
                    let ptr = self.array.data_ptr().cast::<i8>().add(offset).cast::<T>();
                    return Ok((&*ptr).clone());
                }
                Err(JlrsError::WrongType {
                    value_type: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                })?
            }

            Err(JlrsError::IllegalUnionTag {
                union_type: elty.display_string_or(CANNOT_DISPLAY_TYPE),
                tag: tag as usize,
            })?
        }
    }

    /// Set the element at index `idx` to `value` with the type `ty`. The type `T` must be a valid
    /// layout for the value, and `ty` must be a member of the union of all possible element
    /// types.
    pub fn set<T, D>(&mut self, index: D, ty: DataType, value: T) -> JlrsResult<()>
    where
        T: ValidLayout + Clone,
        D: Dims,
    {
        unsafe {
            if !T::valid_layout(ty.as_value()) {
                let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
                Err(JlrsError::InvalidLayout { value_type_str })?;
            }

            let mut tag = 0;
            if !find_union_component(self.array.element_type(), ty.as_value(), &mut tag) {
                let element_type_str = self
                    .array
                    .element_type()
                    .display_string_or(CANNOT_DISPLAY_TYPE);
                let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE);
                Err(JlrsError::ElementTypeError {
                    element_type_str,
                    value_type_str,
                })?;
            }

            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index)?;
            let offset = idx * self.array.unwrap_non_null(Private).as_ref().elsize as usize;
            self.array
                .data_ptr()
                .cast::<i8>()
                .add(offset)
                .cast::<T>()
                .write(value);

            jl_array_typetagdata(self.array.unwrap(Private))
                .add(idx)
                .write(tag as _);
        }

        Ok(())
    }
}

/// Mutably borrowed array data from Julia where the element type is a bits-union.
#[derive(Debug)]
#[repr(transparent)]
pub struct UnresistrictedUnionArrayDataMut<'borrow, 'array> {
    array: Array<'array, 'static>,
    _marker: PhantomData<&'borrow mut ()>,
}

impl<'borrow, 'array> UnresistrictedUnionArrayDataMut<'borrow, 'array> {
    // Safety: The array must contain bits unions
    pub(crate) unsafe fn new<'frame, F>(array: Array<'array, 'static>, _: &'borrow F) -> Self
    where
        F: Frame<'frame>,
    {
        UnresistrictedUnionArrayDataMut {
            array,
            _marker: PhantomData,
        }
    }

    /// Returns the dimensions of this array.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        ArrayDimensions::new(self.array)
    }

    /// Returns `true` if `ty` if a value of that type can be stored in this array.
    pub fn contains(&self, ty: DataType) -> bool {
        let mut tag = 0;
        find_union_component(self.array.element_type(), ty.as_value(), &mut tag)
    }

    /// Returns the type of the element at index `idx`.
    pub fn element_type<D>(&self, index: D) -> JlrsResult<Option<Value<'array, 'static>>>
    where
        D: Dims,
    {
        unsafe {
            let elty = self.array.element_type();
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index)?;

            let tags = jl_array_typetagdata(self.array.unwrap(Private));
            let mut tag = *tags.add(idx) as _;

            Ok(nth_union_component(elty, &mut tag))
        }
    }

    /// Get the element at index `idx`. The type `T` must be a valid layout for the type of the
    /// element stored there.
    pub fn get<T, D>(&self, index: D) -> JlrsResult<T>
    where
        T: ValidLayout + Clone,
        D: Dims,
    {
        unsafe {
            let elty = self.array.element_type();
            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index)?;

            let tags = jl_array_typetagdata(self.array.unwrap(Private));
            let mut tag = *tags.add(idx) as _;

            if let Some(ty) = nth_union_component(elty, &mut tag) {
                if T::valid_layout(ty) {
                    let offset = idx * self.array.unwrap_non_null(Private).as_ref().elsize as usize;
                    let ptr = self.array.data_ptr().cast::<i8>().add(offset).cast::<T>();
                    return Ok((&*ptr).clone());
                }
                Err(JlrsError::WrongType {
                    value_type: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                })?
            }

            Err(JlrsError::IllegalUnionTag {
                union_type: elty.display_string_or(CANNOT_DISPLAY_TYPE),
                tag: tag as usize,
            })?
        }
    }

    /// Set the element at index `idx` to `value` with the type `ty`. The type `T` must be a valid
    /// layout for the value, and `ty` must be a member of the union of all possible element
    /// types.
    pub fn set<T, D>(&mut self, index: D, ty: DataType, value: T) -> JlrsResult<()>
    where
        T: ValidLayout + Clone,
        D: Dims,
    {
        unsafe {
            if !T::valid_layout(ty.as_value()) {
                let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
                Err(JlrsError::InvalidLayout { value_type_str })?;
            }

            let mut tag = 0;
            if !find_union_component(self.array.element_type(), ty.as_value(), &mut tag) {
                let element_type_str = self
                    .array
                    .element_type()
                    .display_string_or(CANNOT_DISPLAY_TYPE);
                let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE);
                Err(JlrsError::ElementTypeError {
                    element_type_str,
                    value_type_str,
                })?;
            }

            let dims = ArrayDimensions::new(self.array);
            let idx = dims.index_of(&index)?;
            let offset = idx * self.array.unwrap_non_null(Private).as_ref().elsize as usize;
            self.array
                .data_ptr()
                .cast::<i8>()
                .add(offset)
                .cast::<T>()
                .write(value);

            jl_array_typetagdata(self.array.unwrap(Private))
                .add(idx)
                .write(tag as _);
        }

        Ok(())
    }
}
