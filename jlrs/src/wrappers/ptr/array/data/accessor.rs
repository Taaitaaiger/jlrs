//! Access and modify the contents of Julia arrays.

#[cfg(not(all(
    target_os = "windows",
    all(feature = "lts", not(feature = "all-features-override"))
)))]
use crate::error::{JuliaResult, JuliaResultRef};

use crate::{
    error::{AccessError, JlrsResult, TypeError, CANNOT_DISPLAY_TYPE},
    layout::valid_layout::ValidLayout,
    memory::{frame::Frame, scope::PartialScope},
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
        value::ValueRef,
        Wrapper, WrapperRef,
    },
};
use jl_sys::{jl_array_ptr_set, jl_array_typetagdata, jl_arrayref, jl_arrayset};
use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::{null_mut, NonNull},
    slice,
};

/// Trait used to indicate how the elements are laid out.
pub trait ArrayLayout: Sized {}

/// Indeterminate layout.
pub enum UnknownLayout {}
impl ArrayLayout for UnknownLayout {}

/// Layout for inline elements with no pointer fields.
pub enum BitsLayout {}
impl ArrayLayout for BitsLayout {}

/// Layout for elements that are bits unions.
pub enum UnionLayout {}
impl ArrayLayout for UnionLayout {}

/// Layout for elements that are pointers.
pub enum PtrLayout {}
impl ArrayLayout for PtrLayout {}

/// Layout for inline elements that have pointer fields.
pub enum InlinePtrLayout {}
impl ArrayLayout for InlinePtrLayout {}

/// Trait used to indicate if the array is accessed mutably or immutably.
pub trait Mutability: Sized {}

/// Immutable array access.
pub struct Immutable<'borrow, T> {
    _marker: PhantomData<&'borrow [T]>,
}
impl<'borrow, T> Mutability for Immutable<'borrow, T> {}

/// Mutable array access.
pub struct Mutable<'borrow, T> {
    _marker: PhantomData<&'borrow mut [T]>,
}
impl<'borrow, T> Mutability for Mutable<'borrow, T> {}

/// An accessor for Julia arrays.
///
/// What methods are available depends on the layout of the data and whether the data is accessed
/// mutably or immutably. The elements can always be accessed as Julia data with
/// [`ArrayAccessor::get_value`], and if the accessor is mutable its contents can be changed with
/// [`ArrayAccessor::get_value`].
///
/// There are four possible layouts:
///
///  - [`BitsLayout`]
///   The element type `T` is an `isbits` type, the array is stored as an array of `T`s. Because
///   these types store no pointers the `IndexMut` trait is implemented for mutable accessor for
///   this layout.
///
///  - [`InlinePtrLayout`]
///   The element type `T` is an inline type, the array is stored as an array of `T`s. Because
///   these types might store pointers the `IndexMut` trait is not implemented, but `Index` is.
///   You can update its contents with [`ArrayAccessor::set_value`].
///
///  - [`UnionLayout`]
///   The element type is a union of `isbits` types, the data and flags of these elements are
///   stored separately in different parts of the array. Due to how the data is stored the `Index`
///   trait is not implemented. You can use [`UnionArrayAccessor::get`] and
///   [`UnionArrayAccessor::set`] instead.
///
///  - [`PtrLayout`]
///   The element type is a mutable type or is not concrete, the elements are stored as pointers
///   to Julia data (i.e. as [`ValueRef`]s). The `IndexMut` trait is not implemented, but `Index`
///   is. You can mutate its contents with [`ArrayAccessor::set_value`].
///
/// In addition to these four layouts, there's also [`UnknownLayout`] which doesn't impose any
/// requirements on the layout, but as a result it can only access its contents with
/// [`ArrayAccessor::get_value`] and mutate them with [`ArrayAccessor::set_value`].
#[repr(transparent)]
pub struct ArrayAccessor<'borrow, 'array, 'data, T, L: ArrayLayout, M: Mutability> {
    array: Array<'array, 'data>,
    _lt_marker: PhantomData<&'borrow ()>,
    _ty_marker: PhantomData<*mut T>,
    _layout_marker: PhantomData<L>,
    _mut_marker: PhantomData<M>,
}

/// A type alias for an ArrayAcccessor for the `BitsLayout`.
pub type BitsArrayAccessor<'borrow, 'array, 'data, T, M> =
    ArrayAccessor<'borrow, 'array, 'data, T, BitsLayout, M>;

