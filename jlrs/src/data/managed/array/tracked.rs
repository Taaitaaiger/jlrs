//! Track arrays to make directly accessing their content safer.

use std::{
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ops::Deref,
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
    Array, ArrayData, ArrayResult, TypedArray, TypedArrayData, TypedArrayResult,
};
use crate::{
    convert::unbox::Unbox,
    data::{
        layout::valid_layout::ValidField,
        managed::{value::ValueRef, Managed, ManagedRef},
    },
    error::JlrsResult,
    memory::{
        context::ledger::Ledger,
        target::{ExtendedTarget, Target},
    },
};

/// An array that has been tracked immutably.
pub struct TrackedArray<'tracked, 'scope, 'data, T: Managed<'scope, 'data>> {
    data: T,
    _scope: PhantomData<&'scope ()>,
    _tracked: PhantomData<&'tracked ()>,
    _data: PhantomData<&'data ()>,
}

unsafe impl<'tracked, 'scope, 'data, T: Managed<'scope, 'data>> Send
    for TrackedArray<'tracked, 'scope, 'data, T>
{
}

impl<'tracked, 'scope, 'data, T: Managed<'scope, 'data>> Clone
    for TrackedArray<'tracked, 'scope, 'data, T>
{
    fn clone(&self) -> Self {
        unsafe {
            Ledger::borrow_shared_unchecked(self.data.as_value()).unwrap();
            Self::new_from_owned(self.data)
        }
    }
}

