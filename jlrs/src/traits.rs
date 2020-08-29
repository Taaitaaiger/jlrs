//! All traits used by this crate.
//!
//! Most of these traits are intended for internal use only and you should never manually
//! implement them for your own types nor call any of their trait methods directly.
//!
//! The one major exception is the [`Frame`] trait. This trait is implemented by the two frame
//! types that are provided, [`StaticFrame`] and [`DynamicFrame`] which are used to ensure the
//! garbage collector doesn't drop the data that's used from Rust. It provides the common
//! functionality these frame types offer.
//!
//! Two of the traits in this module are available as custom derive traits, [`JuliaStruct`] and
//! [`IntoJulia`], which can be used to map a struct between Julia and Rust. Deriving the first
//! will implement [`JuliaType`], [`JuliaTypecheck`], [`ValidLayout`], and [`Cast`], which will let you
//! safely access the raw contents of a value; [`IntoJulia`] can be derived for bits types and lets
//! you create new instances of that type using [`Value::new`]. While it's possible to manually
//! implement and annotate these mapping structs, you should use `JlrsReflect.jl` which can
//! generate these structs for you. If you do want to do this manually, see the documentation of
//! [`JuliaStruct`] for instructions.
//!
//! [`Frame`]: trait.Frame.html
//! [`StaticFrame`]: ../frame/struct.StaticFrame.html
//! [`DynamicFrame`]: ../frame/struct.DynamicFrame.html
//! [`Value::new`]: ../value/struct.Value.html#method.new
//! [`Value::cast`]: ../value/struct.Value.html#method.cast
//! [`JuliaStruct`]: trait.JuliaStruct.html
//! [`JuliaType`]: trait.JuliaType.html
//! [`Cast`]: trait.Cast.html
//! [`ValidLayout`]: trait.ValidLayout.html
//! [`IntoJulia`]: trait.IntoJulia.html
//! [`JuliaTypecheck`]: trait.JuliaTypecheck.html
//! [`Value::is`]: ../value/struct.Value.html#method.is
//! [`DataType::is`]: ../value/datatype/struct.DataType.html#method.is

use crate::error::{AllocError, JlrsError, JlrsResult};
use crate::frame::{DynamicFrame, NullFrame, Output, StaticFrame};
use crate::value::datatype::DataType;
use crate::value::string::JuliaString;
use crate::value::symbol::Symbol;
use crate::value::Value;
use jl_sys::{
    jl_bool_type, jl_box_bool, jl_box_char, jl_box_float32, jl_box_float64, jl_box_int16,
    jl_box_int32, jl_box_int64, jl_box_int8, jl_box_uint16, jl_box_uint32, jl_box_uint64,
    jl_box_uint8, jl_box_voidpointer, jl_char_type, jl_datatype_t, jl_float32_type,
    jl_float64_type, jl_int16_type, jl_int32_type, jl_int64_type, jl_int8_type, jl_pchar_to_string,
    jl_string_data, jl_string_len, jl_uint16_type, jl_uint32_type, jl_uint64_type, jl_uint8_type,
    jl_unbox_float32, jl_unbox_float64, jl_unbox_int16, jl_unbox_int32, jl_unbox_int64,
    jl_unbox_int8, jl_unbox_uint16, jl_unbox_uint32, jl_unbox_uint64, jl_unbox_uint8,
    jl_unbox_voidpointer, jl_value_t, jl_voidpointer_type,
};
use std::borrow::Cow;
use std::ffi::c_void;

macro_rules! p {
    ($trait:ident, $type:ty, $($bounds:tt)+) => {
        unsafe impl<$($bounds)+> $trait for $type {}
    };
    ($trait:ident, $type:ty) => {
        unsafe impl $trait for $type {}
    };
}

/// Trait implemented by types that can be converted to a temporary [`Symbol`].
///
/// [`Symbol`]: ../value/symbol/struct.Symbol.html
pub unsafe trait TemporarySymbol: private::TemporarySymbol {}

/// Trait implemented as part of `JuliaStruct` that is used to verify this type has the same
/// layout as the Julia value.
pub unsafe trait ValidLayout {
    #[doc(hidden)]
    // NB: the type is passed as a value to account for DataTypes, UnionAlls and Unions.
    unsafe fn valid_layout(ty: Value) -> bool;
}

