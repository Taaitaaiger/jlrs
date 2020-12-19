use jl_sys::{
    jl_box_bool, jl_box_char, jl_box_float32, jl_box_float64, jl_box_int16, jl_box_int32,
    jl_box_int64, jl_box_int8, jl_box_uint16, jl_box_uint32, jl_box_uint64, jl_box_uint8,
    jl_box_voidpointer, jl_pchar_to_string,
};
use std::borrow::Cow;
use std::ffi::c_void;

/// Trait implemented by types that can be converted to a Julia value in combination with
/// [`Value::new`]. This trait can be derived for custom bits types that implement
/// `JuliaStruct`.
///
/// [`Value::new`]: ../value/struct.Value.html#method.new
pub unsafe trait IntoJulia {
    #[doc(hidden)]
    unsafe fn into_julia(&self) -> *mut ::jl_sys::jl_value_t;
}

macro_rules! impl_into_julia {
    ($type:ty, $boxer:ident) => {
        unsafe impl IntoJulia for $type {
            unsafe fn into_julia(&self) -> *mut ::jl_sys::jl_value_t {
                $boxer(*self)
            }
        }
    };
    ($type:ty, $as:ty, $boxer:ident) => {
        unsafe impl IntoJulia for $type {
            unsafe fn into_julia(&self) -> *mut ::jl_sys::jl_value_t {
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
    unsafe fn into_julia(&self) -> *mut ::jl_sys::jl_value_t {
        jl_box_uint64(*self as u64)
    }
}

#[cfg(not(target_pointer_width = "64"))]
unsafe impl IntoJulia for isize {
    unsafe fn into_julia(&self) -> *mut ::jl_sys::jl_value_t {
        jl_box_int32(*self as i32)
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl IntoJulia for isize {
    unsafe fn into_julia(&self) -> *mut ::jl_sys::jl_value_t {
        jl_box_int64(*self as i64)
    }
}

unsafe impl<'a> IntoJulia for &'a str {
    unsafe fn into_julia(&self) -> *mut ::jl_sys::jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }
}

unsafe impl<'a> IntoJulia for Cow<'a, str> {
    unsafe fn into_julia(&self) -> *mut ::jl_sys::jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }
}

unsafe impl IntoJulia for String {
    unsafe fn into_julia(&self) -> *mut ::jl_sys::jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }
}

unsafe impl IntoJulia for &dyn AsRef<str> {
    unsafe fn into_julia(&self) -> *mut ::jl_sys::jl_value_t {
        let ptr = self.as_ref().as_ptr().cast();
        let len = self.as_ref().len();
        jl_pchar_to_string(ptr, len)
    }
}