/// A type alias for an ArrayAcccessor for the `InlinePtrLayout`.
pub type InlinePtrArrayAccessor<'borrow, 'array, 'data, T, M> =
    ArrayAccessor<'borrow, 'array, 'data, T, InlinePtrLayout, M>;

/// A type alias for an ArrayAcccessor for the `UnionLayout`.
pub type UnionArrayAccessor<'borrow, 'array, 'data, M> =
    ArrayAccessor<'borrow, 'array, 'data, u8, UnionLayout, M>;

/// A type alias for an ArrayAcccessor for the `PtrLayout`.
pub type PtrArrayAccessor<'borrow, 'array, 'data, T, M> =
    ArrayAccessor<'borrow, 'array, 'data, T, PtrLayout, M>;

/// A type alias for an ArrayAcccessor for the `UnknownLayout`.
pub type IndeterminateArrayAccessor<'borrow, 'array, 'data, M> =
    ArrayAccessor<'borrow, 'array, 'data, u8, UnknownLayout, M>;

impl<'borrow, 'array, 'data, T, L: ArrayLayout> Clone
    for ArrayAccessor<'borrow, 'array, 'data, T, L, Immutable<'borrow, T>>
{
    fn clone(&self) -> Self {
        ArrayAccessor {
            array: self.array,
            _lt_marker: PhantomData,
            _ty_marker: PhantomData,
            _layout_marker: PhantomData,
            _mut_marker: PhantomData,
        }
    }
}

impl<'borrow, 'array, 'data, T, L: ArrayLayout, M: Mutability>
    ArrayAccessor<'borrow, 'array, 'data, T, L, M>
{
    // Safety: The representation of T and the element type must match if L is not
    // `UnknownLayout`.
    pub(crate) unsafe fn new<'frame, F>(array: Array<'array, 'data>, _: &'borrow mut F) -> Self
    where
        F: Frame<'frame>,
    {
        ArrayAccessor {
            array,
            _lt_marker: PhantomData,
            _ty_marker: PhantomData,
            _layout_marker: PhantomData,
            _mut_marker: PhantomData,
        }
    }

    // Safety: The representation of T and the element type must match if L is not
    // `UnknownLayout`. You must not create multiple mutable references to the same data.
    pub(crate) unsafe fn unrestricted_new<'frame, F>(
        array: Array<'array, 'data>,
        _: &'borrow F,
    ) -> Self
    where
        F: Frame<'frame>,
    {
        ArrayAccessor {
            array,
            _lt_marker: PhantomData,
            _ty_marker: PhantomData,
            _layout_marker: PhantomData,
            _mut_marker: PhantomData,
        }
    }

    /// Access the element at `index` and convert it to a `Value` rooted in `scope`.
    ///
    /// If an error is thrown by Julia it's caught and returned.
    #[cfg(not(all(
        target_os = "windows",
        all(feature = "lts", not(feature = "all-features-override"))
    )))]
    pub fn get_value<'frame, D: Dims, P: PartialScope<'frame>>(
        &mut self,
        scope: P,
        index: D,
    ) -> JlrsResult<JuliaResult<'frame, 'data>> {
        use crate::catch::catch_exceptions;
        use jl_sys::jl_value_t;
        use std::mem::MaybeUninit;

        let idx = self.array.dimensions().index_of(&index)?;

        // Safety: exceptions are caught, the result is immediately rooted
        unsafe {
            let mut callback = |result: &mut MaybeUninit<*mut jl_value_t>| {
                let res = jl_arrayref(self.array.unwrap(Private), idx);
                result.write(res);
                Ok(())
            };

            match catch_exceptions(&mut callback)? {
                Ok(ptr) => Ok(Ok(scope.value(NonNull::new_unchecked(ptr), Private)?)),
                Err(e) => Ok(Err(e.root(scope)?)),
            }
        }
    }

    /// Access the element at `index` and convert it to a `Value` rooted in `scope`.
    ///
    /// Safety: If an error is thrown by Julia it's not caught.
    pub unsafe fn get_value_unchecked<'frame, D: Dims, P: PartialScope<'frame>>(
        &mut self,
        scope: P,
        index: D,
    ) -> JlrsResult<Value<'frame, 'data>> {
        let idx = self.array.dimensions().index_of(&index)?;
        let res = jl_arrayref(self.array.unwrap(Private), idx);
        scope.value(NonNull::new_unchecked(res), Private)
    }

    /// Access the element at `index` and convert it to a `ValueRef`.
    ///
    /// If an error is thrown by Julia it's caught and returned.
    #[cfg(not(all(
        target_os = "windows",
        all(feature = "lts", not(feature = "all-features-override"))
    )))]
    pub fn get_value_unrooted<D: Dims>(
        &mut self,
        index: D,
    ) -> JlrsResult<JuliaResultRef<'array, 'data>> {
        use jl_sys::jl_value_t;

        use crate::catch::catch_exceptions;
        use std::mem::MaybeUninit;

        let idx = self.array.dimensions().index_of(&index)?;

        // Safety: exceptions are caught
        unsafe {
            let mut callback = |result: &mut MaybeUninit<*mut jl_value_t>| {
                let res = jl_arrayref(self.array.unwrap(Private), idx);
                result.write(res);
                Ok(())
            };

            match catch_exceptions(&mut callback)? {
                Ok(ptr) => Ok(Ok(ValueRef::wrap(ptr))),
                Err(e) => Ok(Err(e)),
            }
        }
    }

    /// Access the element at `index` and convert it to a `ValueRef`.
    ///
    /// Safety: If an error is thrown by Julia it's not caught.
    pub unsafe fn get_value_unrooted_unchecked<'frame, D: Dims, P: PartialScope<'frame>>(
        &mut self,
        index: D,
    ) -> JlrsResult<ValueRef<'array, 'data>> {
        let idx = self.array.dimensions().index_of(&index)?;
        let res = jl_arrayref(self.array.unwrap(Private), idx);
        Ok(ValueRef::wrap(res))
    }

    /// Returns the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'array> {
        ArrayDimensions::new(self.array)
    }
}

