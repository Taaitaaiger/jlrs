/// Access the data in an `Array`.
///
/// This module provides several traits and types that allow accessing the data in an array.
/// Unlike other parts of jlrs, methods that mutate the array are safe: guaranteeing that it's
/// safe to access and/or mutate an array is considered part of the safety-contract of the methods
/// that create a mutable accessor.
///
/// While the [`ArrayBase`] type only cares about the element type, accessors care about the
/// layout of that type.
///
/// All accessors implement the [`Accessor`] trait, all mutable accessors implement
/// [`AccessorMut`], and all mutable accessors of rank 1 implement [`AccessorMut1D`]. These traits
/// let you access and mutate the content of the array as `Value`s. Only [`AccessorMut1D`] allows
/// growing or shrinking the `Array`.
///
/// See the documentation of the [`array`] module for more information about the layout of an
/// array and choosing the right accessor.
use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, Index, IndexMut},
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

use jl_sys::{
    inlined::jlrs_array_data_fast, jl_array_del_end, jl_array_grow_end, jl_array_ptr_1d_append,
    jl_array_ptr_1d_push, jl_value_t, jlrs_array_typetagdata, jlrs_arrayref, jlrs_arrayset,
};

use super::copied::CopiedArray;
use crate::{
    catch::{catch_exceptions, unwrap_exc},
    data::{
        layout::{
            is_bits::IsBits,
            typed_layout::{HasLayout, TypedLayout},
            valid_layout::ValidField,
        },
        managed::{
            array::{
                dimensions::{Dims, DimsRankAssert, DimsRankCheck},
                ArrayBase,
            },
            private::ManagedPriv,
            union::{find_union_component, nth_union_component},
            Ref,
        },
        types::construct_type::ConstructType,
    },
    error::{AccessError, JlrsError, TypeError, CANNOT_DISPLAY_TYPE},
    memory::target::{unrooted::Unrooted, TargetException},
    prelude::{
        DataType, JlrsResult, Managed, Target, Value, ValueData, ValueRef, ValueResult, VectorAny,
    },
    private::Private,
};

/// Functionality supported by all accessors.
pub trait Accessor<'scope, 'data, T, const N: isize> {
    /// Returns the backing array.
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N>;

    /// Converts the element at `index` to a `Value` and returns it.
    ///
    /// If `index` is not in-bounds, `None` is returned.
    fn get_value<'target, D: Dims, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        index: D,
    ) -> Option<ValueResult<'target, 'data, Tgt>> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let idx = self.array().dimensions().index_of(&index)?;

        unsafe {
            let callback = || jlrs_arrayref(self.array().unwrap(Private), idx);

            let res = match catch_exceptions(callback, unwrap_exc) {
                Ok(v) => Ok(NonNull::new_unchecked(v)),
                Err(e) => Err(e),
            };

            Some(target.result_from_ptr(res, Private))
        }
    }

    /// Converts the element at `index` to a `Value` and returns it.
    ///
    /// Safety: `index` must be in-bounds.
    #[inline]
    unsafe fn get_value_unchecked<'target, D: Dims, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        index: D,
    ) -> ValueData<'target, 'data, Tgt> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        unsafe {
            let idx = self.array().dimensions().index_of_unchecked(&index);
            let v = jlrs_arrayref(self.array().unwrap(Private), idx);
            ValueRef::wrap(NonNull::new_unchecked(v)).root(target)
        }
    }
}

/// Functionality supported by all mutable accessors.
pub trait AccessorMut<'scope, 'data, T, const N: isize>: Accessor<'scope, 'data, T, N> {
    /// Sets the element at `index` to `value`.
    ///
    /// If the `DataType` of `value` is not a valid type for an element of this array,
    /// an exception is thrown which is caught and returned. If the index is not in-bounds,
    /// `None` is returned.
    fn set_value<'target, 'value, D: Dims, Tgt: Target<'target>>(
        &mut self,
        target: Tgt,
        index: D,
        value: Value<'value, 'data>,
    ) -> Result<TargetException<'target, 'data, (), Tgt>, Value<'value, 'data>> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let Some(idx) = self.array().dimensions().index_of(&index) else {
            return Err(value);
        };

        unsafe {
            let callback = || {
                jlrs_arrayset(self.array().unwrap(Private), value.unwrap(Private), idx);
            };

            match catch_exceptions(callback, unwrap_exc) {
                Ok(_) => Ok(Ok(())),
                Err(e) => Ok(Err(ValueRef::wrap(e).root(target))),
            }
        }
    }

    /// Sets the element at `index` to `value`.
    ///
    /// Safety: `index` must be in-bounds. If the `DataType` of `value` is not a valid type for an
    /// element of this array, an exception is thrown which is not caught.
    #[inline]
    unsafe fn set_value_unchecked<D: Dims>(&mut self, index: D, value: Value<'_, 'data>) {
        let idx = self.array().dimensions().index_of_unchecked(&index);
        jlrs_arrayset(self.array().unwrap(Private), value.unwrap(Private), idx);
    }
}