impl<'tracked, 'scope, 'data, T: Managed<'scope, 'data>> TrackedArray<'tracked, 'scope, 'data, T> {
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
    /// Returns the dimensions of the tracked array.
    pub fn dimensions<'borrow>(&'borrow self) -> ArrayDimensions<'borrow> {
        unsafe { self.data.dimensions() }
    }

    /// Try to reborrow the array with the provided element type.
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

    /// Reborrow the array with the provided element type without checking if this conversion is valid.
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

    /// Copy the content of this array.
    pub fn copy_inline_data<T>(&self) -> JlrsResult<CopiedArray<T>>
    where
        T: 'static + ValidField + Unbox,
    {
        unsafe { self.data.copy_inline_data() }
    }

    /// Convert this array to a slice without checking if the layouts are compatible.
    pub unsafe fn as_slice_unchecked<'borrow, T>(&'borrow self) -> &'borrow [T] {
        self.data.as_slice_unchecked()
    }

    /// Create an accessor for the content of the array if the element type is an isbits type.
    pub fn bits_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<BitsArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ValidField + 'static,
    {
        unsafe { self.data.bits_data() }
    }

    /// Create an accessor for the content of the array if the element type is stored inline, but
    /// can contain references to managed data.
    pub fn inline_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<InlinePtrArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ValidField,
    {
        unsafe { self.data.inline_data() }
    }

    /// Create an accessor for the content of the array if the element type is a managed type.
    pub fn managed_data<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, T>>
    where
        T: ManagedRef<'scope, 'data>,
        Option<T>: ValidField,
    {
        unsafe { self.data.managed_data() }
    }

    /// Create an accessor for the content of the array if the element type is a non-inlined type
    /// (e.g. any mutable type).
    pub fn value_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        unsafe { self.data.value_data() }
    }

    /// Create an accessor for the content of the array if the element type is a bits union, i.e.
    /// a union of bits types.
    pub fn union_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<UnionArrayAccessorI<'borrow, 'scope, 'data>> {
        unsafe { self.data.union_data() }
    }

    /// Create an accessor for the content of the array that makes no assumptions about the
    /// element type.
    pub fn indeterminate_data<'borrow>(
        &'borrow self,
    ) -> IndeterminateArrayAccessorI<'borrow, 'scope, 'data> {
        unsafe { self.data.indeterminate_data() }
    }

    /// Reshape the array.
    ///
    /// Returns a new array with the provided dimensions, the content of the array is shared with
    /// the original array. The old and new dimensions must have an equal number of elements.
    pub fn reshape<'target, 'current, 'borrow, D, S>(
        &self,
        target: ExtendedTarget<'target, '_, '_, S>,
        dims: D,
    ) -> ArrayResult<'target, 'data, S>
    where
        D: Dims,
        S: Target<'target>,
    {
        unsafe { self.data.reshape(target, dims) }
    }

    /// Reshape the array.
    ///
    /// Returns a new array with the provided dimensions, the content of the array is shared with
    /// the original array. The old and new dimensions must have an equal number of elements.
    ///
    /// Safety: if an exception is thrown it isn't caught.
    pub unsafe fn reshape_unchecked<'target, 'current, 'borrow, D, S>(
        &self,
        target: ExtendedTarget<'target, '_, '_, S>,
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
    /// Returns the dimensions of the tracked array.
    pub fn dimensions<'borrow>(&'borrow self) -> ArrayDimensions<'borrow> {
        unsafe { self.data.dimensions() }
    }

    /// Copy the content of this array.
    pub fn copy_inline_data(&self) -> JlrsResult<CopiedArray<T>>
    where
        T: 'static,
    {
        unsafe { self.data.copy_inline_data() }
    }

    /// Convert this array to a slice.
    pub fn as_slice<'borrow>(&'borrow self) -> &'borrow [T] {
        unsafe {
            let arr = std::mem::transmute::<&'borrow Self, &'borrow Array>(self);
            arr.as_slice_unchecked()
        }
    }

    /// Create an accessor for the content of the array if the element type is an isbits type.
    pub fn bits_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<BitsArrayAccessorI<'borrow, 'scope, 'data, T>> {
        unsafe { self.data.bits_data() }
    }

    /// Create an accessor for the content of the array if the element type is stored inline, but
    /// can contain references to managed data.
    pub fn inline_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<InlinePtrArrayAccessorI<'borrow, 'scope, 'data, T>> {
        unsafe { self.data.inline_data() }
    }

    /// Create an accessor for the content of the array that makes no assumptions about the
    /// element type.
    pub fn indeterminate_data<'borrow>(
        &'borrow self,
    ) -> IndeterminateArrayAccessorI<'borrow, 'scope, 'data> {
        unsafe { self.data.indeterminate_data() }
    }

    /// Reshape the array.
    ///
    /// Returns a new array with the provided dimensions, the content of the array is shared with
    /// the original array. The old and new dimensions must have an equal number of elements.
    pub fn reshape<'target, 'current, 'borrow, D, S>(
        &self,
        target: ExtendedTarget<'target, '_, '_, S>,
        dims: D,
    ) -> TypedArrayResult<'target, 'data, S, T>
    where
        D: Dims,
        S: Target<'target>,
    {
        unsafe { self.data.reshape(target, dims) }
    }

    /// Reshape the array.
    ///
    /// Returns a new array with the provided dimensions, the content of the array is shared with
    /// the original array. The old and new dimensions must have an equal number of elements.
    ///
    /// Safety: if an exception is thrown it isn't caught.
    pub unsafe fn reshape_unchecked<'target, 'current, 'borrow, D, S>(
        &self,
        target: ExtendedTarget<'target, '_, '_, S>,
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
    /// Create an accessor for the content of the array if the element type is a managed type.
    pub fn managed_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, T>> {
        unsafe { self.data.managed_data() }
    }

    /// Create an accessor for the content of the array if the element type is a non-inlined type
    /// (e.g. any mutable type).
    pub fn value_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessorI<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        unsafe { self.data.value_data() }
    }
}

impl<'scope, 'data, T: Managed<'scope, 'data>> Drop for TrackedArray<'_, 'scope, 'data, T> {
    fn drop(&mut self) {
        unsafe {
            Ledger::unborrow_shared(self.data.as_value()).unwrap();
        }
    }
}

pub struct TrackedArrayMut<'tracked, 'scope, 'data, T: Managed<'scope, 'data>> {
    tracked: ManuallyDrop<TrackedArray<'tracked, 'scope, 'data, T>>,
}