impl<'borrow, 'array, 'data, T, L: ArrayLayout>
    ArrayAccessor<'borrow, 'array, 'data, T, L, Mutable<'borrow, T>>
{
    /// Set the element at `index` to `value`.
    ///
    /// If an error is thrown by Julia it's caught and returned.
    #[cfg(not(all(
        target_os = "windows",
        all(feature = "lts", not(feature = "all-features-override"))
    )))]
    pub fn set_value<'frame, D: Dims, F: Frame<'frame>>(
        &mut self,
        frame: &mut F,
        index: D,
        value: Option<Value<'_, 'data>>,
    ) -> JlrsResult<JuliaResult<'frame, 'static, ()>> {
        use crate::catch::catch_exceptions;
        use std::mem::MaybeUninit;

        let idx = self.array.dimensions().index_of(&index)?;
        let ptr = value.map(|v| v.unwrap(Private)).unwrap_or(null_mut());

        // Safety: exceptions are caught, if one is thrown it's immediately rooted
        unsafe {
            let mut callback = |result: &mut MaybeUninit<()>| {
                jl_arrayset(self.array.unwrap(Private), ptr, idx);
                result.write(());
                Ok(())
            };

            match catch_exceptions(&mut callback)? {
                Ok(()) => Ok(Ok(())),
                Err(e) => Ok(Err(e.root(&mut *frame)?)),
            }
        }
    }

    /// Set the element at `index` to `value`.
    ///
    /// Safety: If an error is thrown by Julia it's not caught.
    pub unsafe fn set_value_unchecked<D: Dims>(
        &mut self,
        index: D,
        value: Option<Value<'_, 'data>>,
    ) -> JlrsResult<()> {
        let idx = self.array.dimensions().index_of(&index)?;
        let ptr = value.map(|v| v.unwrap(Private)).unwrap_or(null_mut());
        jl_arrayset(self.array.unwrap(Private), ptr, idx);
        Ok(())
    }
}

impl<'borrow, 'array, 'data, T: WrapperRef<'array, 'data>, M: Mutability>
    PtrArrayAccessor<'borrow, 'array, 'data, T, M>
{
    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D>(&self, index: D) -> Option<T>
    where
        D: Dims,
    {
        let idx = self.dimensions().index_of(&index).ok()?;
        // The index is in-bounds, the type has been checked in advance
        unsafe { self.array.data_ptr().cast::<T>().add(idx).as_ref().cloned() }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        let n_elems = self.dimensions().size();
        let arr_data = self.array.data_ptr().cast::<T>();
        // Safety: the layout is compatible and the lifetime is limited.
        unsafe { slice::from_raw_parts(arr_data, n_elems) }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn into_slice(self) -> &'borrow [T] {
        let n_elems = self.dimensions().size();
        let arr_data = self.array.data_ptr().cast::<T>();
        // Safety: the layout is compatible and the lifetime is limited.
        unsafe { slice::from_raw_parts(arr_data, n_elems) }
    }
}