/// Functionality supported by all mutable accessors for 1D arrays.
pub trait AccessorMut1D<'scope, 'data, T>: AccessorMut<'scope, 'data, T, 1> {
    /// Inserts `inc` elements at the end of the array. If an exception is thrown, it's caught and
    /// returned.
    fn grow_end<'target, Tgt>(
        &mut self,
        target: Tgt,
        inc: usize,
    ) -> TargetException<'target, 'static, (), Tgt>
    where
        Tgt: Target<'target>,
    {
        // Safety: the C API function is called with valid data. If an exception is thrown it's
        // caught.
        unsafe {
            let callback = || self.grow_end_unchecked(inc);

            let res = match catch_exceptions(callback, unwrap_exc) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            };

            target.exception_from_ptr(res, Private)
        }
    }

    /// Inserts `inc` elements at the end of the array. If an exception is thrown, it's not
    /// caught.
    #[inline]
    unsafe fn grow_end_unchecked(&mut self, inc: usize) {
        jl_array_grow_end(self.array().unwrap(Private), inc);
    }
    /// Removes `dec` elements from the end of the array. If an exception is thrown, it's caught
    /// and returned.
    fn del_end<'target, Tgt>(
        &mut self,
        target: Tgt,
        dec: usize,
    ) -> TargetException<'target, 'static, (), Tgt>
    where
        Tgt: Target<'target>,
    {
        // Safety: the C API function is called with valid data. If an exception is thrown it's
        // caught.
        unsafe {
            let callback = || self.del_end_unchecked(dec);

            let res = match catch_exceptions(callback, unwrap_exc) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            };

            target.exception_from_ptr(res, Private)
        }
    }

    /// Removes `dec` elements from the end of the array. If an exception is thrown, it's not
    /// caught.
    #[inline]
    unsafe fn del_end_unchecked(&mut self, dec: usize) {
        jl_array_del_end(self.array().unwrap(Private), dec);
    }
}

impl<'scope, 'data, A, T> AccessorMut1D<'scope, 'data, T> for A where
    A: AccessorMut<'scope, 'data, T, 1>
{
}

/// An accessor for `isbits` data.
#[repr(transparent)]
pub struct BitsAccessor<'borrow, 'scope, 'data, T, L, const N: isize> {
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow [L]>,
}

impl<'borrow, 'scope, 'data, T, L, const N: isize> BitsAccessor<'borrow, 'scope, 'data, T, L, N>
where
    L: ValidField + IsBits,
{
    #[inline]
    pub(crate) unsafe fn new(array: &'borrow ArrayBase<'scope, 'data, T, N>) -> Self {
        BitsAccessor {
            array: *array,
            _data: PhantomData,
        }
    }

    /// Returns a reference the element at `index` if `index` is in-bounds`, `None` otherwise.
    pub fn get<D: Dims>(&self, index: D) -> Option<&L> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index)?;

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<L>()
                .add(idx);

            Some(&*elem)
        }
    }

    /// Returns a reference to the element at `index`.
    ///
    /// Safety: `index` must be in-bounds.
    pub unsafe fn get_unchecked<D: Dims>(&self, index: D) -> &L {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let idx = self.array.dimensions().index_of_unchecked(&index);

        let elem = jlrs_array_data_fast(self.array.unwrap(Private))
            .cast::<L>()
            .add(idx);

        &*elem
    }

    /// Temporarily converts this accessor to a slice.
    pub fn as_slice<'a>(&'a self) -> &'a [L] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<L>();
            std::slice::from_raw_parts(ptr, sz)
        }
    }

    /// Converts this accessor into a slice.
    pub fn into_slice(self) -> &'borrow [L] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<L>();
            std::slice::from_raw_parts(ptr, sz)
        }
    }

    /// Copies the content of this array to a `CopiedArray`.
    pub fn to_copied_array(&self) -> CopiedArray<L>
    where
        L: Copy,
    {
        unsafe {
            let data = self.as_slice().into();
            let dims = self.array.dimensions().to_dimensions();
            CopiedArray::new(data, dims)
        }
    }

    /// Returns a reference the element at `index` if `index` is in-bounds`, `None` otherwise.
    pub fn get_uninit<D: Dims>(&self, index: D) -> Option<&MaybeUninit<L>> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index)?;

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<MaybeUninit<L>>()
                .add(idx);

            Some(&*elem)
        }
    }

    /// Returns a reference to the element at `index`.
    ///
    /// Safety: `index` must be in-bounds.
    pub unsafe fn get_uninit_unchecked<D: Dims>(&self, index: D) -> &MaybeUninit<L> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let idx = self.array.dimensions().index_of_unchecked(&index);

        let elem = jlrs_array_data_fast(self.array.unwrap(Private))
            .cast::<MaybeUninit<L>>()
            .add(idx);

        &*elem
    }

    /// Temporarily converts this accessor to a slice.
    pub fn as_uninit_slice<'a>(&'a self) -> &'a [MaybeUninit<L>] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<MaybeUninit<L>>();
            std::slice::from_raw_parts(ptr, sz)
        }
    }

    /// Converts this accessor into a slice.
    pub fn into_uninit_slice(self) -> &'borrow [MaybeUninit<L>] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<MaybeUninit<L>>();
            std::slice::from_raw_parts(ptr, sz)
        }
    }

    /// Copies the content of this array to a `CopiedArray` without assuming the content has
    /// been fully initialized.
    pub fn to_maybe_uninit_copied_array(&self) -> CopiedArray<MaybeUninit<L>>
    where
        L: Copy,
    {
        unsafe {
            let s = self.as_uninit_slice();
            let mut v = Vec::with_capacity(s.len());

            v.as_mut_slice().copy_from_slice(s);

            let data = v.into_boxed_slice();
            let dims = self.array.dimensions().to_dimensions();
            CopiedArray::new(data, dims)
        }
    }
}

impl<'borrow, 'scope, 'data, T, L, D: Dims, const N: isize> Index<D>
    for BitsAccessor<'borrow, 'scope, 'data, T, L, N>
{
    type Output = L;

    fn index(&self, index: D) -> &Self::Output {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index).unwrap();

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<L>()
                .add(idx);

            &*elem
        }
    }
}

