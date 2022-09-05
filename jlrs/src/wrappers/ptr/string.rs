//! Wrapper for `String`.

use crate::{
    convert::unbox::Unbox,
    error::{JlrsError, JlrsResult},
    impl_julia_typecheck,
    memory::{global::Global, output::Output, scope::PartialScope},
    private::Private,
    wrappers::ptr::{private::WrapperPriv, value::Value},
};
use jl_sys::{jl_pchar_to_string, jl_string_type};
use std::{
    ffi::CStr,
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    mem,
    ptr::NonNull,
    str,
};

use super::Ref;

/// A Julia string.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct JuliaString<'scope>(*const u8, PhantomData<&'scope ()>);

impl<'scope> JuliaString<'scope> {
    /// Create a new Julia string.
    pub fn new<'target, V, S>(scope: S, string: V) -> JlrsResult<JuliaString<'target>>
    where
        V: AsRef<str>,
        S: PartialScope<'target>,
    {
        let global = scope.global();
        // Safety: the result is immediately rooted
        unsafe { JuliaString::new_unrooted(global, string).root(scope) }
    }

    /// Create a new Julia string. Unlike [`JuliaString::new`] this method doesn't root the
    /// allocated value.
    pub fn new_unrooted<'global, V>(_: Global<'global>, string: V) -> StringRef<'global>
    where
        V: AsRef<str>,
    {
        // Safety: the C API function is called with valid arguments
        unsafe {
            let str_ref = string.as_ref();
            let len = str_ref.len();
            let ptr = str_ref.as_ptr().cast();
            let s = jl_pchar_to_string(ptr, len);
            StringRef::wrap(s.cast())
        }
    }

    /// Create a new Julia string.
    pub fn new_bytes<'target, V, S>(scope: S, bytes: V) -> JlrsResult<JuliaString<'target>>
    where
        V: AsRef<[u8]>,
        S: PartialScope<'target>,
    {
        let global = scope.global();
        // Safety: the result is immediately rooted
        unsafe { JuliaString::new_bytes_unrooted(global, bytes).root(scope) }
    }

    /// Create a new Julia string. Unlike [`JuliaString::new_bytes`] this method doesn't root the
    /// allocated value.
    pub fn new_bytes_unrooted<'global, V>(_: Global<'global>, bytes: V) -> StringRef<'global>
    where
        V: AsRef<[u8]>,
    {
        // Safety: the C API function is called with valid arguments
        unsafe {
            let str_ref = bytes.as_ref();
            let len = str_ref.len();
            let ptr = str_ref.as_ptr().cast();
            let s = jl_pchar_to_string(ptr, len);
            StringRef::wrap(s.cast())
        }
    }

    /// Returns the length of the string.
    pub fn len(self) -> usize {
        // Safety: the pointer points to valid data, the length of the array is stored at the
        // beginning
        unsafe { *self.0.cast() }
    }

    /// Returns the string as a `CStr`.
    pub fn as_c_str(self) -> &'scope CStr {
        // Safety: The string is terminated with a null character.
        unsafe {
            let str_begin = self.0.add(mem::size_of::<usize>());
            CStr::from_ptr(str_begin.cast())
        }
    }

    /// Returns the string as a slice of bytes, including all null characters..
    pub fn as_bytes(self) -> &'scope [u8] {
        unsafe {
            let len = self.len();
            let str_begin = self.0.add(mem::size_of::<usize>());
            std::slice::from_raw_parts(str_begin, len)
        }
    }

    /// Returns the string as a string slice, or an error if it the string contains
    /// invalid characters
    pub fn as_str(self) -> JlrsResult<&'scope str> {
        Ok(str::from_utf8(self.as_c_str().to_bytes()).map_err(JlrsError::other)?)
    }

    /// Returns the string as a string slice without checking if the string is properly encoded.
    ///
    /// Safety: the string must be properly encoded.
    pub unsafe fn as_str_unchecked(self) -> &'scope str {
        str::from_utf8_unchecked(self.as_c_str().to_bytes())
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> JuliaString<'target> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<JuliaString>(ptr);
            JuliaString::wrap_non_null(ptr, Private)
        }
    }
}

impl_julia_typecheck!(JuliaString<'scope>, jl_string_type, 'scope);

unsafe impl Unbox for String {
    type Output = Result<String, Vec<u8>>;
    unsafe fn unbox(value: Value) -> Self::Output {
        let s = value.cast_unchecked::<JuliaString>();
        match s.as_str() {
            Ok(s) => Ok(s.into()),
            Err(_) => Err(s.as_bytes().into()),
        }
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

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        JuliaString(inner.as_ptr(), PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        unsafe { NonNull::new_unchecked(self.0 as *mut _) }
    }
}

impl_root!(JuliaString, 1);

/// A reference to a [`JuliaString`] that has not been explicitly rooted.
pub type StringRef<'scope> = Ref<'scope, 'static, JuliaString<'scope>>;
impl_valid_layout!(StringRef, String);
impl_ref_root!(JuliaString, StringRef, 1);
