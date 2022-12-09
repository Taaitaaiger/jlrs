use std::{
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ops::{Deref, Range},
};

use super::{
    data::{
        accessor::{
            BitsArrayAccessorI, BitsArrayAccessorMut, IndeterminateArrayAccessorI,
            IndeterminateArrayAccessorMut, InlinePtrArrayAccessorI, InlinePtrArrayAccessorMut,
            PtrArrayAccessorI, PtrArrayAccessorMut, UnionArrayAccessorI, UnionArrayAccessorMut,
        },
        copied::CopiedArray,
    },
    dimensions::{ArrayDimensions, Dims},
    Array, ArrayData, TypedArray, TypedArrayData,
};
#[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
use super::{ArrayResult, TypedArrayResult};
use crate::{
    data::{
        layout::valid_layout::ValidField,
        managed::{value::ValueRef, ManagedRef},
    },
    error::JlrsResult,
    memory::{
        context::ledger::Ledger,
        target::{ExtendedTarget, Target},
    },
};

pub trait ArrayWrapper<'scope, 'data>: Copy {
    fn track<'borrow>(&'borrow self) -> JlrsResult<TrackedArray<'borrow, 'scope, 'data, Self>>;

    fn track_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<TrackedArrayMut<'borrow, 'scope, 'data, Self>>;

    fn data_range(&self) -> Range<*const u8>;
}

impl<'scope, 'data> ArrayWrapper<'scope, 'data> for Array<'scope, 'data> {
    fn track<'borrow>(&'borrow self) -> JlrsResult<TrackedArray<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow(self.data_range())?;
        unsafe { Ok(TrackedArray::new(self)) }
    }

    fn track_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<TrackedArrayMut<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow_mut(self.data_range())?;
        unsafe { Ok(TrackedArrayMut::new(self)) }
    }

    fn data_range(&self) -> Range<*const u8> {
        let ptr = self.data_ptr().cast();

        unsafe {
            let n_bytes = self.element_size() * self.dimensions().size();
            ptr..ptr.add(n_bytes)
        }
    }
}

impl<'scope, 'data, U: ValidField> ArrayWrapper<'scope, 'data> for TypedArray<'scope, 'data, U> {
    fn track<'borrow>(&'borrow self) -> JlrsResult<TrackedArray<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow(self.data_range())?;
        unsafe { Ok(TrackedArray::new(self)) }
    }

    fn track_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<TrackedArrayMut<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow_mut(self.data_range())?;
        unsafe { Ok(TrackedArrayMut::new(self)) }
    }

    fn data_range(&self) -> Range<*const u8> {
        let arr = self.as_array();
        let ptr = arr.data_ptr().cast();

        unsafe {
            let n_bytes = arr.element_size() * arr.dimensions().size();
            ptr..ptr.add(n_bytes)
        }
    }
}

pub struct TrackedArray<'tracked, 'scope, 'data, T>
where
    T: ArrayWrapper<'scope, 'data>,
{
    data: T,
    _scope: PhantomData<&'scope ()>,
    _tracked: PhantomData<&'tracked ()>,
    _data: PhantomData<&'data ()>,
}

impl<'tracked, 'scope, 'data, T> Clone for TrackedArray<'tracked, 'scope, 'data, T>
where
    T: ArrayWrapper<'scope, 'data>,
{
    fn clone(&self) -> Self {
        unsafe {
            Ledger::clone_shared(self.data.data_range());
            Self::new_from_owned(self.data)
        }
    }
}

impl<'tracked, 'scope, 'data, T> TrackedArray<'tracked, 'scope, 'data, T>
where
    T: ArrayWrapper<'scope, 'data>,
{
    pub(crate) unsafe fn new(data: &'tracked T) -> Self {
        TrackedArray {
            data: *data,
            _scope: PhantomData,
            _tracked: PhantomData,
            _data: PhantomData,
        }
    }

    pub(crate) unsafe fn new_from_owned(data: T) -> Self {
        TrackedArray {
            data: data,
            _scope: PhantomData,
            _tracked: PhantomData,
            _data: PhantomData,
        }
    }
}