impl<'tracked, 'scope, 'data, T: Managed<'scope, 'data>>
    TrackedArrayMut<'tracked, 'scope, 'data, T>
{
    pub(crate) unsafe fn new(data: &'tracked mut T) -> Self {
        TrackedArrayMut {
            tracked: ManuallyDrop::new(TrackedArray::new(data)),
        }
    }

    pub(crate) unsafe fn new_from_owned(data: T) -> Self {
        TrackedArrayMut {
            tracked: ManuallyDrop::new(TrackedArray::new_from_owned(data)),
        }
    }
}

impl<'tracked, 'scope, 'data> TrackedArrayMut<'tracked, 'scope, 'data, Array<'scope, 'data>> {
    /// Create a mutable accessor for the content of the array if the element type is an isbits
    /// type.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn bits_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<BitsArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidField,
    {
        self.tracked.data.bits_data_mut()
    }

    /// Create a mutable accessor for the content of the array if the element type is stored
    /// inline, but can contain references to managed data.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn inline_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<InlinePtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidField,
    {
        self.tracked.data.inline_data_mut()
    }

    /// Create a mutable accessor for the content of the array if the element type is a managed
    /// type.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn managed_data_mut<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ManagedRef<'scope, 'data>,
        Option<T>: ValidField,
    {
        self.tracked.data.managed_data_mut()
    }

    /// Create a mutable accessor for the content of the array if the element type is a
    /// non-inlined type (e.g. any mutable type).
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn value_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>> {
        self.tracked.data.value_data_mut()
    }

    /// Create a mutable accessor for the content of the array if the element type is a bits
    /// union, i.e. a union of bits types.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn union_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<UnionArrayAccessorMut<'borrow, 'scope, 'data>> {
        self.tracked.data.union_data_mut()
    }

    /// Create a mutable accessor for the content of the array that makes no assumptions about the
    /// element type.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn indeterminate_data_mut<'borrow>(
        &'borrow mut self,
    ) -> IndeterminateArrayAccessorMut<'borrow, 'scope, 'data> {
        self.tracked.data.indeterminate_data_mut()
    }

    /// Convert this array to a mutable slice without checking if the layouts are compatible.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn as_mut_slice_unchecked<'borrow, T>(&'borrow mut self) -> &'borrow mut [T]
    where
        T: 'static,
    {
        self.tracked.data.as_mut_slice_unchecked()
    }
}

impl<'tracked, 'scope> TrackedArrayMut<'tracked, 'scope, 'static, Array<'scope, 'static>> {
    /// Create a mutable accessor for the content of the array if the element type is a managed
    /// type.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
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

    /// Add capacity for `inc` more elements at the end of the array. The array must be
    /// one-dimensional. If the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented. If an
    /// exception is thrown, it isn't caught.
    pub unsafe fn grow_end_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_end_unchecked(inc);
    }

    /// Remove `dec` elements from the end of the array.  The array must be one-dimensional. If
    /// the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn del_end<'target, S>(&mut self, target: S, dec: usize) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.tracked.data.del_end(target, dec)
    }

    /// Remove `dec` elements from the end of the array.  The array must be one-dimensional. If
    /// the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented. If an
    /// exception is thrown, it isn't caught.
    pub unsafe fn del_end_unchecked(&mut self, dec: usize) {
        self.tracked.data.del_end_unchecked(dec);
    }

    /// Add capacity for `inc` more elements at the start of the array. The array must be
    /// one-dimensional. If the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
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

    /// Add capacity for `inc` more elements at the start of the array. The array must be
    /// one-dimensional. If the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented. If an
    /// exception is thrown, it isn't caught.
    pub unsafe fn grow_begin_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_begin_unchecked(inc);
    }

    /// Remove `dec` elements from the start of the array.  The array must be one-dimensional. If
    /// the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
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

    /// Remove `dec` elements from the start of the array.  The array must be one-dimensional. If
    /// the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented. If an
    /// exception is thrown, it isn't caught.
    pub unsafe fn del_begin_unchecked(&mut self, dec: usize) {
        self.tracked.data.del_begin_unchecked(dec);
    }
}

impl<'tracked, 'scope, 'data, T>
    TrackedArrayMut<'tracked, 'scope, 'data, TypedArray<'scope, 'data, T>>