/// Trait implemented by the aligning structs, which ensure bits unions are properly aligned.
/// Used in combination with `BitsUnion` and `Flag` to ensure bits unions are inserted correctly.
pub unsafe trait Align {
    /// The alignment in bytes
    const ALIGNMENT: usize;
}

/// Trait implemented by structs that can contain a bits union.
/// Used in combination with `Align` and `Flag` to ensure bits unions are inserted correctly.
pub unsafe trait BitsUnion {}

/// Trait implemented by structs that can contain the flag of a bits union.
/// Used in combination with `Align` and `BitsUnion` to ensure bits unions are inserted correctly.
pub unsafe trait Flag {}

unsafe impl Flag for u8 {}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_valid_layout {
    ($type:ty, $($lt:lifetime),+) => {
        unsafe impl<$($lt),+> $crate::traits::ValidLayout for $type {
            unsafe fn valid_layout(v: $crate::value::Value) -> bool {
                if let Ok(dt) =  v.cast::<$crate::value::datatype::DataType>() {
                    dt.is::<$type>()
                } else {
                    false
                }
            }
        }
    };
    ($t:ty) => {
        unsafe impl $crate::traits::ValidLayout for $t {
            unsafe fn valid_layout(v: $crate::value::Value) -> bool {
                if let Ok(dt) =  v.cast::<$crate::value::datatype::DataType>() {
                    dt.is::<$t>()
                } else {
                    false
                }
            }
        }
    }
}

/// Trait implemented by types that can be converted to a Julia value in combination with
/// [`Value::new`]. This trait can be derived for custom bits types that implement
/// `JuliaStruct`.
///
/// [`Value::new`]: ../value/struct.Value.html#method.new
pub unsafe trait IntoJulia {
    #[doc(hidden)]
    unsafe fn into_julia(&self) -> *mut jl_value_t;
}

/// Trait implemented by types that have an associated type in Julia.
pub unsafe trait JuliaType {
    #[doc(hidden)]
    unsafe fn julia_type() -> *mut jl_datatype_t;
}

/// This trait can be derived in order to provide a mapping between a type in Julia and one in
/// Rust. When this trait is derived, the following traits are implemented:
///
/// - [`JuliaType`]
/// - [`JuliaTypecheck`]
/// - [`ValidLayout`]
/// - [`Cast`]
///
/// With these traits implemented you can use [`Value::cast`] with this custom type.
///
/// Rather than manually implement the appropriate structs, you should use `JlrsReflect.jl` to
/// generate them for you.  If you do choose to implement this trait manually, the following rules
/// apply.
///
/// First, the struct must be annotated with `#[repr(C)]` to ensure the compiler won't change the
/// layout. Second, the struct must be annotated with `#[jlrs(julia_type = "Path.To.Type")]` where
/// the path provides the full name of the type, eg the path for a struct named`Bar` in the module
/// `Foo` which is a submodule of `Main` is `Main.Foo.Bar`. When this type is used, it must be
/// available at that location. This path must not contain any type parameters.
///
/// Struct have fields and these fields have types. The type can belong to one of the following
/// classes:
///  - DataType
///  - UnionAll
///  - Union
///  - TypeVar
///
/// If the field type is a DataType the field will either be allocated inline or stored as a
/// `Value`. If it's allocated inline, a valid binding for that field must be used. In some cases,
/// for example a field that contains a `Module`, that type can be used as a specialized type.
/// Many of the types defined in the submodules of `value` can be used this way.
///
/// Special care must be taken if the field type is a tuple type. Unlike other types, tuples are
/// covariant in the parameters. This means that a tuple like `Tuple{Int32, Int64}` is a subtype
/// of `Tuple{Int32, Real}`. As a result, a tuple type can only be instantiated if all of its
/// fields are concrete types. If the field type is a concrete tuple type, it is stored inline and
/// can be represented by the appropriate type from the `tuple` module, otherwise it will not be
/// stored inline and a `Value` must be used instead.
///
/// `UnionAll`s are straightforward, they're never allocated inline and must always be mapped to a
/// `Value`.
///
/// Similar to tuples, unions can have two representation depending on the type parameters. If all
/// types are pointer-free, the bits union optimization will apply. Otherwise it is stored as a
/// `Value`.
///
/// The bits union optimization is not straightforward to map to Rust. In fact, three fields are
/// required. Unlike normal structs the size of a bits union field doesn't have to be an integer
/// multiple of its alignment; it will have the alignment of the variant with the largest alignment
/// and is as large as the largest possible variant. Additionally, there will be another `u8` that
/// is used as a flag to indicate the active variant.
///
/// The first field is the correct zero-sized `Align#`-type defined in the `union` module. The
/// second a `BitsUnion` from that same module, its type parameter must be an array of
/// `MaybeUninit<u8>`s with the appropriate numbber of elements. Finally, a `u8` must be used as
/// a flag. In order for the derive macro to handle these fields correctly, they must be annotated
/// with `#[jlrs(bits_union_align)]`, `#[jlrs(bits_union)]`, and `#[jlrs(bits_union_flag)]`
/// respectively.
///
/// Finally, a `TypeVar` field will be mapped to a type parameter in Rust. A parameter that
/// doesn't affect the layout must be elided. The type parameter must implement both `ValidLayout`
/// and `Copy`.
///
/// [`JuliaType`]: trait.JuliaType.html
/// [`JuliaTypecheck`]: trait.JuliaTypecheck.html
/// [`ValidLayout`]: trait.ValidLayout.html
/// [`Cast`]: trait.Cast.html
/// [`Value::cast`]: ../value/struct.Value.html#method.cast
pub unsafe trait JuliaStruct: Copy {}

