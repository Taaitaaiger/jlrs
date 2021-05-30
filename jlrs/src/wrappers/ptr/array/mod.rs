//! Wrappers for `Core.Array`, support for accessing n-dimensional Julia arrays from Rust.
//!
//! You will find two wrappers in this module that can be used to work with Julia arrays from
//! Rust. An [`Array`] is the Julia array itself, [`TypedArray`] is also available which can be
//! used if the element type implements [`ValidLayout`].
//!
//! How the contents of the array must be accessed from Rust depends on the type of the elements.
//! [`Array`] provides methods to (mutably) access their contents for all three possible
//! "layouts": inline, value, and bits unions.
//!
//! The [`Dims`] trait is available to work with n-dimensional indexes for these arrays. This
//! trait is implemented for tuples for arrays with four or fewer dimensions, and for
//! all arrays and array slices in Rust.

use crate::{
    error::{JlrsError, JlrsResult},
    layout::{typecheck::Typecheck, valid_layout::ValidLayout},
    memory::traits::frame::Frame,
    private::Private,
    wrappers::ptr::{
        array::{
            data::{
                copied::CopiedArray,
                inline::{InlineArrayData, InlineArrayDataMut, UnrestrictedInlineArrayDataMut},
                union::{UnionArrayData, UnionArrayDataMut, UnresistrictedUnionArrayDataMut},
                value::{UnrestrictedValueArrayDataMut, ValueArrayData, ValueArrayDataMut},
            },
            dimensions::{ArrayDimensions, Dims},
        },
        datatype::DataType,
        private::Wrapper as WrapperPriv,
        union::Union,
        value::Value,
        Wrapper,
    },
};
use jl_sys::{jl_array_data, jl_array_eltype, jl_array_t, jl_is_array_type, jl_tparam0};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

pub mod data;
pub mod dimensions;

/// An n-dimensional Julia array. It can be used in combination with [`DataType::is`] and
/// [`Value::is`], if the check returns `true` the [`Value`] can be cast to `Array`:
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
/// Each element in the backing storage is either stored as a [`Value`] or inline. If the inline
/// data is a bits union, the flag indicating the active variant is stored separately from the
/// elements. You can check how the data is stored by calling [`Array::is_value_array`],
/// [`Array::is_inline_array`], or [`Array::is_union_array`].
///
/// Arrays that contain integers or floats are an example of inline arrays. Their data is stored
/// as an array that contains numbers of the appropriate type, for example an array of `Float32`s
/// in Julia is backed by an an array of `f32`s. The data in these arrays can be accessed with
/// [`Array::inline_data`] and [`Array::inline_data_mut`], and copied from Julia to Rust with
/// [`Array::copy_inline_data`]. In order to call these methods the type of the elements must be
/// provided, this type must implement [`ValidLayout`] to ensure the layouts in Rust and Julia are
/// compatible.
///
/// If the data isn't inlined each element is stored as a [`Value`]. This data can be accessed
/// using [`Array::value_data`] and [`Array::value_data_mut`].
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Array<'scope, 'data>(
    NonNull<jl_array_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

impl<'scope, 'data> Debug for Array<'scope, 'data> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // TODO: correcly format type if the element type is not a DataType.
        f.write_fmt(format_args!(
            "Array<{}, {}>",
            self.element_type()
                .cast::<DataType>()
                .map(|dt| dt.name())
                .unwrap_or("<Not a DataType>"),
            self.dimensions().n_dimensions()
        ))
    }
}

