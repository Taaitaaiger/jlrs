//! Support for n-dimensional arrays and their dimensions.
//!
//! You will find several structs in this module that can be used to work with Julia arrays from
//! Rust. An [`Array`] is the Julia array itself, and provides methods to (mutably) access the
//! data and copy it to Rust. Accessing array data from Rust when the type of the elements is a
//! union of bits types is not supported, use `Base.getindex` instead.
//!
//! The structs that represent copied or borrowed data can be accessed using an n-dimensional
//! index written as a tuple. For example, if `a` is a three-dimensional array, a single element
//! can be accessed with `a[(row, col, z)]`.
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::layout::{julia_typecheck::JuliaTypecheck, valid_layout::ValidLayout};
use crate::memory::traits::frame::Frame;
use crate::value::datatype::DataType;
use crate::value::Value;
use jl_sys::{jl_array_data, jl_array_eltype, jl_array_t, jl_is_array_type, jl_tparam0};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::ptr::NonNull;

use self::data::{
    inline::{InlineArrayDataMut, UnrestrictedInlineArrayDataMut},
    union::{UnionArrayData, UnionArrayDataMut},
    value::{UnrestrictedValueArrayDataMut, ValueArrayDataMut},
};
use self::dimensions::Dimensions;
use super::union::Union;

pub mod data;
pub mod dimensions;

/// An n-dimensional Julia array. This struct implements [`JuliaTypecheck`] and [`Cast`]. It can
/// be used in combination with [`DataType::is`] and [`Value::is`]; if the check returns `true`
/// the [`Value`] can be cast to `Array`:
///
/// ```
/// # use jlrs::prelude::*;
/// # use jlrs::util::JULIA;
/// # fn main() {
/// # JULIA.with(|j| {
/// # let mut julia = j.borrow_mut();
/// julia.scope(|_global, frame| {
///     let arr = Value::new_array::<f64, _, _, _>(&mut *frame, (3, 3))?;
///     assert!(arr.is::<Array>());
///     assert!(arr.cast::<Array>().is_ok());
///     Ok(())
/// }).unwrap();
/// # });
/// # }
/// ```
///
/// Each element in the backing storage is either stored as a [`Value`] or inline. You can check
/// how the data is stored by calling [`Array::is_value_array`] or [`Array::is_inline_array`].
/// Note that this is not necessarily consistent across different versions of Julia; the array
/// might be value array in Julia 1.0, but an inline array in Julia 1.5. If you want to ensure the
/// data is not stored inline, you should use a mutable struct as the element type. If the data is
/// stored inline, you will need to provide a type with the appropriate layout, the easiest way to
/// create these for types that are not available in jlrs is to use `JlrsReflect.jl`.
///
/// Arrays that contain integers or floats are an example of inline arrays. Their data is stored
/// as an array that contains numbers of the appropriate type, for example an array of `Float32`s
/// in Julia is backed by an an array of `f32`s. The data of these arrays can be accessed with
/// [`Array::inline_data`] and [`Array::inline_data_mut`], and copied from Julia to Rust with
/// [`Array::copy_inline_data`]. In order to call these methods the type of the elements must be
/// provided, arrays that contain numbers can be accessed by providing the appropriate Rust type
/// (eg `f32` for `Float32` and `u64` for `UInt64`).
///
/// If the data isn't inlined each element is stored as a [`Value`]. This data can be accessed
/// using [`Array::value_data`] and [`Array::value_data_mut`] but this is unsafe.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Array<'frame, 'data>(
    NonNull<jl_array_t>,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
);

