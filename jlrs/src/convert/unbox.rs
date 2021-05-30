//! Extract the contents of a Julia value.
//!
//! A [`Value`] contains a pointer to some data owned by Julia. The layout of this data depends on
//! the [`DataType`] of the value. In general, the layout is highly predictable. For example, if
//` the `DataType` is `Int8`, the pointer points to an `i8`. In order to extract the contents of a
//! [`Value`] it must be unboxed. Unlike [`Cast`], which is used to convert a [`Value`] to another
//! builtin wrapper type that is still owned by Julia, the [`Unbox`] trait defined in this module
//! usually dereferences the pointer.
//!
//! There are a few exceptions to this rule. In particular, unboxing a `char` or a `bool` results
//! in a [`Char`] or a [`Bool`] respectively. The reason is that while using invalid `Char`s and
//! `Bool`s is an error in Julia, it's undefined behavior to create them in Rust. Similarly,
//! strings in Julia should be UTF-8 encoded, but to account for the possibility that the contents
//! are invalid the implementation of [`Unbox`] returns a `String` if the contents are valid and a
//! `Vec<u8>` if they're not.
//!
//! [`Cast`]: crate::convert::cast::Cast
//! [`Bool`]: crate::wrappers::inline::bool::Bool
//! [`Char`]: crate::wrappers::inline::char::Char
//! [`DataType`]: crate::wrappers::ptr::datatype::DataType

use crate::{
    private::Private,
    wrappers::ptr::{private::Wrapper, value::Value},
};
use jl_sys::{
    jl_unbox_float32, jl_unbox_float64, jl_unbox_int16, jl_unbox_int32, jl_unbox_int64,
    jl_unbox_int8, jl_unbox_uint16, jl_unbox_uint32, jl_unbox_uint64, jl_unbox_uint8,
    jl_unbox_voidpointer,
};
use std::ffi::c_void;

/// Convert a value from Julia to Rust. By default, it's implemented for primitive types and
/// `String`. You should not implement it manually for custom types, but use `JlrsReflect.jl`
/// and `jlrs-derive` instead.
pub unsafe trait Unbox {
    type Output: Sized + Clone;

    /// Unbox the value as `Self::Output`.
    ///
    /// Safety: The default implementation assumes that `Self::Output` is the correct layout for
    /// the data that `value` points to.
    unsafe fn unbox(value: Value) -> Self::Output {
        value
            .unwrap_non_null(Private)
            .cast::<Self::Output>()
            .as_ref()
            .clone()
    }
}

macro_rules! impl_unboxer {
    ($type:ty, $unboxer:expr) => {
        unsafe impl Unbox for $type {
            type Output = Self;
            unsafe fn unbox(value: Value) -> $type {
                $unboxer(<Value as crate::wrappers::ptr::private::Wrapper>::unwrap(
                    value,
                    $crate::private::Private,
                )) as _
            }
        }
    };
}

impl_unboxer!(u8, jl_unbox_uint8);
impl_unboxer!(u16, jl_unbox_uint16);
impl_unboxer!(u32, jl_unbox_uint32);
impl_unboxer!(u64, jl_unbox_uint64);
impl_unboxer!(i8, jl_unbox_int8);
impl_unboxer!(i16, jl_unbox_int16);
impl_unboxer!(i32, jl_unbox_int32);
impl_unboxer!(i64, jl_unbox_int64);
impl_unboxer!(f32, jl_unbox_float32);
impl_unboxer!(f64, jl_unbox_float64);
impl_unboxer!(*mut c_void, jl_unbox_voidpointer);

#[cfg(not(target_pointer_width = "64"))]
impl_unboxer!(usize, jl_unbox_uint32);

#[cfg(not(target_pointer_width = "64"))]
impl_unboxer!(isize, jl_unbox_int32);

#[cfg(target_pointer_width = "64")]
impl_unboxer!(usize, jl_unbox_uint64);

#[cfg(target_pointer_width = "64")]
impl_unboxer!(isize, jl_unbox_int64);