impl<'scope, 'data> Array<'scope, 'data> {
    /// Returns the array's dimensions.
    pub fn dimensions(self) -> ArrayDimensions<'scope> {
        unsafe { ArrayDimensions::new(self) }
    }

    /// Returns the type of this array's elements.
    pub fn element_type(self) -> Value<'scope, 'static> {
        unsafe { Value::wrap(jl_array_eltype(self.unwrap(Private).cast()).cast(), Private) }
    }

    /// Returns `true` if the layout of the elements is compatible with `T`.
    pub fn contains<T: ValidLayout>(self) -> bool {
        unsafe {
            T::valid_layout(Value::wrap(
                jl_array_eltype(self.unwrap(Private).cast()).cast(),
                Private,
            ))
        }
    }

    /// Returns `true` if the layout of the elements is compatible with `T` and these elements are
    /// stored inline.
    pub fn contains_inline<T: ValidLayout>(self) -> bool {
        self.contains::<T>() && self.is_inline_array()
    }

    /// Returns true if the elements of the array are stored inline.
    pub fn is_inline_array(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().flags.ptrarray() == 0 }
    }

    /// Returns true if the elements of the array are stored inline and the element type is a
    /// union type. In this case the contents of the array can be accessed from Rust with
    /// [`Array::union_array_data`] and [`Array::union_array_data_mut`].
    pub fn is_union_array(self) -> bool {
        self.is_inline_array() && self.element_type().is::<Union>()
    }

    /// Returns true if the elements of the array are stored inline and at least one of the fields
    /// of the inlined type is a pointer.
    pub fn has_inlined_pointers(self) -> bool {
        unsafe {
            let flags = self.unwrap_non_null(Private).as_ref().flags;
            self.is_inline_array() && flags.hasptr() != 0
        }
    }

    /// Returns true if the elements of the array are stored as [`Value`]s.
    pub fn is_value_array(self) -> bool {
        !self.is_inline_array()
    }

    /// Convert this untyped array to a [`TypedArray`].
    pub fn as_typed_array<T>(self) -> JlrsResult<TypedArray<'scope, 'data, T>>
    where
        T: Copy + ValidLayout + Debug,
    {
        if self.contains::<T>() {
            unsafe {
                Ok(TypedArray::wrap_non_null(
                    self.unwrap_non_null(Private),
                    Private,
                ))
            }
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
            let jl_data = jl_array_data(self.unwrap(Private).cast()).cast();
            let dimensions = self.dimensions().into_dimensions();

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
    pub fn inline_data<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<InlineArrayData<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
        F: Frame<'frame>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType)?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe { Ok(InlineArrayData::new(self, frame)) }
    }

    /// Mutably borrow inline array data, you can mutably borrow a single array at a time. Returns
    /// `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType` if the
    /// type of the elements is incorrect.
    pub fn inline_data_mut<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<InlineArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        'borrow: 'data,
        T: ValidLayout,
        F: Frame<'frame>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType)?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe { Ok(InlineArrayDataMut::new(self, frame)) }
    }

    /// Mutably borrow inline array data without the restriction that only a single array can be
    /// mutably borrowed. It's your responsibility to ensure you don't create multiple mutable
    /// references to the same array data.
    ///
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
    pub unsafe fn unrestricted_inline_data_mut<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedInlineArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
        F: Frame<'frame>,
    {
        if !self.contains::<T>() {
            Err(JlrsError::WrongType)?;
        }

        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        Ok(UnrestrictedInlineArrayDataMut::new(self, frame))
    }

    /// Immutably borrow the data of this array of values, you can borrow data from multiple
    /// arrays at the same time. The values themselves can be mutable, but you can't replace an
    /// element with another value. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn value_data<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ValueArrayData<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        unsafe {
            if !self.is_value_array() {
                Err(JlrsError::Inline)?;
            }

            Ok(ValueArrayData::new(self, frame))
        }
    }

    /// Immutably borrow the data of this array of wrappers, you can borrow data from multiple
    /// arrays at the same time. The values themselves can be mutable, but you can't replace an
    /// element with another value. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn wrapper_data<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ValueArrayData<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
        T: Wrapper<'scope, 'data>,
    {
        unsafe {
            if !self.contains::<T>() {
                Err(JlrsError::WrongType)?;
            }

            Ok(ValueArrayData::new(self, frame))
        }
    }

    /// Mutably borrow the data of this array of values, you can mutably borrow a single array at
    /// the same time. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn value_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ValueArrayDataMut<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        unsafe {
            if !self.is_value_array() {
                Err(JlrsError::Inline)?;
            }

            Ok(ValueArrayDataMut::new(self, frame))
        }
    }

    /// Mutably borrow the data of this array of wrappers, you can mutably borrow a single array
    /// at the same time. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn wrapper_data_mut<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ValueArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
        T: Wrapper<'scope, 'data>,
    {
        unsafe {
            if !self.contains::<T>() {
                Err(JlrsError::WrongType)?;
            }

            Ok(ValueArrayDataMut::new(self, frame))
        }
    }

    /// Mutably borrow the data of this array of values without the restriction that only a single
    /// array can be mutably borrowed. It's your responsibility to ensure you don't create
    /// multiple mutable references to the same array data. Returns `JlrsError::Inline` if the
    /// data is stored inline.
    pub unsafe fn unrestricted_value_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedValueArrayDataMut<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        if !self.is_value_array() {
            Err(JlrsError::Inline)?;
        }

        Ok(UnrestrictedValueArrayDataMut::new(self, frame))
    }

    /// Mutably borrow the data of this array of wrappers without the restriction that only a
    /// single array can be mutably borrowed. It's your responsibility to ensure you don't create
    /// multiple mutable references to the same array data. Returns `JlrsError::Inline` if the
    /// data is stored inline.
    pub unsafe fn unrestricted_wrapper_data_mut<'borrow, 'frame, T, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedValueArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
        T: Wrapper<'scope, 'data>,
    {
        if !self.is_value_array() {
            Err(JlrsError::WrongType)?;
        }

        Ok(UnrestrictedValueArrayDataMut::new(self, frame))
    }
}