impl<'borrow, 'array, 'data, T: WrapperRef<'array, 'data>>
    PtrArrayAccessor<'borrow, 'array, 'data, T, Mutable<'borrow, T>>
{
    /// Set the value at `index` to `value` if `value` has a type that's compatible with this array.
    pub fn set<D>(&mut self, index: D, value: Option<Value<'_, 'data>>) -> JlrsResult<()>
    where
        D: Dims,
    {
        let ptr = self.array.unwrap(Private);
        let idx = self.dimensions().index_of(&index)?;

        let data_ptr = if let Some(value) = value {
            if !value.isa(self.array.element_type()) {
                let element_type_str = self
                    .array
                    .element_type()
                    .display_string_or(CANNOT_DISPLAY_TYPE);
                let value_type_str = value.datatype().display_string_or(CANNOT_DISPLAY_TYPE);
                Err(TypeError::IncompatibleType {
                    element_type_str,
                    value_type_str,
                })?;
            }

            value.unwrap(Private)
        } else {
            null_mut()
        };

        // Safety: the index is in bounds, the value can be stored in this array
        unsafe { jl_array_ptr_set(ptr.cast(), idx, data_ptr.cast()) };

        Ok(())
    }
}

impl<'borrow, 'array, 'data, D, T, M> Index<D> for PtrArrayAccessor<'borrow, 'array, 'data, T, M>
where
    D: Dims,
    T: WrapperRef<'array, 'data>,
    M: Mutability,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        let idx = self.dimensions().index_of(&index).unwrap();
        // Safety: the index is in bounds
        unsafe { self.array.data_ptr().cast::<T>().add(idx).as_ref().unwrap() }
    }
}

impl<'borrow, 'array, 'data, T, M: Mutability> BitsArrayAccessor<'borrow, 'array, 'data, T, M> {
    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D>(&self, index: D) -> Option<&T>
    where
        D: Dims,
    {
        let idx = self.dimensions().index_of(&index).ok()?;
        // Safety: the index is in bounds
        unsafe { self.array.data_ptr().cast::<T>().add(idx).as_ref() }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        let len = self.dimensions().size();
        let data = self.array.data_ptr().cast::<T>();
        // Safety: the layout is compatible and the lifetime is limited.
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn into_slice(self) -> &'borrow [T] {
        let len = self.dimensions().size();
        let data = self.array.data_ptr().cast::<T>();
        // Safety: the layout is compatible and the lifetime is limited.
        unsafe { slice::from_raw_parts(data, len) }
    }
}

impl<'borrow, 'array, 'data, T> BitsArrayAccessor<'borrow, 'array, 'data, T, Mutable<'borrow, T>> {
    /// Set the value at `index` to `value` if `value` has a type that's compatible with this array.
    pub fn set<D>(&mut self, index: D, value: T) -> JlrsResult<()>
    where
        D: Dims,
    {
        let idx = self.dimensions().index_of(&index)?;
        // Safety: the index is in bounds and layout is compatible.
        unsafe { self.array.data_ptr().cast::<T>().add(idx).write(value) };

        Ok(())
    }

    /// Get a mutable reference to the element stored at `index`.
    pub fn get_mut<D>(&mut self, index: D) -> Option<&mut T>
    where
        D: Dims,
    {
        let idx = self.dimensions().index_of(&index).ok()?;
        // Safety: the index is in bounds and layout is compatible.
        unsafe { self.array.data_ptr().cast::<T>().add(idx).as_mut() }
    }

    /// Returns the array's data as a mutable slice, the data is in column-major order.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.dimensions().size();
        let data = self.array.data_ptr().cast::<T>();
        // Safety: the layout is compatible and the lifetime is limited.
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    /// Returns the array's data as a mutable slice, the data is in column-major order.
    pub fn into_mut_slice(self) -> &'borrow mut [T] {
        let len = self.dimensions().size();
        let data = self.array.data_ptr().cast::<T>();
        // Safety: the layout is compatible and the lifetime is limited.
        unsafe { slice::from_raw_parts_mut(data, len) }
    }
}

impl<'borrow, 'array, 'data, T, M, D> Index<D> for BitsArrayAccessor<'borrow, 'array, 'data, T, M>
where
    D: Dims,
    M: Mutability,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        let idx = self.dimensions().index_of(&index).unwrap();
        // Safety: the layout is compatible and the index is in bounds.
        unsafe {
            self.array
                .data_ptr()
                .cast::<T>()
                .add(idx)
                .as_ref()
                .unwrap_unchecked()
        }
    }
}

