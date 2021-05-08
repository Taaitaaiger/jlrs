use super::Value;
use crate::{convert::unbox::UnboxFn, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_char_type, jl_unbox_uint32};
use std::fmt::{Debug, Formatter, Result as FmtResult, Write};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JuliaChar(u32);

impl JuliaChar {
    pub fn as_u32(self) -> u32 {
        self.0
    }

    pub fn try_as_char(self) -> Option<char> {
        char::from_u32(self.0)
    }

    pub unsafe fn try_as_char_unchecked(self) -> char {
        char::from_u32_unchecked(self.0)
    }
}

impl<'scope> Debug for JuliaChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(ch) = char::from_u32(self.0) {
            f.write_char(ch)
        } else {
            f.write_fmt(format_args!("{:#010x} <invalid char>", self.0))
        }
    }
}

impl_julia_typecheck!(JuliaChar, jl_char_type);
impl_valid_layout!(JuliaChar);

unsafe impl UnboxFn for JuliaChar {
    type Output = Self;
    unsafe fn call_unboxer(value: Value) -> JuliaChar {
        JuliaChar(jl_unbox_uint32(value.inner().as_ptr().cast()))
    }
}

unsafe impl UnboxFn for char {
    type Output = JuliaChar;
    unsafe fn call_unboxer(value: Value) -> JuliaChar {
        JuliaChar(jl_unbox_uint32(value.inner().as_ptr().cast()))
    }
}
