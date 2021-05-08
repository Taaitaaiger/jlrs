//! Convert a value from Rust to Julia.
//!
//! The trait in this module should be implemented by deriving `IntoJulia`, its methods are
//! never called directly but only through [`Value::new`].
//!
//! [`Value::new`]: crate::value::Value::new

use jl_sys::{
    jl_bool_type, jl_box_bool, jl_box_char, jl_box_float32, jl_box_float64, jl_box_int16,
    jl_box_int32, jl_box_int64, jl_box_int8, jl_box_uint16, jl_box_uint32, jl_box_uint64,
    jl_box_uint8, jl_box_voidpointer, jl_char_type, jl_datatype_t, jl_float32_type,
    jl_float64_type, jl_int16_type, jl_int32_type, jl_int64_type, jl_int8_type,
    jl_new_struct_uninit, jl_pchar_to_string, jl_string_type, jl_uint16_type, jl_uint32_type,
    jl_uint64_type, jl_uint8_type, jl_value_t, jl_voidpointer_type,
};
use std::borrow::Cow;
use std::ffi::c_void;

/// Trait implemented by types that can be converted to a Julia value in combination with
/// [`Value::new`]. This trait can be derived for custom bits types that implement
/// `JuliaStruct`.
///
/// [`Value::new`]: crate::value::Value::new
pub unsafe trait IntoJulia: Sized {
    #[doc(hidden)]
    unsafe fn julia_type() -> *mut jl_datatype_t;

    #[doc(hidden)]
    unsafe fn into_julia(self) -> *mut jl_value_t {
        let ty = Self::julia_type();
        debug_assert!((&*ty).isbitstype != 0);

        let container = jl_new_struct_uninit(ty.cast());
        let data: *mut Self = container.cast();
        ::std::ptr::write(data, self);

        container
    }
}

macro_rules! impl_into_julia {
    ($type:ty, $boxer:ident, $julia_type:expr) => {
        unsafe impl IntoJulia for $type {
            unsafe fn julia_type() -> *mut jl_datatype_t {
                $julia_type
            }

            unsafe fn into_julia(self) -> *mut jl_value_t {
                $boxer(self)
            }
        }
    };
    ($type:ty, $as:ty, $boxer:ident, $julia_type:expr) => {
        unsafe impl IntoJulia for $type {
            unsafe fn julia_type() -> *mut jl_datatype_t {
                $julia_type
            }

            unsafe fn into_julia(self) -> *mut jl_value_t {
                $boxer(self as $as)
            }
        }
    };
}

impl_into_julia!(bool, i8, jl_box_bool, jl_bool_type);
impl_into_julia!(char, u32, jl_box_char, jl_char_type);
impl_into_julia!(u8, jl_box_uint8, jl_uint8_type);
impl_into_julia!(u16, jl_box_uint16, jl_uint16_type);
impl_into_julia!(u32, jl_box_uint32, jl_uint32_type);
impl_into_julia!(u64, jl_box_uint64, jl_uint64_type);
impl_into_julia!(i8, jl_box_int8, jl_int8_type);
impl_into_julia!(i16, jl_box_int16, jl_int16_type);
impl_into_julia!(i32, jl_box_int32, jl_int32_type);
impl_into_julia!(i64, jl_box_int64, jl_int64_type);
impl_into_julia!(f32, jl_box_float32, jl_float32_type);
impl_into_julia!(f64, jl_box_float64, jl_float64_type);
impl_into_julia!(*mut c_void, jl_box_voidpointer, jl_voidpointer_type);

#[cfg(not(target_pointer_width = "64"))]
unsafe impl IntoJulia for usize {
    unsafe fn into_julia(self) -> *mut jl_value_t {
        jl_box_uint32(self as u32)
    }

    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_uint32_type
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl IntoJulia for usize {
    unsafe fn into_julia(self) -> *mut jl_value_t {
        jl_box_uint64(self as u64)
    }

    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_uint64_type
    }
}

#[cfg(not(target_pointer_width = "64"))]
unsafe impl IntoJulia for isize {
    unsafe fn into_julia(self) -> *mut jl_value_t {
        jl_box_int32(self as i32)
    }

    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_int32_type
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl IntoJulia for isize {
    unsafe fn into_julia(self) -> *mut jl_value_t {
        jl_box_int64(self as i64)
    }

    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_int64_type
    }
}

unsafe impl<'a> IntoJulia for &'a str {
    unsafe fn into_julia(self) -> *mut jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }

    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_string_type
    }
}

unsafe impl<'a> IntoJulia for Cow<'a, str> {
    unsafe fn into_julia(self) -> *mut jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }

    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_string_type
    }
}

unsafe impl IntoJulia for String {
    unsafe fn into_julia(self) -> *mut jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }

    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_string_type
    }
}

unsafe impl IntoJulia for &dyn AsRef<str> {
    unsafe fn into_julia(self) -> *mut jl_value_t {
        let ptr = self.as_ref().as_ptr().cast();
        let len = self.as_ref().len();
        jl_pchar_to_string(ptr, len)
    }

    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_string_type
    }
}