/// This trait is used in combination with [`Value::is`] and [`DataType::is`]; types that
/// implement this trait can be used to check many properties of a Julia `DataType`.
///
/// This trait is implemented for a few types that implement [`JuliaType ], eg `String`,
/// [`Array`], and `u8`. In these cases, if the check returns `true` the value can be successfully
/// cast to that type with [`Value::cast`].
///
/// [`DataType::is`]: ../value/datatype/struct.DataType.html#method.is
/// [`Value::is`]: ../value/struct.Value.html#method.is
/// [`JuliaType`]: trait.JuliaType.html
/// [`Array`]: ../value/array/struct.Array.html
/// [`Value::cast`]: ../value/struct.Value.html#method.cast
pub unsafe trait JuliaTypecheck {
    #[doc(hidden)]
    unsafe fn julia_typecheck(t: DataType) -> bool;
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_julia_typecheck {
    ($type:ty, $jl_type:expr, $($lt:lifetime),+) => {
        unsafe impl<$($lt),+> crate::traits::JuliaTypecheck for $type {
            unsafe fn julia_typecheck(t: $crate::value::datatype::DataType) -> bool {
                t.ptr() == $jl_type
            }
        }
    };
    ($type:ty, $jl_type:expr) => {
        unsafe impl crate::traits::JuliaTypecheck for $type {
            unsafe fn julia_typecheck(t: $crate::value::datatype::DataType) -> bool {
                t.ptr() == $jl_type
            }
        }
    };
    ($type:ty) => {
        unsafe impl crate::traits::JuliaTypecheck for $type {
            unsafe fn julia_typecheck(t: crate::value::datatype::DataType) -> bool {
                t.ptr() == <$type as $crate::traits::JuliaType>::julia_type()
            }
        }
    };
}

impl_julia_typecheck!(i8);
impl_julia_typecheck!(i16);
impl_julia_typecheck!(i32);
impl_julia_typecheck!(i64);
impl_julia_typecheck!(isize);
impl_julia_typecheck!(u8);
impl_julia_typecheck!(u16);
impl_julia_typecheck!(u32);
impl_julia_typecheck!(u64);
impl_julia_typecheck!(usize);
impl_julia_typecheck!(f32);
impl_julia_typecheck!(f64);
impl_julia_typecheck!(bool);
impl_julia_typecheck!(char);
impl_julia_typecheck!(*mut c_void);

/// This trait is implemented by types that a [`Value`] can be converted into by calling
/// [`Value::cast`]. This includes types like `String`, [`Array`], and `u8`.
///
/// [`Value`]: ../value/struct.Value.html
/// [`Value::cast`]: ../value/struct.Value.html#method.cast
/// [`Array`]: ../value/array/struct.Array.html
pub unsafe trait Cast<'frame, 'data> {
    type Output;
    #[doc(hidden)]
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output>;