where
    T: ValidField,
{
    /// Convert this array to a slice.
    pub fn as_mut_slice<'borrow>(&'borrow mut self) -> &'borrow mut [T] {
        unsafe {
            let arr = std::mem::transmute::<&'borrow mut Self, &'borrow mut Array>(self);
            arr.as_mut_slice_unchecked()
        }
    }

    /// Create a mutable accessor for the content of the array if the element type is an isbits
    /// type.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn bits_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<BitsArrayAccessorMut<'borrow, 'scope, 'data, T>> {
        self.tracked.data.bits_data_mut()
    }

    /// Create a mutable accessor for the content of the array if the element type is stored
    /// inline, but can contain references to managed data.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn inline_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<InlinePtrArrayAccessorMut<'borrow, 'scope, 'data, T>> {
        self.tracked.data.inline_data_mut()
    }

    /// Create a mutable accessor for the content of the array that makes no assumptions about the
    /// element type.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn indeterminate_data_mut<'borrow>(
        &'borrow mut self,
    ) -> IndeterminateArrayAccessorMut<'borrow, 'scope, 'data> {
        self.tracked.data.indeterminate_data_mut()
    }
}

unsafe impl<'tracked, 'scope, 'data, T: Managed<'scope, 'data>> Send
    for TrackedArrayMut<'tracked, 'scope, 'data, T>
{
}

impl<'tracked, 'scope, 'data, T>
    TrackedArrayMut<'tracked, 'scope, 'data, TypedArray<'scope, 'data, Option<T>>>
where
    T: ManagedRef<'scope, 'data>,
    Option<T>: ValidField,
{
    /// Create a mutable accessor for the content of the array if the element type is a managed
    /// type.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn managed_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, T>> {
        self.tracked.data.managed_data_mut()
    }

    /// Create a mutable accessor for the content of the array if the element type is a
    /// non-inlined type (e.g. any mutable type).
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
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
    /// Add capacity for `inc` more elements at the end of the array. The array must be
    /// one-dimensional. If the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
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

    /// Add capacity for `inc` more elements at the end of the array. The array must be
    /// one-dimensional. If the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented. If an
    /// exception is thrown, it isn't caught.
    pub unsafe fn grow_end_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_end_unchecked(inc)
    }

    /// Remove `dec` elements from the end of the array.  The array must be one-dimensional. If
    /// the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
    pub unsafe fn del_end<'target, S>(&mut self, target: S, dec: usize) -> S::Exception<'static, ()>
    where
        S: Target<'target>,
    {
        self.tracked.data.del_end(target, dec)
    }

    /// Remove `dec` elements from the end of the array.  The array must be one-dimensional. If
    /// the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented. If an
    /// exception is thrown, it isn't caught.
    pub unsafe fn del_end_unchecked(&mut self, dec: usize) {
        self.tracked.data.del_end_unchecked(dec)
    }

    /// Add capacity for `inc` more elements at the start of the array. The array must be
    /// one-dimensional. If the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
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

    /// Add capacity for `inc` more elements at the start of the array. The array must be
    /// one-dimensional. If the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented. If an
    /// exception is thrown, it isn't caught.
    pub unsafe fn grow_begin_unchecked(&mut self, inc: usize) {
        self.tracked.data.grow_begin_unchecked(inc)
    }

    /// Remove `dec` elements from the start of the array.  The array must be one-dimensional. If
    /// the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented.
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

    /// Remove `dec` elements from the start of the array.  The array must be one-dimensional. If
    /// the array isn't one-dimensional an exception is thrown.
    ///
    /// Safety: Mutating things that should absolutely not be mutated is not prevented. If an
    /// exception is thrown, it isn't caught.
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

impl<'tracked, 'scope, 'data, T: Managed<'scope, 'data>> Drop
    for TrackedArrayMut<'tracked, 'scope, 'data, T>
{
    fn drop(&mut self) {
        unsafe {
            Ledger::unborrow_exclusive(self.tracked.data.as_value()).unwrap();
        }
    }
}
