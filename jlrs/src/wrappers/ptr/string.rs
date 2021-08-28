//! Wrapper for `String`.

use crate::error::{JlrsError, JlrsResult};
use crate::memory::frame::Frame;
use crate::memory::global::Global;
use crate::memory::scope::Scope;
use crate::wrappers::ptr::{private::Wrapper as WrapperPriv, value::Value, StringRef};
use crate::{convert::unbox::Unbox, private::Private};
use crate::{impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_pchar_to_string, jl_string_type};
use std::{ffi::CStr, ptr::NonNull};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    mem, str,
};

/// A Julia string.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct JuliaString<'scope>(*const u8, PhantomData<&'scope ()>);

impl<'scope> JuliaString<'scope> {
    /// Create a new Julia string.
    pub fn new<'target, 'current, V, S, F>(scope: S, string: V) -> JlrsResult<S::Value>
    where
        V: AsRef<str>,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            let global = scope.global();
            JuliaString::new_unrooted(global, string).root(scope)
        }
    }

    /// Create a new Julia string. Unlike [`JuliaString::new`] this method doesn't root the
    /// allocated value.
    pub fn new_unrooted<'global, V>(_: Global<'global>, string: V) -> StringRef<'global>
    where
        V: AsRef<str>,
    {
        unsafe {
            let str_ref = string.as_ref();
            let ptr = str_ref.as_ptr().cast();
            let len = str_ref.len();
            let s = jl_pchar_to_string(ptr, len);
            debug_assert!(!s.is_null());
            StringRef::wrap(s.cast())
        }
    }

    /// Returns the length of the string.
    pub fn len(self) -> usize {
        unsafe { *self.0.cast() }
    }

    /// Returns the string as a `CStr`.
    pub fn as_c_str(self) -> &'scope CStr {
        unsafe {
            let str_begin = self.0.add(mem::size_of::<usize>());
            CStr::from_ptr(str_begin.cast())
        }
    }

    /// Returns the string as a slice of bytes without the terminating `\0`.
    pub fn as_slice(self) -> &'scope [u8] {
        self.as_c_str().to_bytes()
    }

    /// Returns the string as a string slice, or an error if it the string contains
    /// invalid characters
    pub fn as_str(self) -> JlrsResult<&'scope str> {
        Ok(str::from_utf8(self.as_slice()).or(Err(JlrsError::NotUTF8))?)
    }

    /// Returns the string as a string slice without checking if the string is properly encoded.
    pub unsafe fn as_str_unchecked(self) -> &'scope str {
        str::from_utf8_unchecked(self.as_slice())
    }
}

impl_julia_typecheck!(JuliaString<'scope>, jl_string_type, 'scope);

impl_valid_layout!(JuliaString<'scope>, 'scope);

unsafe impl<'scope> Unbox for JuliaString<'scope> {
    type Output = Result<String, Vec<u8>>;
    unsafe fn unbox(value: Value) -> Self::Output {
        let slice = value.cast_unchecked::<JuliaString>().as_slice();
        str::from_utf8(slice)
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

impl Debug for JuliaString<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.as_str().unwrap_or("<Non-UTF8 string>"))
    }
}

impl<'scope> WrapperPriv<'scope, '_> for JuliaString<'scope> {
    type Wraps = u8;
    const NAME: &'static str = "String";

    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        JuliaString(inner.as_ptr(), PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        unsafe { NonNull::new_unchecked(self.0 as *mut _) }
    }
}
