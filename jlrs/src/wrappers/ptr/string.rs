//! Wrapper for `Core.String`.

use crate::error::{JlrsError, JlrsResult};
use crate::wrappers::ptr::value::Value;
use crate::{convert::unbox::Unbox, private::Private};
use crate::{impl_julia_typecheck, impl_valid_layout};
use jl_sys::jl_string_type;
use std::mem;
use std::{ffi::CStr, ptr::NonNull};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
};

use super::private::Wrapper;

/// A raw Julia string.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct JuliaString<'frame>(*const u8, PhantomData<&'frame ()>);

impl<'frame> JuliaString<'frame> {
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
}

impl<'scope> Debug for JuliaString<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("JuliaString").field(&self.as_str()).finish()
    }
}

impl_julia_typecheck!(JuliaString<'frame>, jl_string_type, 'frame);

impl_valid_layout!(JuliaString<'frame>, 'frame);

unsafe impl<'scope> Unbox for JuliaString<'scope> {
    type Output = Result<String, Vec<u8>>;
    unsafe fn unbox(value: Value) -> Self::Output {
        let slice = value.cast_unchecked::<JuliaString>().as_slice();
        std::str::from_utf8(slice)
            .map(String::from)
            .map_err(|_| slice.into())
    }
}

unsafe impl Unbox for String {
    type Output = Result<String, Vec<u8>>;
    unsafe fn unbox(value: Value) -> Self::Output {
        JuliaString::unbox(value)
    }
}
impl<'scope> Wrapper<'scope, '_> for JuliaString<'scope> {
    type Internal = u8;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        JuliaString(inner.as_ptr(), PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        std::mem::transmute(self.0)
    }
}
