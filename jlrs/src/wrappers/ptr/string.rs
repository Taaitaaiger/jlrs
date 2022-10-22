//! Wrapper for `String`.

use crate::{
    convert::unbox::Unbox,
    error::{JlrsError, JlrsResult},
    impl_julia_typecheck,
    memory::target::Target,
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
    pub fn new<'target, V, T>(target: T, string: V) -> T::Data
    where
        V: AsRef<str>,
        T: Target<'target, 'static, JuliaString<'target>>,
    {
        let str_ref = string.as_ref();
        let len = str_ref.len();
        let ptr = str_ref.as_ptr().cast();
        unsafe {
            let s = jl_pchar_to_string(ptr, len);
            target.data_from_ptr(NonNull::new_unchecked(s).cast(), Private)
        }
    }

    /// Create a new Julia string.
    pub fn new_bytes<'target, V, T>(target: T, bytes: V) -> T::Data
    where
        V: AsRef<[u8]>,
        T: Target<'target, 'static, JuliaString<'target>>,
    {
        let bytes_ref = bytes.as_ref();
        let len = bytes_ref.len();
        let ptr = bytes_ref.as_ptr().cast();
        unsafe {
            let s = jl_pchar_to_string(ptr, len);
            target.data_from_ptr(NonNull::new_unchecked(s).cast(), Private)
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

    /// Use the target to reroot this data.
    pub fn root<'target, T>(self, target: T) -> T::Data
    where
        T: Target<'target, 'static, JuliaString<'target>>,
    {
        // Safety: the data is valid.
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
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
    type StaticPriv = JuliaString<'static>;
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