    #[doc(hidden)]
    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output;
}

/// Functionality shared by [`StaticFrame`] and [`DynamicFrame`]. These structs let you protect
/// data from garbage collection. The lifetime of a frame is assigned to the values and outputs
/// that are created using that frame. After a frame is dropped, these items are no longer
/// protected and cannot be used.
///
/// If you need the result of a function call to be valid outside the frame where it is called,
/// you can call `Frame::output` to create an [`Output`] and use [`Value::with_output`] to use the
/// output to protect the value rather than the current frame. The result will share the output's
/// lifetime so it can be used until the output's frame goes out of scope.
///
/// [`StaticFrame`]: ../frame/struct.StaticFrame.html
/// [`DynamicFrame`]: ../frame/struct.DynamicFrame.html
/// [`Module`]: ../module/struct.Module.html
/// [`Julia::frame`]: ../struct.Julia.html#method.frame
/// [`Julia::dynamic_frame`]: ../struct.Julia.html#method.dynamic_frame
/// [`Output`]: ../frame/struct.Output.html
/// [`Value::with_output`]: ../value/struct.Value.html#method.with_output
pub trait Frame<'frame>: private::Frame<'frame> {
    /// Create a `StaticFrame` that can hold `capacity` values, and call the given closure.
    /// Returns the result of this closure, or an error if the new frame can't be created
    /// because there's not enough space on the GC stack. The number of required slots on the
    /// stack is `capacity + 2`.
    ///
    /// Returns an error if there is not enough space on the stack.
    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T>;

    /// Create a `DynamicFrame` and call the given closure.  Returns the result of this closure,
    /// or an error if the new frame can't be created because the stack is too small. The number
    /// of required slots on the stack is `2`.
    ///
    /// Returns an error if there is not enough space on the stack.
    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<T>;

    /// Returns a new `Output`, this takes one slot on the GC stack. A function that uses this
    /// output will not use a slot on the GC stack, but the one associated with this output. This
    /// extends the lifetime of that value to be valid until the frame that created the output
    /// goes out of scope.
    ///
    /// Returns an error if there is not enough space on the stack.
    fn output(&mut self) -> JlrsResult<Output<'frame>>;

    /// Returns the number of values belonging to this frame.
    fn size(&self) -> usize;

    #[doc(hidden)]
    // Exists for debugging purposes, prints the contents of the GC stack.
    fn print_memory(&self);
}

p!(TemporarySymbol, String);
p!(TemporarySymbol, &dyn AsRef<str>);
p!(TemporarySymbol, &'a str, 'a);
p!(TemporarySymbol, Cow<'a, str>, 'a);
p!(TemporarySymbol, Symbol<'s>, 's);
p!(TemporarySymbol, JuliaString<'frame>, 'frame);

impl_valid_layout!(bool);
impl_valid_layout!(char);
impl_valid_layout!(i8);
impl_valid_layout!(i16);
impl_valid_layout!(i32);
impl_valid_layout!(i64);
impl_valid_layout!(isize);
impl_valid_layout!(u8);
impl_valid_layout!(u16);
impl_valid_layout!(u32);
impl_valid_layout!(u64);
impl_valid_layout!(usize);
impl_valid_layout!(f32);
impl_valid_layout!(f64);

macro_rules! impl_into_julia {
    ($type:ty, $boxer:ident) => {
        unsafe impl IntoJulia for $type {
            unsafe fn into_julia(&self) -> *mut jl_value_t {
                $boxer(*self)
            }
        }
    };
    ($type:ty, $as:ty, $boxer:ident) => {
        unsafe impl IntoJulia for $type {
            unsafe fn into_julia(&self) -> *mut jl_value_t {
                $boxer(*self as $as)
            }
        }
    };
}

impl_into_julia!(bool, i8, jl_box_bool);
impl_into_julia!(char, u32, jl_box_char);
impl_into_julia!(u8, jl_box_uint8);
impl_into_julia!(u16, jl_box_uint16);
impl_into_julia!(u32, jl_box_uint32);
impl_into_julia!(u64, jl_box_uint64);
impl_into_julia!(i8, jl_box_int8);
impl_into_julia!(i16, jl_box_int16);
impl_into_julia!(i32, jl_box_int32);
impl_into_julia!(i64, jl_box_int64);
impl_into_julia!(f32, jl_box_float32);
impl_into_julia!(f64, jl_box_float64);
impl_into_julia!(*mut c_void, jl_box_voidpointer);

#[cfg(not(target_pointer_width = "64"))]
unsafe impl IntoJulia for usize {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        jl_box_uint32(*self as u32)
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl IntoJulia for usize {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        jl_box_uint64(*self as u64)
    }
}

#[cfg(not(target_pointer_width = "64"))]
unsafe impl IntoJulia for isize {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        jl_box_int32(*self as i32)
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl IntoJulia for isize {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        jl_box_int64(*self as i64)
    }
}

unsafe impl<'a> IntoJulia for &'a str {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }
}