impl<'scope, 'data, T, L, const N: isize> Accessor<'scope, 'data, T, N>
    for BitsAccessor<'_, 'scope, 'data, T, L, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

/// A mutable accessor for `isbits` data.
#[repr(transparent)]
pub struct BitsAccessorMut<'borrow, 'scope, 'data, T, L, const N: isize> {
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow mut [L]>,
}

impl<'borrow, 'scope, 'data, T, L, const N: isize> BitsAccessorMut<'borrow, 'scope, 'data, T, L, N>
where
    L: ValidField + IsBits,
{
    pub(crate) unsafe fn new(array: &'borrow mut ArrayBase<'scope, 'data, T, N>) -> Self {
        BitsAccessorMut {
            array: *array,
            _data: PhantomData,
        }
    }

    /// Returns a mutable reference to the element at `index` if it is in-bounds, `None`
    /// otherwise.
    pub fn get_mut<D: Dims>(&mut self, index: D) -> Option<&mut L> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index)?;

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<L>()
                .add(idx);

            Some(&mut *elem)
        }
    }

    /// Returns a mutable reference to the element at `index`.
    ///
    /// Safety: `index` must be in-bounds.
    pub unsafe fn get_mut_unchecked<D: Dims>(&mut self, index: D) -> &mut L {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let idx = self.array.dimensions().index_of_unchecked(&index);

        let elem = jlrs_array_data_fast(self.array.unwrap(Private))
            .cast::<L>()
            .add(idx);

        &mut *elem
    }

    /// Returns a mutable reference to the element at `index` if it is in-bounds, `None`
    /// otherwise.
    pub fn get_mut_uninit<D: Dims>(&mut self, index: D) -> Option<&mut MaybeUninit<L>> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index)?;

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<MaybeUninit<L>>()
                .add(idx);

            Some(&mut *elem)
        }
    }

    /// Returns a mutable reference to the element at `index`.
    ///
    /// Safety: `index` must be in-bounds.
    pub unsafe fn get_mut_uninit_unchecked<D: Dims>(&mut self, index: D) -> &mut MaybeUninit<L> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let idx = self.array.dimensions().index_of_unchecked(&index);

        let elem = jlrs_array_data_fast(self.array.unwrap(Private))
            .cast::<MaybeUninit<L>>()
            .add(idx);

        &mut *elem
    }

    /// Sets the elements at `index` to `value` if `index` is in-bounds.
    ///
    /// If `index` is not in-bounds `Err(value)` is retuned, if it is in-bounds `Ok(())` is
    /// returned.
    pub fn set<D: Dims>(&mut self, index: D, value: L) -> Result<(), L> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index);
        match idx {
            None => Err(value),
            Some(idx) => unsafe {
                let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                    .cast::<L>()
                    .add(idx);

                elem.write(value);
                Ok(())
            },
        }
    }

    /// Sets the elements at `index` to `value`.
    ///
    /// Safety: `index` must be in-bounds.
    pub unsafe fn set_unchecked<D: Dims>(&mut self, index: D, value: L) {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let idx = self.array.dimensions().index_of_unchecked(&index);

        let elem = jlrs_array_data_fast(self.array.unwrap(Private))
            .cast::<L>()
            .add(idx);

        elem.write(value);
    }

    /// Temporarily converts this accessor to a mutable slice.
    pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [L] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<L>();
            std::slice::from_raw_parts_mut(ptr, sz)
        }
    }

    /// Converts this accessor into a mutable slice.
    pub fn into_mut_slice(self) -> &'borrow mut [L] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<L>();
            std::slice::from_raw_parts_mut(ptr, sz)
        }
    }

    /// Temporarily converts this accessor to a mutable slice.
    pub fn as_mut_uninit_slice<'a>(&'a mut self) -> &'a mut [MaybeUninit<L>] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<MaybeUninit<L>>();
            std::slice::from_raw_parts_mut(ptr, sz)
        }
    }

    /// Converts this accessor into a mutable slice.
    pub fn into_mut_uninit_slice(self) -> &'borrow mut [MaybeUninit<L>] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<MaybeUninit<L>>();
            std::slice::from_raw_parts_mut(ptr, sz)
        }
    }
}

impl<'borrow, 'scope, 'data, T, L> BitsAccessorMut<'borrow, 'scope, 'data, T, L, 1>
where
    L: ValidField + IsBits,
{
    /// Pushes the new element `data` to the end of this `Vector`. If an exception is thrown, it's
    /// caught and returned.
    pub fn push<'target, Tgt>(
        &mut self,
        target: Tgt,
        data: L,
    ) -> TargetException<'target, 'static, (), Tgt>
    where
        Tgt: Target<'target>,
    {
        let length = self.array.length();
        self.grow_end(target, 1)?;
        unsafe {
            self.get_mut_uninit_unchecked(length).write(data);
        }

        Ok(())
    }

    /// Pushes the new element `data` to the end of this `Vector`. If an exception is thrown, it's
    /// not caught.
    pub unsafe fn push_unchecked(&mut self, data: L) {
        let length = self.array.length();
        self.grow_end_unchecked(1);
        self.get_mut_uninit_unchecked(length).write(data);
    }

    /// Pushes all elements from `data` to the end of this `Vector`. If an exception is thrown,
    /// it's caught and returned.
    pub fn extend_from_slice<'target, Tgt>(
        &mut self,
        target: Tgt,
        data: &[L],
    ) -> TargetException<'target, 'static, (), Tgt>
    where
        L: Copy,
        Tgt: Target<'target>,
    {
        let length = self.array.length();
        let n = data.len();
        self.grow_end(target, n)?;

        unsafe {
            let slice = &mut self.as_mut_uninit_slice()[length..];
            let data_ptr = data.as_ptr().cast::<MaybeUninit<L>>();
            let data = std::slice::from_raw_parts(data_ptr, n);

            slice.copy_from_slice(data);
        }

        Ok(())
    }

    /// Pushes all elements from `data` to the end of this `Vector`. If an exception is thrown,
    /// it's not caught.
    pub unsafe fn extend_from_slice_unchecked(&mut self, data: &[L])
    where
        L: Copy,
    {
        let length = self.array.length();
        let n = data.len();
        self.grow_end_unchecked(n);
        let slice = &mut self.as_mut_uninit_slice()[length..];
        let data_ptr = data.as_ptr().cast::<MaybeUninit<L>>();
        let data = std::slice::from_raw_parts(data_ptr, n);

        slice.copy_from_slice(data);
    }
}

