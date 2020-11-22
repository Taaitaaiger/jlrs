//! Convert a `Value` to another type.

use crate::error::{JlrsError, JlrsResult};
use crate::value::Value;
use jl_sys::{
    jl_string_data, jl_string_len, jl_unbox_float32, jl_unbox_float64, jl_unbox_int16,
    jl_unbox_int32, jl_unbox_int64, jl_unbox_int8, jl_unbox_uint16, jl_unbox_uint32,
    jl_unbox_uint64, jl_unbox_uint8, jl_unbox_voidpointer,
};
use std::ffi::c_void;

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