impl<'frame, 'data> Array<'frame, 'data> {
    pub(crate) unsafe fn wrap(array: *mut jl_array_t) -> Self {
        debug_assert!(!array.is_null());
        Array(NonNull::new_unchecked(array), PhantomData, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_array_t> {
        self.0
    }

    /// Returns the array's dimensions.
    pub fn dimensions(self) -> Dimensions {
        unsafe { Dimensions::from_array(self.inner().as_ptr().cast()) }
    }

    /// Returns the type of this array's elements.
    pub fn element_type(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(jl_array_eltype(self.inner().as_ptr().cast()).cast()) }
    }

    /// Returns `true` if the type of the elements of this array is `T`.
    pub fn contains<T: ValidLayout>(self) -> bool {
        unsafe {
            T::valid_layout(Value::wrap(
                jl_array_eltype(self.inner().as_ptr().cast()).cast(),
            ))
        }
    }

    /// Returns `true` if the type of the elements of this array is `T` and these elements are
    /// stored inline.
    pub fn contains_inline<T: ValidLayout>(self) -> bool {
        self.contains::<T>() && self.is_inline_array()
    }

    /// Returns true if the elements of the array are stored inline.
    pub fn is_inline_array(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).flags.ptrarray() == 0 }
    }

    /// Returns true if the elements of the array are stored inline and the element type is a
    /// union type. In this case the contents of the array can be accessed from Rust with
    /// [`Array::union_array_data`] and [`Array::union_array_data_mut`].
    pub fn is_union_array(self) -> bool {
        self.is_inline_array() && self.element_type().is::<Union>()
    }

    /// Returns true if the elements of the array are stored inline and at least one of the field
    /// of the inlined type is a pointer.
    pub fn has_inlined_pointers(self) -> bool {
        unsafe {
            let flags = (&*self.inner().as_ptr()).flags;
            self.is_inline_array() && flags.hasptr() != 0
        }
    }

    /// Returns true if the elements of the array are stored as [`Value`]s.
    pub fn is_value_array(self) -> bool {
        !self.is_inline_array()
    }

    /// Convert this untyped array into a `TypedArray`.
    pub fn into_typed_array<T>(self) -> JlrsResult<TypedArray<'frame, 'data, T>>
    where
        T: Copy + ValidLayout,
    {
        if self.contains::<T>() {
            unsafe { Ok(TypedArray::wrap(self.inner().as_ptr())) }
        } else {
            Err(JlrsError::WrongType)?
        }
    }

    /// Copy the data of an inline array to Rust. Returns `JlrsError::NotInline` if the data is
    /// not stored inline or `JlrsError::WrongType` if the type of the elements is incorrect.
    pub fn copy_inline_data<T>(self) -> JlrsResult<CopiedArray<T>>
    where
        T: ValidLayout,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType)?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe {
            let jl_data = jl_array_data(self.inner().as_ptr().cast()).cast();
            let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());

            let sz = dimensions.size();
            let mut data = Vec::with_capacity(sz);
            let ptr = data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(jl_data, ptr, sz);
            data.set_len(sz);

            Ok(CopiedArray::new(data, dimensions))
        }
    }

    /// Immutably borrow inline array data, you can borrow data from multiple arrays at the same
    /// time. Returns `JlrsError::NotInline` if the data is not stored inline or
    /// `JlrsError::WrongType` if the type of the elements is incorrect.
    pub fn inline_data<'borrow, 'fr, T, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ArrayData<'borrow, 'fr, T, F>>
    where
        T: ValidLayout,
        F: Frame<'fr>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType)?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe {
            let jl_data = jl_array_data(self.inner().as_ptr().cast()).cast();
            let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
            let data = std::slice::from_raw_parts(jl_data, dimensions.size());
            Ok(ArrayData::new(data, dimensions, frame))
        }
    }

    /// Mutably borrow inline array data, you can mutably borrow a single array at the same time.
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
    pub fn inline_data_mut<'borrow, 'fr, T, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<InlineArrayDataMut<'borrow, 'frame, 'fr, 'data, T, F>>
    where
        'borrow: 'data,
        T: ValidLayout,
        F: Frame<'fr>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType)?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe {
            let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
            Ok(InlineArrayDataMut::new(self, dimensions, frame))
        }
    }

    /// Mutably borrow inline array data without the restriction that only a single array can be
    /// mutably borrowed. It's your responsibility to ensure you don't create multiple mutable
    /// references to the same array data.
    ///
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
    pub unsafe fn unrestricted_inline_data_mut<'borrow, 'fr, T, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedInlineArrayDataMut<'borrow, 'fr, T, F>>
    where
        T: ValidLayout,
        F: Frame<'fr>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType)?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        let jl_data = jl_array_data(self.inner().as_ptr().cast()).cast();
        let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
        let data = std::slice::from_raw_parts_mut(jl_data, dimensions.size());
        Ok(UnrestrictedInlineArrayDataMut::new(data, dimensions, frame))
    }

    /// Immutably borrow the data of this value array, you can borrow data from multiple arrays at
    /// the same time. The values themselves can be mutable, but you can't replace an element with
    /// another value. Returns `JlrsError::Inline` if the data is stored inline.
    ///
    /// Safety: no slot on the GC stack is required to access and use the values in this array,
    /// the GC is aware of the array's data. If the element is changed either from Rust or Julia,
    /// the original value is no longer protected from garbage collection. If you need to keep
    /// using this value you must protect it by calling [`Value::root`].
    ///
    /// [`Value::root`]: crate::value::Value::root
    pub unsafe fn value_data<'borrow, 'fr, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ArrayData<'borrow, 'fr, Value<'frame, 'data>, F>>
    where
        F: Frame<'fr>,
    {
        if !self.is_value_array() {
            Err(JlrsError::Inline)?;
        }

        let jl_data = jl_array_data(self.inner().as_ptr().cast()).cast();
        let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
        let data = std::slice::from_raw_parts(jl_data, dimensions.size());
        Ok(ArrayData::new(data, dimensions, frame))
    }

    /// Mutably borrow the data of this value array, you can mutably borrow a single array at the
    /// same time. Returns `JlrsError::Inline` if the data is stored inline.
    ///
    /// Safety: no slot on the GC stack is required to access and use the values in this array,
    /// the GC is aware of the array's data. If the element is changed either from Rust or Julia,
    /// the original value is no longer protected from garbage collection. If you need to keep
    /// using this value you must protect it by calling [`Value::root`].
    ///
    /// [`Value::root`]: crate::value::Value::root
    pub unsafe fn value_data_mut<'borrow, 'fr, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ValueArrayDataMut<'borrow, 'frame, 'data, 'fr, F>>
    where
        F: Frame<'fr>,
    {
        if !self.is_value_array() {
            Err(JlrsError::Inline)?;
        }

        let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
        Ok(ValueArrayDataMut::new(self, dimensions, frame))
    }

    /// Mutably borrow the data of this value array without the restriction that only a single
    /// array can be mutably borrowed. It's your responsibility to ensure you don't create
    /// multiple mutable references to the same array data. Returns `JlrsError::Inline` if the
    /// data is stored inline.
    ///
    /// Safety: no slot on the GC stack is required to access and use the values in this array,
    /// the GC is aware of the array's data. If the element is changed either from Rust or Julia,
    /// the original value is no longer protected from garbage collection. If you need to keep
    /// using this value you must protect it by calling [`Value::root`].
    ///
    /// [`Value::root`]: crate::value::Value::root
    pub unsafe fn unrestricted_value_data_mut<'borrow, 'fr, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedValueArrayDataMut<'borrow, 'frame, 'data, 'fr, F>>
    where
        F: Frame<'fr>,
    {
        if !self.is_value_array() {
            Err(JlrsError::Inline)?;
        }

        let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
        Ok(UnrestrictedValueArrayDataMut::new(self, dimensions, frame))
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'data> {
        self.into()
    }
}

