//! Support for accessing raw Julia strings.

use crate::convert::{cast::Cast, unbox::UnboxFn};
use crate::error::{JlrsError, JlrsResult};
use crate::value::Value;
use crate::{impl_julia_typecheck, impl_valid_layout};
use jl_sys::jl_string_type;
use std::ffi::CStr;
use std::mem;
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
};

/// A raw Julia string.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct JuliaString<'frame>(*const u8, PhantomData<&'frame ()>);

impl<'frame> JuliaString<'frame> {
    pub(crate) unsafe fn wrap(ptr: *const u8) -> Self {
        debug_assert!(!ptr.is_null());
        JuliaString(ptr, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *const u8 {
        self.0
    }

    /// Returns the length of the string.
    pub fn len(self) -> usize {
        unsafe { *self.0.cast() }
    }

    /// Returns the string as a `CStr`.
    pub fn as_c_str(self) -> &'frame CStr {
        unsafe {
            let str_begin = self.0.add(mem::size_of::<usize>());
            CStr::from_ptr(str_begin.cast())
        }
    }

    /// Returns the string as a slice of bytes without the terminating `\0`.
    pub fn as_slice(self) -> &'frame [u8] {
        self.as_c_str().to_bytes()
    }

    /// Returns the string as a string slice, or an error if it the string contains
    /// invalid characters
    pub fn as_str(self) -> JlrsResult<&'frame str> {
        Ok(std::str::from_utf8(self.as_slice()).or(Err(JlrsError::NotUnicode))?)
    }

    /// Returns the string as a string slice without checking if the string is properly encoded.
    pub unsafe fn as_str_unchecked(self) -> &'frame str {
        std::str::from_utf8_unchecked(self.as_slice())
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for JuliaString<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("JuliaString").field(&self.as_str()).finish()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for JuliaString<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { mem::transmute(self.ptr()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for JuliaString<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAString)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        std::mem::transmute(value)
    }
}

impl_julia_typecheck!(JuliaString<'frame>, jl_string_type, 'frame);

impl_valid_layout!(JuliaString<'frame>, 'frame);

unsafe impl<'scope> UnboxFn for JuliaString<'scope> {
    type Output = Result<String, Vec<u8>>;
    unsafe fn call_unboxer(value: Value) -> Self::Output {
        let slice = value.cast_unchecked::<JuliaString>().as_slice();
        std::str::from_utf8(slice)
            .map(String::from)
            .map_err(|_| slice.into())
    }
}

unsafe impl UnboxFn for String {
    type Output = Result<String, Vec<u8>>;
    unsafe fn call_unboxer(value: Value) -> Self::Output {
        let slice = value.cast_unchecked::<JuliaString>().as_slice();
        std::str::from_utf8(slice)
            .map(String::from)
            .map_err(|_| slice.into())
    }
}