impl<'tracked, 'scope, 'data> TrackedArray<'tracked, 'scope, 'data, Array<'scope, 'data>> {
    pub fn dimensions<'borrow>(&'borrow self) -> ArrayDimensions<'borrow> {
        unsafe { self.data.dimensions() }
    }

    pub fn try_as_typed<T>(
        self,
    ) -> JlrsResult<TrackedArray<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>>
    where
        T: ValidField,
    {
        let data = self.data.try_as_typed::<T>()?;
        let ret = unsafe { Ok(TrackedArray::new_from_owned(data)) };
        mem::forget(self);
        ret
    }

    pub unsafe fn as_typed_unchecked<T>(
        self,
    ) -> TrackedArray<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
    where
        T: ValidField,
    {
        let data = self.data.as_typed_unchecked::<T>();
        let ret = TrackedArray::new_from_owned(data);
        mem::forget(self);
        ret
    }

    pub fn copy_inline_data<T>(&self) -> JlrsResult<CopiedArray<T>>
    where
        T: 'static + ValidField,
    {
        unsafe { self.data.copy_inline_data() }
    }

    pub fn as_slice_unchecked<'borrow, T>(&'borrow self) -> &'borrow [T] {
        unsafe { self.data.as_slice_unchecked() }
    }

    pub fn bits_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<BitsArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ValidField,
    {
        unsafe { self.data.bits_data() }
    }

    pub fn inline_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<InlinePtrArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ValidField,
    {
        unsafe { self.data.inline_data() }
    }

    pub fn managed_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ManagedRef<'scope, 'data>,
        Option<T>: ValidField,
    {
        unsafe { self.data.managed_data() }
    }

    pub fn value_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        unsafe { self.data.value_data() }
    }

    pub fn union_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<UnionArrayAccessorI<'borrow, 'scope, 'data>> {
        unsafe { self.data.union_data() }
    }

    pub fn indeterminate_data<'borrow>(
        &'borrow self,
    ) -> IndeterminateArrayAccessorI<'borrow, 'scope, 'data> {
        unsafe { self.data.indeterminate_data() }
    }

    #[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
    pub fn reshape<'target, 'current, 'borrow, D, S>(
        &self,
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> ArrayResult<'target, 'data, S>
    where
        D: Dims,
        S: Target<'target>,
    {
        unsafe { self.data.reshape(target, dims) }
    }

    pub unsafe fn reshape_unchecked<'target, 'current, 'borrow, D, S>(
        &self,
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> ArrayData<'target, 'data, S>
    where
        D: Dims,
        S: Target<'target>,
    {
        self.data.reshape_unchecked(target, dims)
    }
}

impl<'tracked, 'scope, 'data, T> TrackedArray<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
where
    T: ValidField,
{
    pub fn dimensions<'borrow>(&'borrow self) -> ArrayDimensions<'borrow> {
        unsafe { self.data.dimensions() }
    }

    pub fn bits_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<BitsArrayAccessorI<'borrow, 'scope, 'data, T>> {
        unsafe { self.data.bits_data() }
    }

    pub fn inline_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<InlinePtrArrayAccessorI<'borrow, 'scope, 'data, T>> {
        unsafe { self.data.inline_data() }
    }

    pub fn indeterminate_data<'borrow>(
        &'borrow self,
    ) -> IndeterminateArrayAccessorI<'borrow, 'scope, 'data> {
        unsafe { self.data.indeterminate_data() }
    }

    #[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
    pub fn reshape<'target, 'current, 'borrow, D, S>(
        &self,
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> TypedArrayResult<'target, 'data, S, T>
    where
        D: Dims,
        S: Target<'target>,
    {
        unsafe { self.data.reshape(target, dims) }
    }

    pub unsafe fn reshape_unchecked<'target, 'current, 'borrow, D, S>(
        &self,
        target: ExtendedTarget<'target, 'current, 'borrow, S>,
        dims: D,
    ) -> TypedArrayData<'target, 'data, S, T>
    where
        D: Dims,
        S: Target<'target>,
    {
        self.data.reshape_unchecked(target, dims)
    }
}

impl<'tracked, 'scope, 'data, T>
    TrackedArray<'tracked, 'scope, 'data, TypedArray<'scope, 'data, Option<T>>>
where
    T: ManagedRef<'scope, 'data>,
    Option<T>: ValidField,
{
    pub fn managed_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, T>> {
        unsafe { self.data.managed_data() }
    }

    pub fn value_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        unsafe { self.data.value_data() }
    }
}

impl<'tracked, 'scope, 'data, T> TrackedArray<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
where
    T: 'static + ValidField,
{
    pub fn copy_inline_data(&self) -> JlrsResult<CopiedArray<T>> {
        unsafe { self.data.copy_inline_data() }
    }
}

impl<'scope, 'data, T: ArrayWrapper<'scope, 'data>> Drop for TrackedArray<'_, 'scope, 'data, T> {
    fn drop(&mut self) {
        Ledger::unborrow_shared(self.data.data_range());
    }
}

pub struct TrackedArrayMut<'tracked, 'scope, 'data, T>
where
    T: ArrayWrapper<'scope, 'data>,
{
    tracked: ManuallyDrop<TrackedArray<'tracked, 'scope, 'data, T>>,
}

impl<'tracked, 'scope, 'data, T> TrackedArrayMut<'tracked, 'scope, 'data, T>
where
    T: ArrayWrapper<'scope, 'data>,
{
    pub(crate) unsafe fn new(data: &'tracked mut T) -> Self {
        TrackedArrayMut {
            tracked: ManuallyDrop::new(TrackedArray::new(data)),
        }
    }
}

impl<'tracked, 'scope, 'data> TrackedArrayMut<'tracked, 'scope, 'data, Array<'scope, 'data>> {
    pub unsafe fn bits_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<BitsArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidField,
    {
        self.tracked.data.bits_data_mut()
    }

    pub unsafe fn inline_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<InlinePtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidField,
    {
        self.tracked.data.inline_data_mut()
    }

    pub unsafe fn managed_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ManagedRef<'scope, 'data>,
        Option<T>: ValidField,
    {
        self.tracked.data.managed_data_mut()
    }