impl<'frame> Array<'frame, 'static> {
    /// Access the contents of a bits-union array.
    pub fn union_array_data<'borrow, 'fr, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnionArrayData<'borrow, 'frame, 'fr, F>>
    where
        F: Frame<'fr>,
    {
        if self.is_union_array() {
            unsafe {
                let dims = Dimensions::from_array(self.inner().as_ptr().cast());
                Ok(UnionArrayData::new(self, dims, frame))
            }
        } else {
            Err(JlrsError::NotAUnionArray)?
        }
    }

    /// Mutable access the contents of a bits-union array.
    pub fn union_array_data_mut<'borrow, 'fr, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<UnionArrayDataMut<'borrow, 'frame, 'fr, F>>
    where
        F: Frame<'fr>,
    {
        if self.is_union_array() {
            unsafe {
                let dims = Dimensions::from_array(self.inner().as_ptr().cast());
                Ok(UnionArrayDataMut::new(self, dims, frame))
            }
        } else {
            Err(JlrsError::NotAUnionArray)?
        }
    }
}

unsafe impl<'frame, 'data> JuliaTypecheck for Array<'frame, 'data> {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        jl_is_array_type(t.inner().as_ptr().cast())
    }
}

impl<'frame, 'data> Into<Value<'frame, 'data>> for Array<'frame, 'data> {
    fn into(self) -> Value<'frame, 'data> {
        unsafe { Value::wrap_non_null(self.inner().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Array<'frame, 'data> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAnArray)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.inner().as_ptr().cast())
    }
}