impl<'borrow, 'scope, 'data, T, L, const N: isize> Deref
    for BitsAccessorMut<'borrow, 'scope, 'data, T, L, N>
{
    type Target = BitsAccessor<'borrow, 'scope, 'data, T, L, N>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl<'borrow, 'scope, 'data, T, L, D: Dims, const N: isize> Index<D>
    for BitsAccessorMut<'borrow, 'scope, 'data, T, L, N>
{
    type Output = L;

    fn index(&self, index: D) -> &Self::Output {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index).unwrap();

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<L>()
                .add(idx);

            &*elem
        }
    }
}

impl<'borrow, 'scope, 'data, T, L, D: Dims, const N: isize> IndexMut<D>
    for BitsAccessorMut<'borrow, 'scope, 'data, T, L, N>
{
    fn index_mut(&mut self, index: D) -> &mut Self::Output {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index).unwrap();

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<L>()
                .add(idx);

            &mut *elem
        }
    }
}

impl<'scope, 'data, T, L, const N: isize> Accessor<'scope, 'data, T, N>
    for BitsAccessorMut<'_, 'scope, 'data, T, L, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

impl<'scope, 'data, T, L, const N: isize> AccessorMut<'scope, 'data, T, N>
    for BitsAccessorMut<'_, 'scope, 'data, T, L, N>
{
}

/// An accessor for inline data.
#[repr(transparent)]
pub struct InlineAccessor<'borrow, 'scope, 'data, T, L, const N: isize> {
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow [L]>,
}

impl<'borrow, 'scope, 'data, T, L, const N: isize> InlineAccessor<'borrow, 'scope, 'data, T, L, N>
where
    L: ValidField,
{
    pub(crate) unsafe fn new(array: &'borrow ArrayBase<'scope, 'data, T, N>) -> Self {
        InlineAccessor {
            array: *array,
            _data: PhantomData,
        }
    }

    /// Returns a reference the element at `index` if `index` is in-bounds`, `None` otherwise.
    pub fn get<D: Dims>(&self, index: D) -> Option<&L> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index)?;

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<L>()
                .add(idx);

            Some(&*elem)
        }
    }

    /// Returns a reference the element at `index` if `index` is in-bounds`.
    ///
    /// Safety: `index` must be in-bounds.
    pub unsafe fn get_unchecked<D: Dims>(&self, index: D) -> &L {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let idx = self.array.dimensions().index_of_unchecked(&index);

        let elem = jlrs_array_data_fast(self.array.unwrap(Private))
            .cast::<L>()
            .add(idx);

        &*elem
    }

    /// Returns a reference the element at `index` if `index` is in-bounds`, `None` otherwise.
    pub fn get_uninit<D: Dims>(&self, index: D) -> Option<&MaybeUninit<L>> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index)?;

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<MaybeUninit<L>>()
                .add(idx);

            Some(&*elem)
        }
    }

    /// Returns a reference the element at `index` if `index` is in-bounds`.
    ///
    /// Safety: `index` must be in-bounds.
    pub unsafe fn get_uninit_unchecked<D: Dims>(&self, index: D) -> &MaybeUninit<L> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let idx = self.array.dimensions().index_of_unchecked(&index);

        let elem = jlrs_array_data_fast(self.array.unwrap(Private))
            .cast::<MaybeUninit<L>>()
            .add(idx);

        &*elem
    }

    /// Temporarily converts this accessor to a slice.
    pub fn as_slice<'a>(&'a self) -> &'a [L] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<L>();
            std::slice::from_raw_parts(ptr, sz)
        }
    }

    /// Converts this accessor into a slice.
    pub fn into_slice(self) -> &'borrow [L] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<L>();
            std::slice::from_raw_parts(ptr, sz)
        }
    }

    /// Temporarily converts this accessor to a slice.
    pub fn as_uninit_slice<'a>(&'a self) -> &'a [MaybeUninit<L>] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<MaybeUninit<L>>();
            std::slice::from_raw_parts(ptr, sz)
        }
    }

    /// Converts this accessor into a slice.
    pub fn into_uninit_slice(self) -> &'borrow [MaybeUninit<L>] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast::<MaybeUninit<L>>();
            std::slice::from_raw_parts(ptr, sz)
        }
    }
}

impl<'borrow, 'scope, 'data, T, L, D: Dims, const N: isize> Index<D>
    for InlineAccessor<'borrow, 'scope, 'data, T, L, N>
{
    type Output = L;

    fn index(&self, index: D) -> &Self::Output {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index).unwrap();

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<L>()
                .add(idx);

            &*elem
        }
    }
}

