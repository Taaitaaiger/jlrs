//! Managed for `Char`.
//!
//! In Rust it's unsafe to create an invalid `char`, while a `Char` in Julia can be an arbitrary
//! `u32` in some rare cases. Rather than treating all `Char`s as `u32` or `char`s, jlrs provides
//! a wrapper for this type.
use std::fmt::{Debug, Formatter, Result as FmtResult, Write};

use jl_sys::{jl_char_type, jl_unbox_uint32};

use crate::{
    convert::unbox::Unbox,
    data::managed::{private::ManagedPriv, value::Value},
    impl_julia_typecheck, impl_valid_layout,
    private::Private,
};

/// A Julia `Char`.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Char(u32);

impl Char {
    #[inline(always)]
    pub fn new(val: char) -> Self {
        Char(val as u32)
    }

    /// Returns the value of the `Char` as a `u32`.
    #[inline(always)]
    pub fn as_u32(self) -> u32 {
        self.0
    }

    /// Returns the value of the `Char` as a `char` if it's valid, `None` if it isn't.
    #[inline(always)]
    pub fn try_as_char(self) -> Option<char> {
        char::from_u32(self.0)
    }

    /// Returns the value of the `Char` as a `char`.
    ///
    /// Safety: the `Char` must be a valid `char`.
    #[inline(always)]
    pub unsafe fn try_as_char_unchecked(self) -> char {
        char::from_u32_unchecked(self.0)
    }
}

impl<'scope> Debug for Char {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(ch) = char::from_u32(self.0) {
            f.write_char(ch)
        } else {
            f.write_fmt(format_args!("{:#010x} <invalid char>", self.0))
        }
    }
}

impl_julia_typecheck!(Char, jl_char_type);
impl_valid_layout!(Char);

unsafe impl Unbox for Char {
    type Output = Self;
    #[inline(always)]
    unsafe fn unbox(value: Value) -> Char {
        Char(jl_unbox_uint32(value.unwrap(Private).cast()))
    }
}

unsafe impl Unbox for char {
    type Output = Char;
    #[inline(always)]
    unsafe fn unbox(value: Value) -> Char {
        Char(jl_unbox_uint32(value.unwrap(Private).cast()))
    }
}