impl<'scope> Array<'scope, 'static> {
    /// Access the contents of a bits-union array.
    pub fn union_data<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnionArrayData<'borrow, 'scope>>
    where
        F: Frame<'frame>,
    {
        if self.is_union_array() {
            Ok(UnionArrayData::new(self, frame))
        } else {
            Err(JlrsError::NotAUnionArray)?
        }
    }

    /// Mutably borrow the data of this array of bits-unions, you can mutably borrow a single
    /// array at a time.
    pub fn union_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<UnionArrayDataMut<'borrow, 'scope>>
    where
        F: Frame<'frame>,
    {
        if self.is_union_array() {
            Ok(UnionArrayDataMut::new(self, frame))
        } else {
            Err(JlrsError::NotAUnionArray)?
        }
    }

    /// Mutably borrow the data of this array of bits-unions without the restriction that only a
    /// single array can be mutably borrowed. It's your responsibility to ensure you don't create
    /// multiple mutable references to the same array data.
    pub unsafe fn unrestricted_union_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnresistrictedUnionArrayDataMut<'borrow, 'scope>>
    where
        F: Frame<'frame>,
    {
        if self.is_union_array() {
            Ok(UnresistrictedUnionArrayDataMut::new(self, frame))
        } else {
            Err(JlrsError::NotAUnionArray)?
        }
    }
}

unsafe impl<'scope, 'data> Typecheck for Array<'scope, 'data> {
    unsafe fn typecheck(t: DataType) -> bool {
        jl_is_array_type(t.unwrap(Private).cast())
    }
}

unsafe impl<'scope, 'data> ValidLayout for Array<'scope, 'data> {
    unsafe fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<Array>()
        } else if let Ok(ua) = v.cast::<super::union_all::UnionAll>() {
            ua.base_type().wrapper_unchecked().is::<Array>()
        } else {
            false
        }
    }
}

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Array<'scope, 'data> {
    type Internal = jl_array_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}

/// Exactly the same as [`Array`], except it has an explicit element type `T`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypedArray<'scope, 'data, T>(
    NonNull<jl_array_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
    PhantomData<T>,
)
where
    T: Copy + ValidLayout + Debug;

impl<'scope, 'data, T: Debug + Copy + ValidLayout + Debug> Debug for TypedArray<'scope, 'data, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // TODO: correcly format type if the element type is not a DataType.
        f.write_fmt(format_args!(
            "Array<{}, {}>",
            self.element_type()
                .cast::<DataType>()
                .map(|dt| dt.name())
                .unwrap_or("<Not a DataType>"),
            self.dimensions().n_dimensions()
        ))
    }
}