unsafe impl<'a> IntoJulia for Cow<'a, str> {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }
}

unsafe impl IntoJulia for String {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }
}

unsafe impl IntoJulia for &dyn AsRef<str> {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        let ptr = self.as_ref().as_ptr().cast();
        let len = self.as_ref().len();
        jl_pchar_to_string(ptr, len)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_julia_type {
    ($type:ty, $jl_type:expr) => {
        unsafe impl crate::traits::JuliaType for $type {
            unsafe fn julia_type() -> *mut ::jl_sys::jl_datatype_t {
                $jl_type
            }
        }
    };
    ($type:ty, $jl_type:expr, $($bounds:tt)+) => {
        unsafe impl<$($bounds)+> crate::traits::JuliaType for $type {
            unsafe fn julia_type() -> *mut ::jl_sys::jl_datatype_t {
                $jl_type
            }
        }
    };
}

impl_julia_type!(u8, jl_uint8_type);
impl_julia_type!(u16, jl_uint16_type);
impl_julia_type!(u32, jl_uint32_type);
impl_julia_type!(u64, jl_uint64_type);
impl_julia_type!(i8, jl_int8_type);
impl_julia_type!(i16, jl_int16_type);
impl_julia_type!(i32, jl_int32_type);
impl_julia_type!(i64, jl_int64_type);
impl_julia_type!(f32, jl_float32_type);
impl_julia_type!(f64, jl_float64_type);
impl_julia_type!(bool, jl_bool_type);
impl_julia_type!(char, jl_char_type);
impl_julia_type!(*mut c_void, jl_voidpointer_type);

#[cfg(not(target_pointer_width = "64"))]
unsafe impl JuliaType for usize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_uint32_type
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl JuliaType for usize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_uint64_type
    }
}

#[cfg(not(target_pointer_width = "64"))]
unsafe impl JuliaType for isize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_int32_type
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl JuliaType for isize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_int64_type
    }
}

macro_rules! impl_primitive_cast {
    ($type:ty, $unboxer:ident) => {
        unsafe impl<'frame, 'data> Cast<'frame, 'data> for $type {
            type Output = Self;

            fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
                if value.is::<$type>() {
                    return unsafe { Ok(Self::cast_unchecked(value)) };
                }

                Err(JlrsError::WrongType)?
            }

            unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
                $unboxer(value.ptr().cast()) as _
            }
        }
    };
}

impl_primitive_cast!(u8, jl_unbox_uint8);
impl_primitive_cast!(u16, jl_unbox_uint16);
impl_primitive_cast!(u32, jl_unbox_uint32);
impl_primitive_cast!(u64, jl_unbox_uint64);
impl_primitive_cast!(i8, jl_unbox_int8);
impl_primitive_cast!(i16, jl_unbox_int16);
impl_primitive_cast!(i32, jl_unbox_int32);
impl_primitive_cast!(i64, jl_unbox_int64);
impl_primitive_cast!(f32, jl_unbox_float32);
impl_primitive_cast!(f64, jl_unbox_float64);
impl_primitive_cast!(*mut c_void, jl_unbox_voidpointer);

#[cfg(not(target_pointer_width = "64"))]
impl_primitive_cast!(usize, jl_unbox_uint32);

#[cfg(not(target_pointer_width = "64"))]
impl_primitive_cast!(isize, jl_unbox_int32);

#[cfg(target_pointer_width = "64")]
impl_primitive_cast!(usize, jl_unbox_uint64);

#[cfg(target_pointer_width = "64")]
impl_primitive_cast!(isize, jl_unbox_int64);

