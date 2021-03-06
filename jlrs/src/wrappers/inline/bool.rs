//! Wrapper for `Core.Bool`.
//!
//! In Rust it's unsound to create an invalid `bool`, while a `Bool` in Julia can be an arbitrary
//! `i8` in some rare cases. Rather than treating all `Bool`s as `i8` or `bool`s jlrs provides
/// a wrapper for this type.
use crate::{convert::unbox::Unbox, impl_julia_typecheck, impl_valid_layout};
use crate::{
    private::Private,
    wrappers::ptr::{private::Wrapper, value::Value},
};
use jl_sys::{jl_bool_type, jl_unbox_int8};
use std::fmt::{Debug, Formatter, Result as FmtResult};

/// A Julia `Bool`.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bool(i8);

impl Bool {
    pub fn new(val: bool) -> Self {
        Bool(val as i8)
    }

    /// Returns the value of the `Bool` as a `i8`.
    pub fn as_i8(self) -> i8 {
        self.0
    }

    /// Returns the value of the `Bool` as a `bool` if it's 0 or 1, `None` if it isn't.
    pub fn try_as_bool(self) -> Option<bool> {
        match self.0 {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    /// Returns the value of the `Bool` as a `bool`.
    pub fn as_bool(self) -> bool {
        self.0 != 0
    }
}

impl<'scope> Debug for Bool {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.as_i8() {
            0 => f.write_str("false"),
            1 => f.write_str("true"),
            n => f.write_fmt(format_args!("{} <invalid bool>", n)),
        }
    }
}

impl_julia_typecheck!(Bool, jl_bool_type);
impl_valid_layout!(Bool);

unsafe impl Unbox for Bool {
    type Output = Self;
    unsafe fn unbox(value: Value) -> Bool {
        Bool(jl_unbox_int8(value.unwrap(Private).cast()))
    }
}

unsafe impl Unbox for bool {
    type Output = Bool;
    unsafe fn unbox(value: Value) -> Bool {
        Bool(jl_unbox_int8(value.unwrap(Private).cast()))
    }
}