impl<'scope, 'data, T, L, const N: isize> Accessor<'scope, 'data, T, N>
    for InlineAccessor<'_, 'scope, 'data, T, L, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

/// A mutable accessor for inline data.
#[repr(transparent)]
pub struct InlineAccessorMut<'borrow, 'scope, 'data, T, L, const N: isize> {
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow mut [L]>,
}

impl<'borrow, 'scope, 'data, T, L, const N: isize> Deref
    for InlineAccessorMut<'borrow, 'scope, 'data, T, L, N>
{
    type Target = InlineAccessor<'borrow, 'scope, 'data, T, L, N>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl<'borrow, 'scope, 'data, T, L, const N: isize>
    InlineAccessorMut<'borrow, 'scope, 'data, T, L, N>
where
    L: ValidField,
{
    pub(crate) unsafe fn new(array: &'borrow mut ArrayBase<'scope, 'data, T, N>) -> Self {
        InlineAccessorMut {
            array: *array,
            _data: PhantomData,
        }
    }
}

impl<'scope, 'data, T, L, const N: isize> Accessor<'scope, 'data, T, N>
    for InlineAccessorMut<'_, 'scope, 'data, T, L, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

impl<'scope, 'data, T, L, const N: isize> AccessorMut<'scope, 'data, T, N>
    for InlineAccessorMut<'_, 'scope, 'data, T, L, N>
{
}

/// An accessor for value data.
#[repr(transparent)]
pub struct ValueAccessor<'borrow, 'scope, 'data, T, const N: isize> {
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow [Option<ValueRef<'scope, 'data>>]>,
}

/// An atomic reference to a `Value`.
#[repr(transparent)]
pub struct AtomicValueRef<M> {
    ptr: AtomicPtr<jl_value_t>,
    _marker: PhantomData<M>,
}

impl<'scope, 'data, M: Managed<'scope, 'data>> AtomicValueRef<M> {
    pub fn load(&self, order: Ordering) -> Option<Ref<'scope, 'data, M>> {
        let ptr = self.ptr.load(order);
        if ptr.is_null() {
            return None;
        }

        unsafe { Some(Ref::wrap(NonNull::new_unchecked(ptr.cast()))) }
    }
}

impl<'borrow, 'scope, 'data, T, const N: isize> ValueAccessor<'borrow, 'scope, 'data, T, N> {
    pub(crate) unsafe fn new(array: &'borrow ArrayBase<'scope, 'data, T, N>) -> Self {
        ValueAccessor {
            array: *array,
            _data: PhantomData,
        }
    }

    /// Returns the element at `index` if `index` is in-bounds`, `None` otherwise.
    pub fn get<'target, D: Dims, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        index: D,
    ) -> Option<ValueData<'target, 'data, Tgt>> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index)?;

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<AtomicValueRef<Value>>()
                .add(idx);

            if let Some(elem) = (&*elem).load(Ordering::Relaxed) {
                Some(elem.root(target))
            } else {
                None
            }
        }
    }

    /// Returns the element at `index`.
    ///
    /// Safety: `index` must be in-bounds.
    pub unsafe fn get_unchecked<'target, D: Dims, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        index: D,
    ) -> Option<ValueData<'target, 'data, Tgt>> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let idx = self.array.dimensions().index_of_unchecked(&index);

        let elem = jlrs_array_data_fast(self.array.unwrap(Private))
            .cast::<AtomicValueRef<Value>>()
            .add(idx);

        match (&*elem).load(Ordering::Relaxed) {
            Some(elem) => Some(elem.root(target)),
            None => None,
        }
    }

    /// Temporarily converts this accessor to a slice.
    pub fn as_slice<'a>(&'a self) -> &'a [AtomicValueRef<Value<'scope, 'data>>] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast();
            std::slice::from_raw_parts(ptr, sz)
        }
    }

    /// Converts this accessor into a slice.
    pub fn into_slice(self) -> &'borrow [AtomicValueRef<Value<'scope, 'data>>] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast();
            std::slice::from_raw_parts(ptr, sz)
        }
    }
}

impl<'borrow, 'scope, 'data, T, D: Dims, const N: isize> Index<D>
    for ValueAccessor<'borrow, 'scope, 'data, T, N>
{
    type Output = AtomicValueRef<Value<'scope, 'data>>;

    fn index(&self, index: D) -> &Self::Output {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index).unwrap();

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<Self::Output>()
                .add(idx);

            &*elem
        }
    }
}

impl<'scope, 'data, T, const N: isize> Accessor<'scope, 'data, T, N>
    for ValueAccessor<'_, 'scope, 'data, T, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

/// A mutable accessor for value data.
#[repr(transparent)]
pub struct ValueAccessorMut<'borrow, 'scope, 'data, T, const N: isize> {
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow mut [Option<ValueRef<'scope, 'data>>]>,
}

impl<'borrow, 'scope, 'data, T, const N: isize> Deref
    for ValueAccessorMut<'borrow, 'scope, 'data, T, N>
{
    type Target = ValueAccessor<'borrow, 'scope, 'data, T, N>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl<'borrow, 'scope, 'data, T, const N: isize> ValueAccessorMut<'borrow, 'scope, 'data, T, N> {
    pub(crate) unsafe fn new(array: &'borrow mut ArrayBase<'scope, 'data, T, N>) -> Self {
        ValueAccessorMut {
            array: *array,
            _data: PhantomData,
        }
    }
}

