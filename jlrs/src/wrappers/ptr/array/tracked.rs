use crate::{
    error::JuliaResult,
    layout::valid_layout::ValidLayout,
    memory::{frame::GcFrame, ledger::Ledger},
    prelude::{Frame, JlrsResult, Scope, ValueRef},
    wrappers::ptr::WrapperRef,
};
use std::{
    cell::RefCell,
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
    Array, TypedArray,
};

pub trait ArrayWrapper<'scope, 'data>: Copy {
    fn track<'borrow, 'frame: 'borrow, F: Frame<'frame>>(
        &'borrow self,
        frame: &F,
    ) -> JlrsResult<TrackedArray<'borrow, 'scope, 'data, Self>>;

    fn track_mut<'borrow, 'frame: 'borrow, F: Frame<'frame>>(
        &'borrow mut self,
        frame: &F,
    ) -> JlrsResult<TrackedArrayMut<'borrow, 'scope, 'data, Self>>;

    fn data_range(&self) -> Range<*const u8>;
}

impl<'scope, 'data> ArrayWrapper<'scope, 'data> for Array<'scope, 'data> {
    fn track<'borrow, 'frame: 'borrow, F: Frame<'frame>>(
        &'borrow self,
        frame: &F,
    ) -> JlrsResult<TrackedArray<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow_array(frame.ledger(), *self)
    }

    fn track_mut<'borrow, 'frame: 'borrow, F: Frame<'frame>>(
        &'borrow mut self,
        frame: &F,
    ) -> JlrsResult<TrackedArrayMut<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow_array_mut(frame.ledger(), *self)
    }

    fn data_range(&self) -> Range<*const u8> {
        let ptr = self.data_ptr().cast();

        unsafe {
            let n_bytes = self.element_size() * self.dimensions().size();
            ptr..ptr.add(n_bytes)
        }
    }
}

impl<'scope, 'data, T: ValidLayout> ArrayWrapper<'scope, 'data> for TypedArray<'scope, 'data, T> {
    fn track<'borrow, 'frame: 'borrow, F: Frame<'frame>>(
        &'borrow self,
        frame: &F,
    ) -> JlrsResult<TrackedArray<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow_array(frame.ledger(), *self)
    }

    fn track_mut<'borrow, 'frame: 'borrow, F: Frame<'frame>>(
        &'borrow mut self,
        frame: &F,
    ) -> JlrsResult<TrackedArrayMut<'borrow, 'scope, 'data, Self>> {
        Ledger::try_borrow_array_mut(frame.ledger(), *self)
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
    ledger: &'tracked RefCell<Ledger>,
    data: T,
    _scope: PhantomData<&'scope ()>,
    _data: PhantomData<&'data ()>,
}

impl<'tracked, 'scope, 'data, T> Clone for TrackedArray<'tracked, 'scope, 'data, T>
where
    T: ArrayWrapper<'scope, 'data>,
{
    fn clone(&self) -> Self {
        unsafe {
            Ledger::clone_shared(self.ledger, self.data);
            Self::new(self.ledger, self.data)
        }
    }
}

impl<'tracked, 'scope, 'data, T> TrackedArray<'tracked, 'scope, 'data, T>
where
    T: ArrayWrapper<'scope, 'data>,
{
    pub(crate) unsafe fn new(ledger: &'tracked RefCell<Ledger>, data: T) -> Self {
        TrackedArray {
            ledger,
            data,
            _scope: PhantomData,
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
        T: ValidLayout,
    {
        let data = self.data.try_as_typed::<T>()?;
        let ret = unsafe { Ok(TrackedArray::new(self.ledger, data)) };
        mem::forget(self);
        ret
    }

    pub unsafe fn as_typed_unchecked<T>(
        self,
    ) -> TrackedArray<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        let data = self.data.as_typed_unchecked::<T>();
        let ret = TrackedArray::new(self.ledger, data);
        mem::forget(self);
        ret
    }

    pub fn copy_inline_data<T>(&self) -> JlrsResult<CopiedArray<T>>
    where
        T: 'static + ValidLayout,
    {
        unsafe { self.data.copy_inline_data() }
    }

    pub fn bits_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<BitsArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        unsafe { self.data.bits_data() }
    }

    pub fn inline_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<InlinePtrArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        unsafe { self.data.inline_data() }
    }

    pub fn wrapper_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: WrapperRef<'scope, 'data>,
    {
        unsafe { self.data.wrapper_data() }
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

    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn reshape<'target, 'current, D, S, F>(
        &self,
        scope: S,
        dims: D,
    ) -> JuliaResult<'target, 'data, Array<'target, 'data>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        unsafe { self.data.reshape(scope, dims) }
    }

    pub unsafe fn reshape_unchecked<'target, 'current, D, S, F>(
        &self,
        scope: S,
        dims: D,
    ) -> Array<'target, 'data>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        self.data.reshape_unchecked(scope, dims)
    }
}