impl<'scope, 'data, T: Copy + ValidLayout + Debug> TypedArray<'scope, 'data, T> {
    /// Returns the array's dimensions.
    pub fn dimensions(&self) -> ArrayDimensions<'scope> {
        unsafe { ArrayDimensions::new(self.as_array()) }
    }

    /// Returns the type of this array's elements.
    pub fn element_type(self) -> Value<'scope, 'static> {
        unsafe { Value::wrap(jl_array_eltype(self.unwrap(Private).cast()).cast(), Private) }
    }

    /// Returns true if the elements of the array are stored inline.
    pub fn is_inline_array(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().flags.ptrarray() == 0 }
    }

    /// Returns true if the elements of the array are stored inline and at least one of the field
    /// of the inlined type is a pointer.
    pub fn has_inlined_pointers(self) -> bool {
        unsafe {
            let flags = self.unwrap_non_null(Private).as_ref().flags;
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
            let jl_data = jl_array_data(self.unwrap(Private).cast()).cast();
            let dimensions = self.dimensions().into_dimensions();

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
    pub fn inline_data<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<InlineArrayData<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
    {
        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe { Ok(InlineArrayData::new(self.as_array(), frame)) }
    }

    /// Mutably borrow inline array data, you can mutably borrow a single array at the same time.
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
    pub fn inline_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<InlineArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        'borrow: 'data,
        F: Frame<'frame>,
    {
        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        unsafe { Ok(InlineArrayDataMut::new(self.as_array(), frame)) }
    }

    /// Mutably borrow inline array data without the restriction that only a single array can be
    /// mutably borrowed. It's your responsibility to ensure you don't create multiple mutable
    /// references to the same array data.
    ///
    /// Returns `JlrsError::NotInline` if the data is not stored inline or `JlrsError::WrongType`
    /// if the type of the elements is incorrect.
    pub unsafe fn unrestricted_inline_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedInlineArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
    {
        if !self.is_inline_array() {
            Err(JlrsError::NotInline)?;
        }

        Ok(UnrestrictedInlineArrayDataMut::new(self.as_array(), frame))
    }

    /// Convert `self` to `Array`.
    pub fn as_array(self) -> Array<'scope, 'data> {
        unsafe { Array::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }
}

impl<'scope, 'data, T: Wrapper<'scope, 'data>> TypedArray<'scope, 'data, T> {
    /// Immutably borrow the data of this array of values, you can borrow data from multiple
    /// arrays at the same time. The values themselves can be mutable, but you can't replace an
    /// element with another value. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn value_data<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ValueArrayData<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        unsafe { Ok(ValueArrayData::new(self.as_array(), frame)) }
    }

    /// Immutably borrow the data of this array of wrappers, you can borrow data from multiple
    /// arrays at the same time. The values themselves can be mutable, but you can't replace an
    /// element with another value. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn wrapper_data<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<ValueArrayData<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
    {
        unsafe { Ok(ValueArrayData::new(self.as_array(), frame)) }
    }

    /// Mutably borrow the data of this array of values, you can mutably borrow a single array at
    /// the same time. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn value_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ValueArrayDataMut<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        unsafe { Ok(ValueArrayDataMut::new(self.as_array(), frame)) }
    }

    /// Mutably borrow the data of this array of wrappers, you can mutably borrow a single array
    /// at the same time. Returns `JlrsError::Inline` if the data is stored inline.
    pub fn wrapper_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow mut F,
    ) -> JlrsResult<ValueArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
    {
        unsafe { Ok(ValueArrayDataMut::new(self.as_array(), frame)) }
    }

    /// Mutably borrow the data of this array of values without the restriction that only a single
    /// array can be mutably borrowed. It's your responsibility to ensure you don't create
    /// multiple mutable references to the same array data. Returns `JlrsError::Inline` if the
    /// data is stored inline.
    pub unsafe fn unrestricted_value_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedValueArrayDataMut<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        Ok(UnrestrictedValueArrayDataMut::new(
            Array::wrap_non_null(self.unwrap_non_null(Private), Private),
            frame,
        ))
    }

    /// Mutably borrow the data of this array of wrappers without the restriction that only a
    /// single array can be mutably borrowed. It's your responsibility to ensure you don't create
    /// multiple mutable references to the same array data. Returns `JlrsError::Inline` if the
    /// data is stored inline.
    pub unsafe fn unrestricted_wrapper_data_mut<'borrow, 'frame, F>(
        self,
        frame: &'borrow F,
    ) -> JlrsResult<UnrestrictedValueArrayDataMut<'borrow, 'scope, 'data, T>>
    where
        F: Frame<'frame>,
    {
        Ok(UnrestrictedValueArrayDataMut::new(
            Array::wrap_non_null(self.unwrap_non_null(Private), Private),
            frame,
        ))
    }
}

unsafe impl<'scope, 'data, T: Copy + ValidLayout + Debug> Typecheck
    for TypedArray<'scope, 'data, T>
{
    unsafe fn typecheck(t: DataType) -> bool {
        jl_is_array_type(t.unwrap(Private).cast())
            && T::valid_layout(Value::wrap(jl_tparam0(t.unwrap(Private)).cast(), Private))
    }
}

unsafe impl<'scope, 'data, T: Copy + ValidLayout + Debug> ValidLayout
    for TypedArray<'scope, 'data, T>
{
    unsafe fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<TypedArray<T>>()
        } else if let Ok(ua) = v.cast::<super::union_all::UnionAll>() {
            ua.base_type().wrapper_unchecked().is::<TypedArray<T>>()
        } else {
            false
        }
    }
}

impl<'scope, 'data, T: Copy + ValidLayout + Debug> WrapperPriv<'scope, 'data>
    for TypedArray<'scope, 'data, T>
{
    type Internal = jl_array_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