impl<'borrow, 'scope, const N: isize>
    ValueAccessorMut<'borrow, 'scope, 'static, Value<'scope, 'static>, N>
{
    /// Push a new value to the end of this vector.
    pub fn push(&mut self, value: Value) {
        unsafe {
            jl_array_ptr_1d_push(self.array.unwrap(Private), value.unwrap(Private));
        }
    }

    /// Append the data from `arr` to the end of this vector.
    pub fn append(&mut self, arr: VectorAny) {
        unsafe {
            jl_array_ptr_1d_append(self.array.unwrap(Private), arr.unwrap(Private));
        }
    }
}

impl<'scope, 'data, T, const N: isize> Accessor<'scope, 'data, T, N>
    for ValueAccessorMut<'_, 'scope, 'data, T, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

impl<'scope, 'data, T, const N: isize> AccessorMut<'scope, 'data, T, N>
    for ValueAccessorMut<'_, 'scope, 'data, T, N>
{
}

/// An accessor for managed data.
#[repr(transparent)]
pub struct ManagedAccessor<'borrow, 'scope, 'data, T, M: Managed<'scope, 'data>, const N: isize> {
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow [Option<Ref<'scope, 'data, M>>]>,
}

impl<'borrow, 'scope, 'data, T, M, const N: isize> ManagedAccessor<'borrow, 'scope, 'data, T, M, N>
where
    M: Managed<'scope, 'data>,
{
    pub(crate) unsafe fn new(array: &'borrow ArrayBase<'scope, 'data, T, N>) -> Self {
        ManagedAccessor {
            array: *array,
            _data: PhantomData,
        }
    }

    /// Returns the element at `index` if `index` is in-bounds`, `None` otherwise.
    pub fn get<'target, D: Dims, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        index: D,
    ) -> Option<Tgt::Data<'data, M::InScope<'target>>> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index)?;

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<AtomicValueRef<M>>()
                .add(idx);

            if let Some(elem) = (&*elem).load(Ordering::Relaxed) {
                Some(elem.root(target))
            } else {
                None
            }
        }
    }

    /// Returns the element at `index`.
    ///
    /// Safety: `index` must be in-bounds.
    pub unsafe fn get_unchecked<'target, D: Dims, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        index: D,
    ) -> Option<Tgt::Data<'data, M::InScope<'target>>> {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let idx = self.array.dimensions().index_of_unchecked(&index);

        let elem = jlrs_array_data_fast(self.array.unwrap(Private))
            .cast::<AtomicValueRef<M>>()
            .add(idx);

        match (&*elem).load(Ordering::Relaxed) {
            Some(elem) => Some(elem.root(target)),
            None => None,
        }
    }

    /// Temporarily converts this accessor to a slice.
    pub fn as_slice<'a>(&'a self) -> &'a [AtomicValueRef<M>] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast();
            std::slice::from_raw_parts(ptr, sz)
        }
    }

    /// Converts this accessor into a slice.
    pub fn into_slice(self) -> &'borrow [AtomicValueRef<M>] {
        unsafe {
            let sz = self.array.dimensions().size();
            let ptr = jlrs_array_data_fast(self.array.unwrap(Private)).cast();
            std::slice::from_raw_parts(ptr, sz)
        }
    }
}

impl<'borrow, 'scope, 'data, T, M: Managed<'scope, 'data>, D: Dims, const N: isize> Index<D>
    for ManagedAccessor<'borrow, 'scope, 'data, T, M, N>
{
    type Output = AtomicValueRef<M>;

    fn index(&self, index: D) -> &Self::Output {
        let array_dims = self.array.dimensions();
        let idx = array_dims.index_of(&index).unwrap();

        unsafe {
            let elem = jlrs_array_data_fast(self.array.unwrap(Private))
                .cast::<Self::Output>()
                .add(idx);

            &*elem
        }
    }
}

impl<'scope, 'data, T, M: Managed<'scope, 'data>, const N: isize> Accessor<'scope, 'data, T, N>
    for ManagedAccessor<'_, 'scope, 'data, T, M, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

/// A mutable accessor for managed data.
#[repr(transparent)]
pub struct ManagedAccessorMut<'borrow, 'scope, 'data, T, M: Managed<'scope, 'data>, const N: isize>
{
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow mut [Option<Ref<'scope, 'data, M>>]>,
}

impl<'borrow, 'scope, 'data, T, M: Managed<'scope, 'data>, const N: isize> Deref
    for ManagedAccessorMut<'borrow, 'scope, 'data, T, M, N>
{
    type Target = ManagedAccessor<'borrow, 'scope, 'data, T, M, N>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl<'borrow, 'scope, 'data, T, M, const N: isize>
    ManagedAccessorMut<'borrow, 'scope, 'data, T, M, N>
where
    M: Managed<'scope, 'data>,
{
    pub(crate) unsafe fn new(array: &'borrow mut ArrayBase<'scope, 'data, T, N>) -> Self {
        ManagedAccessorMut {
            array: *array,
            _data: PhantomData,
        }
    }
}

impl<'scope, 'data, T, M: Managed<'scope, 'data>, const N: isize> Accessor<'scope, 'data, T, N>
    for ManagedAccessorMut<'_, 'scope, 'data, T, M, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

impl<'scope, 'data, T, M: Managed<'scope, 'data>, const N: isize> AccessorMut<'scope, 'data, T, N>
    for ManagedAccessorMut<'_, 'scope, 'data, T, M, N>
{
}

/// An accessor for bits-union data.
#[repr(transparent)]
pub struct BitsUnionAccessor<'borrow, 'scope, 'data, T, const N: isize> {
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow [MaybeUninit<u8>]>,
}