unsafe impl<'frame, 'data> Cast<'frame, 'data> for bool {
    type Output = Self;

    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<bool>() {
            unsafe { return Ok(Self::cast_unchecked(value)) }
        }

        Err(JlrsError::WrongType)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        jl_unbox_int8(value.ptr()) != 0
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for char {
    type Output = Self;

    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<char>() {
            unsafe {
                return std::char::from_u32(jl_unbox_uint32(value.ptr()))
                    .ok_or(JlrsError::InvalidCharacter.into());
            }
        }

        Err(JlrsError::WrongType)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        std::char::from_u32_unchecked(jl_unbox_uint32(value.ptr()))
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for String {
    type Output = Self;

    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<String>() {
            unsafe {
                let len = jl_string_len(value.ptr());

                if len == 0 {
                    return Ok(String::new());
                }

                // Is neither null nor dangling, we've just checked
                let raw = jl_string_data(value.ptr());
                let raw_slice = std::slice::from_raw_parts(raw, len);
                return Ok(String::from_utf8(raw_slice.into()).map_err(JlrsError::other)?);
            }
        }

        Err(JlrsError::WrongType)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        let len = jl_string_len(value.ptr());

        if len == 0 {
            return String::new();
        }

        // Is neither null nor dangling, we've just checked
        let raw = jl_string_data(value.ptr());
        let raw_slice = std::slice::from_raw_parts(raw, len);
        let owned_slice = Vec::from(raw_slice);
        String::from_utf8_unchecked(owned_slice)
    }
}

impl<'frame> Frame<'frame> for StaticFrame<'frame> {
    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        let mut frame = unsafe { self.nested_frame(capacity).unwrap() };
        func(&mut frame)
    }

    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut view = self.memory.nest_dynamic();
            let idx = view.new_frame()?;
            let mut frame = DynamicFrame {
                idx,
                len: 0,
                memory: view,
            };

            func(&mut frame)
        }
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        if self.capacity == self.len {
            return Err(AllocError::FrameOverflow(1, self.len).into());
        }

        let out = unsafe {
            let out = self.memory.new_output(self.idx, self.len);
            self.len += 1;
            out
        };

        Ok(out)
    }

    fn size(&self) -> usize {
        self.len
    }

    fn print_memory(&self) {
        self.memory.print_memory()
    }
}

impl<'frame> Frame<'frame> for DynamicFrame<'frame> {
    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<T> {
        let mut frame = unsafe { self.nested_frame().unwrap() };
        func(&mut frame)
    }

    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut view = self.memory.nest_static();
            let idx = view.new_frame(capacity)?;
            let mut frame = StaticFrame {
                idx,
                capacity,
                len: 0,
                memory: view,
            };

            func(&mut frame)
        }
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        unsafe {
            let out = self.memory.new_output(self.idx)?;
            self.len += 1;
            Ok(out)
        }
    }

    fn size(&self) -> usize {
        self.len
    }

    fn print_memory(&self) {
        self.memory.print_memory()
    }
}

impl<'frame> Frame<'frame> for NullFrame<'frame> {
    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        _: usize,
        _: F,
    ) -> JlrsResult<T> {
        Err(JlrsError::NullFrame)?
    }

    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        _: F,
    ) -> JlrsResult<T> {
        Err(JlrsError::NullFrame)?
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        Err(JlrsError::NullFrame)?
    }

    fn size(&self) -> usize {
        0
    }

    fn print_memory(&self) {}
}

pub(crate) mod private {
    use crate::error::AllocError;
    use crate::frame::{DynamicFrame, NullFrame, Output, StaticFrame};
    use crate::stack::FrameIdx;
    use crate::value::string::JuliaString;
    use crate::value::symbol::Symbol;
    use crate::value::{Value, Values};
    use jl_sys::jl_value_t;
    use jl_sys::{jl_symbol, jl_symbol_n};
    use std::borrow::Cow;

    // If a trait A is used in a trait bound, the trait methods from traits that A extends become
    // available without explicitly using those base traits. By taking this struct, which can only
    // be created inside this crate, as an argument, these methods can only be called from this
    // crate.
    pub struct Internal;

    // safety: never return the symbol to the user without assigning the 'base lifetime.
    pub trait TemporarySymbol {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol>;
    }

