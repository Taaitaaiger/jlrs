use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::value::Value;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::jl_string_type;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem;
use std::slice;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct JlString<'frame>(*const u8, PhantomData<&'frame ()>);

impl<'frame> JlString<'frame> {
    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *const u8 {
        self.0
    }

    pub fn len(self) -> usize {
        unsafe { *self.0.cast() }
    }

    pub fn data_cstr(self) -> &'frame CStr {
        unsafe {
            let str_begin = self.0.add(mem::size_of::<usize>());
            CStr::from_ptr(str_begin.cast())
        }
    }

    pub fn data_slice(self) -> &'frame [u8] {
        unsafe {
            let str_begin = self.0.add(mem::size_of::<usize>());
            slice::from_raw_parts(str_begin, self.len())
        }
    }
}

impl<'frame> Into<Value<'frame, 'static>> for JlString<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { mem::transmute(self.ptr()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for JlString<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAString)?
    }

    unsafe fn cast_unchecked<'fr, 'da>(value: Value<'frame, 'data>) -> Self::Output {
        std::mem::transmute(value)
    }
}

impl_julia_typecheck!(JlString<'frame>, jl_string_type, 'frame);
impl_julia_type!(JlString<'frame>, jl_string_type, 'frame);
impl_valid_layout!(JlString<'frame>, 'frame);