impl<'borrow, 'scope, 'data, T, const N: isize> BitsUnionAccessor<'borrow, 'scope, 'data, T, N> {
    pub(crate) unsafe fn new(array: &'borrow ArrayBase<'scope, 'data, T, N>) -> Self {
        BitsUnionAccessor {
            array: *array,
            _data: PhantomData,
        }
    }

    pub fn get<L, D>(&self, index: D) -> JlrsResult<Option<L>>
    where
        L: ValidField + IsBits,
        D: Dims,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array = self.array;
        let elty = array.element_type();
        let idx = array.dimensions().index_of(&index);
        if let Some(idx) = idx {
            unsafe {
                let tags = jlrs_array_typetagdata(array.unwrap(Private));
                let mut tag = *tags.add(idx) as _;

                if let Some(ty) = nth_union_component(elty, &mut tag) {
                    if L::valid_field(ty) {
                        let offset = idx * array.element_size();
                        let ptr = array.data_ptr().cast::<u8>().add(offset).cast::<L>();
                        return Ok(Some(ptr.read()));
                    }
                    Err(AccessError::InvalidLayout {
                        value_type: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                    })?
                }

                Err(AccessError::IllegalUnionTag {
                    union_type: elty.display_string_or(CANNOT_DISPLAY_TYPE),
                    tag: tag as usize,
                })?
            }
        } else {
            Ok(None)
        }
    }

    pub unsafe fn get_unchecked<L, D>(&self, index: D) -> L
    where
        L: ValidField + IsBits,
        D: Dims,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array = self.array;
        let idx = array.dimensions().index_of_unchecked(&index);
        let offset = idx * array.element_size();
        let ptr = array.data_ptr().cast::<u8>().add(offset).cast::<L>();
        ptr.read()
    }
}

impl<'scope, 'data, T, const N: isize> Accessor<'scope, 'data, T, N>
    for BitsUnionAccessor<'_, 'scope, 'data, T, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

/// A mutable accessor for bits-union data.
#[repr(transparent)]
pub struct BitsUnionAccessorMut<'borrow, 'scope, 'data, T, const N: isize> {
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow mut [MaybeUninit<u8>]>,
}