impl<'tracked, 'scope, 'data, T> TrackedArray<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
where
    T: ValidLayout,
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

    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn reshape<'target, 'current, D, S, F>(
        &self,
        scope: S,
        dims: D,
    ) -> JuliaResult<'target, 'data, TypedArray<'target, 'data, T>>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        unsafe { self.data.reshape(scope, dims) }
    }

    pub unsafe fn reshape_unchecked<'target, 'current, D, S, F>(
        &self,
        scope: S,
        dims: D,
    ) -> TypedArray<'target, 'data, T>
    where
        D: Dims,
        S: Scope<'target, 'current, F>,
        F: Frame<'current>,
    {
        self.data.reshape_unchecked(scope, dims)
    }
}

impl<'tracked, 'scope, 'data, T> TrackedArray<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
where
    T: WrapperRef<'scope, 'data>,
{
    pub fn wrapper_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, T>> {
        unsafe { self.data.wrapper_data() }
    }

    pub fn value_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        unsafe { self.data.value_data() }
    }
}

impl<'tracked, 'scope, 'data, T> TrackedArray<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
where
    T: 'static + ValidLayout,
{
    pub fn copy_inline_data(&self) -> JlrsResult<CopiedArray<T>> {
        unsafe { self.data.copy_inline_data() }
    }
}

impl<'scope, 'data, T: ArrayWrapper<'scope, 'data>> Drop for TrackedArray<'_, 'scope, 'data, T> {
    fn drop(&mut self) {
        let mut ledger = self.ledger.borrow_mut();
        let range = self.data.data_range();
        let i = ledger.shared.iter().rposition(|r| r == &range).unwrap();

        ledger.shared.remove(i);
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
    pub(crate) unsafe fn new(ledger: &'tracked RefCell<Ledger>, data: T) -> Self {
        TrackedArrayMut {
            tracked: ManuallyDrop::new(TrackedArray::new(ledger, data)),
        }
    }
}

impl<'tracked, 'scope, 'data> TrackedArrayMut<'tracked, 'scope, 'data, Array<'scope, 'data>> {
    pub unsafe fn bits_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<BitsArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.tracked.data.bits_data_mut()
    }

    pub unsafe fn inline_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<InlinePtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.tracked.data.inline_data_mut()
    }

    pub unsafe fn wrapper_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: WrapperRef<'scope, 'data>,
    {
        self.tracked.data.wrapper_data_mut()
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
}

impl<'tracked, 'scope> TrackedArrayMut<'tracked, 'scope, 'static, Array<'scope, 'static>> {
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn grow_end<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        inc: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.tracked.data.grow_end(frame, inc)
    }

    pub unsafe fn grow_end_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_end_unchecked(inc)
    }

    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn del_end<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        dec: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.tracked.data.del_end(frame, dec)
    }

    pub unsafe fn del_end_unchecked(&mut self, dec: usize) {
        self.tracked.data.del_end_unchecked(dec)
    }

    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn grow_begin<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        inc: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.tracked.data.grow_begin(frame, inc)
    }

    pub unsafe fn grow_begin_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_begin_unchecked(inc)
    }

    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn del_begin<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        dec: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.tracked.data.del_begin(frame, dec)
    }

    pub unsafe fn del_begin_unchecked(&mut self, dec: usize) {
        self.tracked.data.del_begin_unchecked(dec)
    }
}

impl<'tracked, 'scope, 'data, T>
    TrackedArrayMut<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
where
    T: ValidLayout,
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
    TrackedArrayMut<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
where
    T: WrapperRef<'scope, 'data>,
{
    pub unsafe fn wrapper_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, T>> {
        self.tracked.data.wrapper_data_mut()
    }

    pub unsafe fn value_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        self.tracked.data.value_data_mut()
    }
}

impl<'tracked, 'scope, T> TrackedArrayMut<'tracked, 'scope, 'static, TypedArray<'scope, 'static, T>>
where
    T: ValidLayout,
{
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn grow_end<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        inc: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.tracked.data.grow_end(frame, inc)
    }

    pub unsafe fn grow_end_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_end_unchecked(inc)
    }

    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn del_end<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        dec: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.tracked.data.del_end(frame, dec)
    }

    pub unsafe fn del_end_unchecked(&mut self, dec: usize) {
        self.tracked.data.del_end_unchecked(dec)
    }

    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn grow_begin<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        inc: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.tracked.data.grow_begin(frame, inc)
    }

    pub unsafe fn grow_begin_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_begin_unchecked(inc)
    }

    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub unsafe fn del_begin<'current>(
        &mut self,
        frame: &mut GcFrame<'current>,
        dec: usize,
    ) -> JuliaResult<'current, 'static, ()> {
        self.tracked.data.del_begin(frame, dec)
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
    T: ValidLayout,
{
    type Target = TrackedArray<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>;

    fn deref(&self) -> &Self::Target {
        &self.tracked
    }
}

impl<'tracked, 'scope, 'data, T> Drop
    for TrackedArrayMut<'tracked, 'scope, 'data, T>
where
    T: ArrayWrapper<'scope, 'data>,
{
    fn drop(&mut self) {
        let mut ledger = self.tracked.ledger.borrow_mut();
        let range = self.tracked.data.data_range();
        let i = ledger.owned.iter().rposition(|r| r == &range).unwrap();

        ledger.owned.remove(i);
    }
}