    pub trait Frame<'frame> {
        // protect the value from being garbage collected while this frame is active.
        // safety: the value must be a valid Julia value
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError>;

        // Create and protect multiple values from being garbage collected while this frame is active.
        fn create_many<P: super::IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError>;

        // Create and protect multiple values from being garbage collected while this frame is active.
        fn create_many_dyn(
            &mut self,
            values: &[&dyn super::IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError>;

        // Protect a value from being garbage collected while the output's frame is active.
        fn assign_output<'output>(
            &mut self,
            output: Output<'output>,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static>;
    }

    impl<'a> TemporarySymbol for &'a str {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ptr();
            let symbol = jl_symbol_n(symbol_ptr.cast(), self.len());
            Symbol::wrap(symbol)
        }
    }

    impl<'a> TemporarySymbol for Cow<'a, str> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.len());
            Symbol::wrap(symbol)
        }
    }

    impl TemporarySymbol for String {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.len());
            Symbol::wrap(symbol)
        }
    }

    impl TemporarySymbol for &dyn AsRef<str> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ref().as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.as_ref().len());
            Symbol::wrap(symbol)
        }
    }

    impl<'frame> TemporarySymbol for JuliaString<'frame> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_c_str();
            let symbol = jl_symbol(symbol_ptr.as_ptr());
            Symbol::wrap(symbol)
        }
    }

    impl<'s> TemporarySymbol for Symbol<'s> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            Symbol::wrap(self.ptr())
        }
    }

    impl<'frame> Frame<'frame> for StaticFrame<'frame> {
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            if self.capacity == self.len {
                return Err(AllocError::FrameOverflow(1, self.len));
            }

            let out = {
                let out = self.memory.protect(self.idx, self.len, value.cast());
                self.len += 1;
                out
            };

            Ok(out)
        }

        fn create_many<P: super::IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                if self.capacity < self.len + values.len() {
                    return Err(AllocError::FrameOverflow(values.len(), self.capacity()));
                }

                let offset = self.len;
                for value in values {
                    self.memory
                        .protect(self.idx, self.len, value.into_julia().cast());
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn create_many_dyn(
            &mut self,
            values: &[&dyn super::IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                if self.capacity < self.len + values.len() {
                    return Err(AllocError::FrameOverflow(values.len(), self.capacity()));
                }

                let offset = self.len;
                for value in values {
                    self.memory
                        .protect(self.idx, self.len, value.into_julia().cast());
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn assign_output<'output>(
            &mut self,
            output: Output<'output>,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static> {
            unsafe {
                self.memory
                    .protect(FrameIdx::default(), output.offset, value.cast())
            }
        }
    }

    impl<'frame> Frame<'frame> for DynamicFrame<'frame> {
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            let out = self.memory.protect(self.idx, value.cast())?;
            self.len += 1;
            Ok(out)
        }

        fn create_many<P: super::IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                let offset = self.len;
                // TODO: check capacity

                for value in values {
                    match self.memory.protect(self.idx, value.into_julia().cast()) {
                        Ok(_) => (),
                        Err(AllocError::StackOverflow(_, n)) => {
                            return Err(AllocError::StackOverflow(values.len(), n))
                        }
                        _ => unreachable!(),
                    }
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn create_many_dyn(
            &mut self,
            values: &[&dyn super::IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                let offset = self.len;
                // TODO: check capacity in advance

                for value in values {
                    self.memory.protect(self.idx, value.into_julia().cast())?;
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn assign_output<'output>(
            &mut self,
            output: Output<'output>,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static> {
            unsafe { self.memory.protect_output(output, value.cast()) }
        }
    }

    impl<'frame> Frame<'frame> for NullFrame<'frame> {
        unsafe fn protect(
            &mut self,
            _: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            Err(AllocError::FrameOverflow(1, 0))
        }

        fn create_many<P: super::IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            Err(AllocError::FrameOverflow(values.len(), 0))
        }

        fn create_many_dyn(
            &mut self,
            values: &[&dyn super::IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            Err(AllocError::FrameOverflow(values.len(), 0))
        }

        fn assign_output<'output>(
            &mut self,
            _: Output<'output>,
            _: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static> {
            unreachable!()
        }
    }
}