impl<'borrow, 'scope, 'data, T, const N: isize> Deref
    for BitsUnionAccessorMut<'borrow, 'scope, 'data, T, N>
{
    type Target = BitsUnionAccessor<'borrow, 'scope, 'data, T, N>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl<'scope, 'data, T, const N: isize> Accessor<'scope, 'data, T, N>
    for BitsUnionAccessorMut<'_, 'scope, 'data, T, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

impl<'scope, 'data, T, const N: isize> AccessorMut<'scope, 'data, T, N>
    for BitsUnionAccessorMut<'_, 'scope, 'data, T, N>
{
}

impl<'borrow, 'scope, 'data, T, const N: isize> BitsUnionAccessorMut<'borrow, 'scope, 'data, T, N> {
    pub(crate) unsafe fn new(array: &'borrow mut ArrayBase<'scope, 'data, T, N>) -> Self {
        BitsUnionAccessorMut {
            array: *array,
            _data: PhantomData,
        }
    }

    /// Sets the element at `index` to `value`.
    ///
    /// Returns a nested error. The outer error is returned if the index is out-of-bounds, the
    /// inner error is returned if `L` is not a variant of the element type.
    pub fn set_typed<L, D>(&mut self, index: D, value: L) -> Result<JlrsResult<()>, L>
    where
        L: ConstructType + ValidField + IsBits + HasLayout<'static, 'static, Layout = L>,
        D: Dims,
    {
        self.set_typed_layout(index, TypedLayout::<L, L>::new(value))
            .map_err(TypedLayout::into_layout)
    }

    /// Sets the element at `index` to `value`.
    ///
    /// Returns a nested error. The outer error is returned if the index is out-of-bounds, the
    /// inner error is returned if `L` is not a variant of the element type.
    pub fn set_typed_layout<U, L, D>(
        &mut self,
        index: D,
        value: TypedLayout<L, U>,
    ) -> Result<JlrsResult<()>, TypedLayout<L, U>>
    where
        U: ConstructType + HasLayout<'static, 'static, Layout = L>,
        L: ValidField + IsBits,
        D: Dims,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let ty = unsafe {
            let unrooted = Unrooted::new();
            U::construct_type(unrooted).as_value()
        };

        let mut tag = 0;
        let array = self.array;
        let elty = array.element_type();
        if !find_union_component(elty, ty, &mut tag) {
            let element_type = elty.display_string_or(CANNOT_DISPLAY_TYPE);
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE);
            return Ok(Err(Box::new(JlrsError::TypeError(
                TypeError::IncompatibleType {
                    element_type,
                    value_type,
                },
            ))));
        }

        let idx = array.dimensions().index_of(&index);
        if let Some(idx) = idx {
            // Safety: The data can be stored in this array, the tag is updated accordingly.
            unsafe {
                let offset = idx * self.array.element_size() as usize;
                array
                    .data_ptr()
                    .cast::<u8>()
                    .add(offset)
                    .cast::<L>()
                    .write(value.into_layout());

                jlrs_array_typetagdata(self.array.unwrap(Private))
                    .add(idx)
                    .write(tag as _);
            }

            Ok(Ok(()))
        } else {
            Err(value)
        }
    }

    /// Sets the element at `index` to `value`.
    ///
    /// Safety: `index` must be in-bounds and `U` must be a valid variant of the element type.
    pub unsafe fn set_typed_unchecked<L, D>(&mut self, index: D, value: L)
    where
        L: ConstructType + ValidField + IsBits + HasLayout<'static, 'static, Layout = L>,
        D: Dims,
    {
        self.set_typed_layout_unchecked(index, TypedLayout::<L, L>::new(value))
    }

    /// Sets the element at `index` to `value`.
    ///
    /// Safety: `index` must be in-bounds and `U` must be a valid variant of the element type.
    pub unsafe fn set_typed_layout_unchecked<U, L, D>(&mut self, index: D, value: TypedLayout<L, U>)
    where
        U: ConstructType + HasLayout<'static, 'static, Layout = L>,
        L: ValidField + IsBits,
        D: Dims,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let ty = {
            let unrooted = Unrooted::new();
            U::construct_type(unrooted).as_value()
        };

        debug_assert!(ty.is::<DataType>());
        debug_assert!(ty.cast_unchecked::<DataType>().is_bits());

        let mut tag = 0;
        let elty = self.array.element_type();
        let success = find_union_component(elty, ty, &mut tag);

        debug_assert!(success);

        let idx = self.array.dimensions().index_of_unchecked(&index);
        let offset = idx * self.array.element_size() as usize;
        self.array
            .data_ptr()
            .cast::<u8>()
            .add(offset)
            .cast::<L>()
            .write(value.into_layout());

        jlrs_array_typetagdata(self.array.unwrap(Private))
            .add(idx)
            .write(tag as _);
    }

    /// Sets the element at `index` to `value` using `ty` as the type.
    ///
    /// Returns an error if `ty` is not a valid variant of the element type, returns `None` if
    /// `index` is out-of-bounds.
    pub fn set<L, D>(&mut self, index: D, ty: DataType, value: L) -> Result<JlrsResult<()>, L>
    where
        L: ValidField + IsBits,
        D: Dims,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        if !L::valid_field(ty.as_value()) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            return Ok(Err(Box::new(JlrsError::AccessError(
                AccessError::InvalidLayout { value_type },
            ))));
        }

        let mut tag = 0;
        let elty = self.array.element_type();
        if !find_union_component(elty, ty.as_value(), &mut tag) {
            let element_type = elty.display_string_or(CANNOT_DISPLAY_TYPE);
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE);
            return Ok(Err(Box::new(JlrsError::TypeError(
                TypeError::IncompatibleType {
                    element_type,
                    value_type,
                },
            ))));
        }

        let idx = self.array.dimensions().index_of(&index);
        if let Some(idx) = idx {
            // Safety: The data can be stored in this array, the tag is updated accordingly.
            unsafe {
                let offset = idx * self.array.element_size() as usize;
                self.array
                    .data_ptr()
                    .cast::<u8>()
                    .add(offset)
                    .cast::<L>()
                    .write(value);

                jlrs_array_typetagdata(self.array.unwrap(Private))
                    .add(idx)
                    .write(tag as _);
            }

            Ok(Ok(()))
        } else {
            Err(value)
        }
    }

    /// Sets the element at `index` to `value` using `ty` as the type.
    ///
    /// Safety: `ty` must be a valid variant of the element type, `index` must be in-bounds.
    pub unsafe fn set_unchecked<L, D>(&mut self, index: D, ty: DataType, value: L)
    where
        L: ValidField + IsBits,
        D: Dims,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let mut tag = 0;
        let array = self.array;
        let elty = array.element_type();
        let success = find_union_component(elty, ty.as_value(), &mut tag);
        debug_assert!(success);

        let idx = array.dimensions().index_of_unchecked(&index);
        let offset = idx * self.array.element_size() as usize;
        array
            .data_ptr()
            .cast::<u8>()
            .add(offset)
            .cast::<L>()
            .write(value);

        jlrs_array_typetagdata(self.array.unwrap(Private))
            .add(idx)
            .write(tag as _);
    }
}

/// An accessor for indeterminate data.
#[repr(transparent)]
pub struct IndeterminateAccessor<'borrow, 'scope, 'data, T, const N: isize> {
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow [MaybeUninit<u8>]>,
}

impl<'borrow, 'scope, 'data, T, const N: isize>
    IndeterminateAccessor<'borrow, 'scope, 'data, T, N>
{
    pub(crate) unsafe fn new(array: &'borrow ArrayBase<'scope, 'data, T, N>) -> Self {
        IndeterminateAccessor {
            array: *array,
            _data: PhantomData,
        }
    }
}

impl<'scope, 'data, T, const N: isize> Accessor<'scope, 'data, T, N>
    for IndeterminateAccessor<'_, 'scope, 'data, T, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

/// A mutable accessor for indeterminate data.
#[repr(transparent)]
pub struct IndeterminateAccessorMut<'borrow, 'scope, 'data, T, const N: isize> {
    array: ArrayBase<'scope, 'data, T, N>,
    _data: PhantomData<&'borrow mut [MaybeUninit<u8>]>,
}

impl<'borrow, 'scope, 'data, T, const N: isize>
    IndeterminateAccessorMut<'borrow, 'scope, 'data, T, N>
{
    pub(crate) unsafe fn new(array: &'borrow mut ArrayBase<'scope, 'data, T, N>) -> Self {
        IndeterminateAccessorMut {
            array: *array,
            _data: PhantomData,
        }
    }
}

impl<'scope, 'data, T, const N: isize> Accessor<'scope, 'data, T, N>
    for IndeterminateAccessorMut<'_, 'scope, 'data, T, N>
{
    #[inline]
    fn array(&self) -> &ArrayBase<'scope, 'data, T, N> {
        &self.array
    }
}

impl<'scope, 'data, T, const N: isize> AccessorMut<'scope, 'data, T, N>
    for IndeterminateAccessorMut<'_, 'scope, 'data, T, N>
{
}