unsafe impl<'frame, 'data> ValidLayout for Array<'frame, 'data> {
    unsafe fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<Array>()
        } else if let Ok(ua) = v.cast::<super::union_all::UnionAll>() {
            ua.base_type().assume_reachable_unchecked().is::<Array>()
        } else {
            false
        }
    }
}

/// Exactly the same as [`Array`], except it has an explicit element type `T`.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct TypedArray<'frame, 'data, T>(
    NonNull<jl_array_t>,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
    PhantomData<T>,
)
where
    T: Copy + ValidLayout;

impl<'frame, 'data, T: Copy + ValidLayout> TypedArray<'frame, 'data, T> {
    pub(crate) unsafe fn wrap(array: *mut jl_array_t) -> Self {
        debug_assert!(T::valid_layout(Value::wrap(
            jl_array_eltype(array.cast()).cast()
        )));
        debug_assert!(!array.is_null());
        TypedArray(
            NonNull::new_unchecked(array),
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_array_t> {
        self.0
    }

    /// Returns the array's dimensions.
    pub fn dimensions(self) -> Dimensions {
        unsafe { Dimensions::from_array(self.inner().as_ptr().cast()) }
    }

    /// Returns the type of this array's elements.
    pub fn element_type(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(jl_array_eltype(self.inner().as_ptr().cast()).cast()) }
    }

    /// Returns true if the elements of the array are stored inline.
    pub fn is_inline_array(self) -> bool {
        unsafe { (&*self.inner().as_ptr()).flags.ptrarray() == 0 }
    }

    /// Returns true if the elements of the array are stored inline and at least one of the field
    /// of the inlined type is a pointer.
    pub fn has_inlined_pointers(self) -> bool {
        unsafe {
            let flags = (&*self.inner().as_ptr()).flags;
            self.is_inline_array() && flags.hasptr() != 0
        }
    }

    /// Returns true if the elements of the array are stored as [`Value`]s.
    pub fn is_value_array(self) -> bool {
        !self.is_inline_array()
    }

    /// Copy the data of an inline array to Rust. Returns `JlrsError::NotInline` if the data is
    /// not stored inline or `JlrsError::WrongType` if the type of the elements is incorrect.
    pub fn copy_inline_data(self) -> JlrsResult<CopiedArray<T>> {
        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe {
            let jl_data = jl_array_data(self.inner().as_ptr().cast()).cast();
            let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());

            let sz = dimensions.size();
            let mut data = Vec::with_capacity(sz);
            let ptr = data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(jl_data, ptr, sz);
            data.set_len(sz);

            Ok(CopiedArray::new(data, dimensions))
        }
    }

    /// Immutably borrow inline array data, you can borrow data from multiple arrays at the same
    /// time. Returns `JlrsError::NotInline` if the data is not stored inline or
    /// `JlrsError::WrongType` if the type of the elements is incorrect.
    pub fn inline_data<'borrow, 'fr, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ArrayData<'borrow, 'fr, T, F>>
    where
        F: Frame<'fr>,
    {
        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe {
            let jl_data = jl_array_data(self.inner().as_ptr().cast()).cast();
            let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
            let data = std::slice::from_raw_parts(jl_data, dimensions.size());
            Ok(ArrayData::new(data, dimensions, frame))
        }
    }

    /// Mutably borrow inline array data, you can mutably borrow a single array at the same time.
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
    pub fn inline_data_mut<'borrow, 'fr, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<InlineArrayDataMut<'borrow, 'frame, 'fr, 'data, T, F>>
    where
        'borrow: 'data,
        F: Frame<'fr>,
    {
        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe {
            let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
            Ok(InlineArrayDataMut::new(self.as_array(), dimensions, frame))
        }
    }

    /// Mutably borrow inline array data without the restriction that only a single array can be
    /// mutably borrowed. It's your responsibility to ensure you don't create multiple mutable
    /// references to the same array data.
    ///
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
    pub unsafe fn unrestricted_inline_data_mut<'borrow, 'fr, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedInlineArrayDataMut<'borrow, 'fr, T, F>>
    where
        F: Frame<'fr>,
    {
        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        let jl_data = jl_array_data(self.inner().as_ptr().cast()).cast();
        let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
        let data = std::slice::from_raw_parts_mut(jl_data, dimensions.size());
        Ok(UnrestrictedInlineArrayDataMut::new(data, dimensions, frame))
    }