    pub unsafe fn value_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        self.tracked.data.value_data_mut()
    }

    pub unsafe fn union_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<UnionArrayAccessorMut<'borrow, 'scope, 'data>> {
        self.tracked.data.union_data_mut()
    }

    pub unsafe fn indeterminate_data_mut<'borrow>(
        &'borrow mut self,
    ) -> IndeterminateArrayAccessorMut<'borrow, 'scope, 'data> {
        self.tracked.data.indeterminate_data_mut()
    }

    pub unsafe fn as_mut_slice_unchecked<'borrow, T>(&'borrow mut self) -> &'borrow mut [T] {
        self.tracked.data.as_mut_slice_unchecked()
    }
}

impl<'tracked, 'scope> TrackedArrayMut<'tracked, 'scope, 'static, Array<'scope, 'static>> {
    #[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
    pub unsafe fn grow_end<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.tracked.data.grow_end(target, inc)
    }

    pub unsafe fn grow_end_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_end_unchecked(inc)
    }

    #[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
    pub unsafe fn del_end<'target, S>(&mut self, target: S, dec: usize) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.tracked.data.del_end(target, dec)
    }

    pub unsafe fn del_end_unchecked(&mut self, dec: usize) {
        self.tracked.data.del_end_unchecked(dec)
    }

    #[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
    pub unsafe fn grow_begin<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.tracked.data.grow_begin(target, inc)
    }

    pub unsafe fn grow_begin_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_begin_unchecked(inc)
    }

    #[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
    pub unsafe fn del_begin<'target, S>(
        &mut self,
        target: S,
        dec: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.tracked.data.del_begin(target, dec)
    }

    pub unsafe fn del_begin_unchecked(&mut self, dec: usize) {
        self.tracked.data.del_begin_unchecked(dec)
    }
}

impl<'tracked, 'scope, 'data, T>
    TrackedArrayMut<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
where
    T: ValidField,
{
    pub unsafe fn bits_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<BitsArrayAccessorMut<'borrow, 'scope, 'data, T>> {
        self.tracked.data.bits_data_mut()
    }

    pub unsafe fn inline_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<InlinePtrArrayAccessorMut<'borrow, 'scope, 'data, T>> {
        self.tracked.data.inline_data_mut()
    }

    pub unsafe fn indeterminate_data_mut<'borrow>(
        &'borrow mut self,
    ) -> IndeterminateArrayAccessorMut<'borrow, 'scope, 'data> {
        self.tracked.data.indeterminate_data_mut()
    }
}

impl<'tracked, 'scope, 'data, T>
    TrackedArrayMut<'tracked, 'scope, 'data, TypedArray<'scope, 'data, Option<T>>>
where
    T: ManagedRef<'scope, 'data>,
    Option<T>: ValidField,
{
    pub unsafe fn managed_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, T>> {
        self.tracked.data.managed_data_mut()
    }

    pub unsafe fn value_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        self.tracked.data.value_data_mut()
    }
}

impl<'tracked, 'scope, T> TrackedArrayMut<'tracked, 'scope, 'static, TypedArray<'scope, 'static, T>>
where
    T: ValidField,
{
    #[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
    pub unsafe fn grow_end<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.tracked.data.grow_end(target, inc)
    }

    pub unsafe fn grow_end_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_end_unchecked(inc)
    }

    #[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
    pub unsafe fn del_end<'target, S>(&mut self, target: S, dec: usize) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.tracked.data.del_end(target, dec)
    }

    pub unsafe fn del_end_unchecked(&mut self, dec: usize) {
        self.tracked.data.del_end_unchecked(dec)
    }

    #[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
    pub unsafe fn grow_begin<'target, S>(
        &mut self,
        target: S,
        inc: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.tracked.data.grow_begin(target, inc)
    }

    pub unsafe fn grow_begin_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_begin_unchecked(inc)
    }

    #[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
    pub unsafe fn del_begin<'target, S>(
        &mut self,
        target: S,
        dec: usize,
    ) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.tracked.data.del_begin(target, dec)
    }

    pub unsafe fn del_begin_unchecked(&mut self, dec: usize) {
        self.tracked.data.del_begin_unchecked(dec)
    }
}

impl<'tracked, 'scope, 'data> Deref
    for TrackedArrayMut<'tracked, 'scope, 'data, Array<'scope, 'data>>
{
    type Target = TrackedArray<'tracked, 'scope, 'data, Array<'scope, 'data>>;

    fn deref(&self) -> &Self::Target {
        &self.tracked
    }
}

impl<'tracked, 'scope, 'data, T> Deref
    for TrackedArrayMut<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
where
    T: ValidField,
{
    type Target = TrackedArray<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>;

    fn deref(&self) -> &Self::Target {
        &self.tracked
    }
}

impl<'tracked, 'scope, 'data, T> Drop for TrackedArrayMut<'tracked, 'scope, 'data, T>
where
    T: ArrayWrapper<'scope, 'data>,
{
    fn drop(&mut self) {
        Ledger::unborrow_owned(self.tracked.data.data_range());
    }
}
