//! Associate a Rust type with a Julia type.

use jl_sys::{
    jl_bool_type, jl_char_type, jl_datatype_t, jl_float32_type, jl_float64_type, jl_int16_type,
    jl_int32_type, jl_int64_type, jl_int8_type, jl_uint16_type, jl_uint32_type, jl_uint64_type,
    jl_uint8_type, jl_voidpointer_type,
};
use std::ffi::c_void;

/// Trait implemented by types that have an associated type in Julia.
pub unsafe trait JuliaType {
    #[doc(hidden)]
    unsafe fn julia_type() -> *mut jl_datatype_t;
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_julia_type {
    ($type:ty, $jl_type:expr) => {
        unsafe impl crate::layout::julia_type::JuliaType for $type {
            unsafe fn julia_type() -> *mut ::jl_sys::jl_datatype_t {
                $jl_type
            }
        }
    };
    ($type:ty, $jl_type:expr, $($bounds:tt)+) => {
        unsafe impl<$($bounds)+> crate::layout::julia_type::JuliaType for $type {
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