impl<'borrow, 'array, 'data, T, D> IndexMut<D>
    for BitsArrayAccessor<'borrow, 'array, 'data, T, Mutable<'borrow, T>>
where
    D: Dims,
{
    fn index_mut(&mut self, index: D) -> &mut Self::Output {
        let idx = self.dimensions().index_of(&index).unwrap();
        // Safety: the layout is compatible and the index is in bounds.
        unsafe {
            self.array
                .data_ptr()
                .cast::<T>()
                .add(idx)
                .as_mut()
                .unwrap_unchecked()
        }
    }
}

impl<'borrow, 'array, 'data, T, M: Mutability>
    InlinePtrArrayAccessor<'borrow, 'array, 'data, T, M>
{
    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D>(&self, index: D) -> Option<&T>
    where
        D: Dims,
    {
        let idx = self.dimensions().index_of(&index).ok()?;
        // Safety: the layout is compatible and the index is in bounds.
        unsafe { self.array.data_ptr().cast::<T>().add(idx).as_ref() }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        let len = self.dimensions().size();
        let data = self.array.data_ptr().cast::<T>();
        // Safety: the layout is compatible and the lifetime is limited.
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn into_slice(self) -> &'borrow [T] {
        let len = self.dimensions().size();
        let data = self.array.data_ptr().cast::<T>();
        // Safety: the layout is compatible and the lifetime is limited.
        unsafe { slice::from_raw_parts(data, len) }
    }
}

impl<'borrow, 'array, 'data, T, M, D> Index<D>
    for InlinePtrArrayAccessor<'borrow, 'array, 'data, T, M>
where
    D: Dims,
    M: Mutability,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        let idx = self.dimensions().index_of(&index).unwrap();
        // Safety: the layout is compatible and the index is in bounds.
        unsafe {
            self.array
                .data_ptr()
                .cast::<T>()
                .add(idx)
                .as_ref()
                .unwrap_unchecked()
        }
    }
}

impl<'borrow, 'array, 'data, M: Mutability> UnionArrayAccessor<'borrow, 'array, 'data, M> {
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
        let elty = self.array.element_type();
        let idx = self.array.dimensions().index_of(&index)?;

        // Safety: the index is in bounds.
        unsafe {
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
        let elty = self.array.element_type();
        let idx = self.array.dimensions().index_of(&index)?;

        // Safety: The index is in bounds and layout compatibility is checked.
        unsafe {
            let tags = jl_array_typetagdata(self.array.unwrap(Private));
            let mut tag = *tags.add(idx) as _;

            if let Some(ty) = nth_union_component(elty, &mut tag) {
                if T::valid_layout(ty) {
                    let offset = idx * self.array.unwrap_non_null(Private).as_ref().elsize as usize;
                    let ptr = self.array.data_ptr().cast::<i8>().add(offset).cast::<T>();
                    return Ok((&*ptr).clone());
                }
                Err(AccessError::InvalidLayout {
                    value_type_str: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                })?
            }

            Err(AccessError::IllegalUnionTag {
                union_type: elty.display_string_or(CANNOT_DISPLAY_TYPE),
                tag: tag as usize,
            })?
        }
    }
}

impl<'borrow, 'array, 'data> UnionArrayAccessor<'borrow, 'array, 'data, Mutable<'borrow, u8>> {
    /// Set the element at index `idx` to `value` with the type `ty`.
    ///
    /// The type `T` must be a valid layout for the value, and `ty` must be a member of the union
    /// of all possible element types.
    pub fn set<T, D>(&mut self, index: D, ty: DataType, value: T) -> JlrsResult<()>
    where
        T: ValidLayout + Clone,
        D: Dims,
    {
        if !T::valid_layout(ty.as_value()) {
            let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type_str })?;
        }

        let mut tag = 0;
        if !find_union_component(self.array.element_type(), ty.as_value(), &mut tag) {
            let element_type_str = self
                .array
                .element_type()
                .display_string_or(CANNOT_DISPLAY_TYPE);
            let value_type_str = ty.display_string_or(CANNOT_DISPLAY_TYPE);
            Err(TypeError::IncompatibleType {
                element_type_str,
                value_type_str,
            })?;
        }

        let idx = self.array.dimensions().index_of(&index)?;
        // Safety: The data can be stored in this array, the tag is updated accordingly.
        unsafe {
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