    /// Immutably borrow the data of this value array, you can borrow data from multiple arrays at
    /// the same time. The values themselves can be mutable, but you can't replace an element with
    /// another value. Returns `JlrsError::Inline` if the data is stored inline.
    ///
    /// Safety: no slot on the GC stack is required to access and use the values in this array,
    /// the GC is aware of the array's data. If the element is changed either from Rust or Julia,
    /// the original value is no longer protected from garbage collection. If you need to keep
    /// using this value you must protect it by calling [`Value::root`].
    ///
    /// [`Value::root`]: crate::value::Value::root
    pub unsafe fn value_data<'borrow, 'fr, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ArrayData<'borrow, 'fr, Value<'frame, 'data>, F>>
    where
        F: Frame<'fr>,
    {
        if !self.is_value_array() {
            Err(JlrsError::Inline)?;
        }

        let jl_data = jl_array_data(self.inner().as_ptr().cast()).cast();
        let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
        let data = std::slice::from_raw_parts(jl_data, dimensions.size());
        Ok(ArrayData::new(data, dimensions, frame))
    }

    /// Mutably borrow the data of this value array, you can mutably borrow a single array at the
    /// same time. Returns `JlrsError::Inline` if the data is stored inline.
    ///
    /// Safety: no slot on the GC stack is required to access and use the values in this array,
    /// the GC is aware of the array's data. If the element is changed either from Rust or Julia,
    /// the original value is no longer protected from garbage collection. If you need to keep
    /// using this value you must protect it by calling [`Value::root`].
    ///
    /// [`Value::root`]: crate::value::Value::root
    pub unsafe fn value_data_mut<'borrow, 'fr, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ValueArrayDataMut<'borrow, 'frame, 'data, 'fr, F>>
    where
        F: Frame<'fr>,
    {
        if !self.is_value_array() {
            Err(JlrsError::Inline)?;
        }

        let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
        Ok(ValueArrayDataMut::new(self.into(), dimensions, frame))
    }

    /// Mutably borrow the data of this value array without the restriction that only a single
    /// array can be mutably borrowed. It's your responsibility to ensure you don't create
    /// multiple mutable references to the same array data. Returns `JlrsError::Inline` if the
    /// data is stored inline.
    ///
    /// Safety: no slot on the GC stack is required to access and use the values in this array,
    /// the GC is aware of the array's data. If the element is changed either from Rust or Julia,
    /// the original value is no longer protected from garbage collection. If you need to keep
    /// using this value you must protect it by calling [`Value::root`].
    ///
    /// [`Value::root`]: crate::value::Value::root
    pub unsafe fn unrestricted_value_data_mut<'borrow, 'fr, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedValueArrayDataMut<'borrow, 'frame, 'data, 'fr, F>>
    where
        F: Frame<'fr>,
    {
        if !self.is_value_array() {
            Err(JlrsError::Inline)?;
        }

        let dimensions = Dimensions::from_array(self.inner().as_ptr().cast());
        Ok(UnrestrictedValueArrayDataMut::new(
            Array::wrap(self.inner().as_ptr()),
            dimensions,
            frame,
        ))
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'data> {
        self.into()
    }

    /// Convert `self` to a `Value`.
    pub fn as_array(self) -> Array<'frame, 'data> {
        unsafe { Array::wrap(self.inner().as_ptr()) }
    }
}

unsafe impl<'frame, 'data, T: Copy + ValidLayout> JuliaTypecheck for TypedArray<'frame, 'data, T> {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        jl_is_array_type(t.inner().as_ptr().cast())
            && T::valid_layout(Value::wrap(jl_tparam0(t.inner().as_ptr()).cast()))
    }
}

impl<'frame, 'data, T: Copy + ValidLayout> Into<Value<'frame, 'data>>
    for TypedArray<'frame, 'data, T>
{
    fn into(self) -> Value<'frame, 'data> {
        unsafe { Value::wrap_non_null(self.inner().cast()) }
    }
}

impl<'frame, 'data, T: Copy + ValidLayout> Into<Array<'frame, 'data>>
    for TypedArray<'frame, 'data, T>
{
    fn into(self) -> Array<'frame, 'data> {
        unsafe { Array::wrap(self.inner().as_ptr()) }
    }
}

unsafe impl<'frame, 'data, T: Copy + ValidLayout> Cast<'frame, 'data>
    for TypedArray<'frame, 'data, T>
{
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAnArray)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.inner().as_ptr().cast())
    }
}

unsafe impl<'frame, 'data, T: Copy + ValidLayout> ValidLayout for TypedArray<'frame, 'data, T> {
    unsafe fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<TypedArray<T>>()
        } else if let Ok(ua) = v.cast::<super::union_all::UnionAll>() {
            ua.base_type()
                .assume_reachable_unchecked()
                .is::<TypedArray<T>>()
        } else {
            false
        }
    }
}

/// An n-dimensional array whose contents have been copied from Julia to Rust. You can create this
/// struct by calling [`Array::copy_inline_data`]. The data has a column-major order and can be
/// indexed with anything that implements `Into<Dimensions>`; see [`Dimensions`] for more
/// information.
#[derive(Debug)]
pub struct CopiedArray<T> {
    data: Vec<T>,
    dimensions: Dimensions,
}

impl<T> CopiedArray<T> {
    pub(crate) fn new(data: Vec<T>, dimensions: Dimensions) -> Self {
        CopiedArray { data, dimensions }
    }

    /// Turn the array into a tuple containing its data in column-major order and its dimensions.
    pub fn splat(self) -> (Vec<T>, Dimensions) {
        (self.data, self.dimensions)
    }

    /// Returns a reference to the element at the given n-dimensional index if the index is valid,
    /// `None` otherwise.
    pub fn get<D: Into<Dimensions>>(&self, idx: D) -> Option<&T> {
        Some(&self.data[self.dimensions.index_of(idx).ok()?])
    }

    /// Returns a mutable reference to the element at the given n-dimensional index if the index
    /// is valid, `None` otherwise.
    pub fn get_mut<D: Into<Dimensions>>(&mut self, idx: D) -> Option<&mut T> {
        Some(&mut self.data[self.dimensions.index_of(idx).ok()?])
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns the array's data as a mutable slice, the data is in column-major order.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Returns a reference to the array's dimensions.
    pub fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }
}

impl<T, D: Into<Dimensions>> Index<D> for CopiedArray<T> {
    type Output = T;
    fn index(&self, idx: D) -> &T {
        &self.data[self.dimensions.index_of(idx).unwrap()]
    }
}

impl<T, D: Into<Dimensions>> IndexMut<D> for CopiedArray<T> {
    fn index_mut(&mut self, idx: D) -> &mut T {
        &mut self.data[self.dimensions.index_of(idx).unwrap()]
    }
}

/// Immutably borrowed array data from Julia. The data has a column-major order and can be indexed
/// with anything that implements `Into<Dimensions>`; see [`Dimensions`] for more information.
pub struct ArrayData<'borrow, 'frame, T, F>
where
    F: Frame<'frame>,
{
    data: &'borrow [T],
    dimensions: Dimensions,
    _notsendsync: PhantomData<*const ()>,
    _borrow: PhantomData<&'borrow F>,
    _frame: PhantomData<&'frame ()>,
}

impl<'borrow, 'frame, T, F> ArrayData<'borrow, 'frame, T, F>
where
    F: Frame<'frame>,
{
    pub(crate) unsafe fn new(data: &'borrow [T], dimensions: Dimensions, _: &'borrow F) -> Self {
        ArrayData {
            data,
            dimensions,
            _notsendsync: PhantomData,
            _borrow: PhantomData,
            _frame: PhantomData,
        }
    }

    /// Get a reference to the value at `index`, or `None` if the index is out of bounds.
    pub fn get<D: Into<Dimensions>>(&self, index: D) -> Option<&T> {
        Some(&self.data[self.dimensions.index_of(index).ok()?])
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn as_slice(&self) -> &[T] {
        self.data
    }

    /// Returns the array's data as a slice, the data is in column-major order.
    pub fn into_slice(self) -> &'borrow [T] {
        self.data
    }

    /// Returns a reference to the array's dimensions.
    pub fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }
}

impl<'borrow, 'frame, T, D, F> Index<D> for ArrayData<'borrow, 'frame, T, F>
where
    D: Into<Dimensions>,
    F: Frame<'frame>,
{
    type Output = T;
    fn index(&self, index: D) -> &Self::Output {
        &self.data[self.dimensions.index_of(index).unwrap()]
    }
}
