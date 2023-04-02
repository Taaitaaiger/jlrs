//! Layout type for `Bool`.
//!
//! In Rust it's unsound to create an invalid `bool`, while a `Bool` in Julia can be an arbitrary
//! `i8` in some rare cases. Rather than treating all `Bool`s as `i8` or `bool`s jlrs provides
//! a custom layout for this type, [`Bool`].
use std::fmt::{Debug, Formatter, Result as FmtResult};

use jl_sys::{jl_bool_type, jl_unbox_int8};

use crate::{
    convert::{construct_type::ConstructType, unbox::Unbox},
    data::managed::{datatype::DataTypeData, private::ManagedPriv, value::Value},
    impl_julia_typecheck, impl_valid_layout,
    memory::target::ExtendedTarget,
    prelude::Target,
    private::Private,
};

/// A Julia `Bool`.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bool(i8);

impl Bool {
    #[inline(always)]
    pub fn new(val: bool) -> Self {
        Bool(val as i8)
    }

    #[inline(always)]
    /// Returns the value of the `Bool` as a `i8`.
    pub fn as_i8(self) -> i8 {
        self.0
    }

    #[inline(always)]
    /// Returns the value of the `Bool` as a `bool` if it's 0 or 1, `None` if it isn't.
    pub fn try_as_bool(self) -> Option<bool> {
        match self.0 {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    /// Returns the value of the `Bool` as a `bool`.
    #[inline(always)]
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

    #[inline(always)]
    unsafe fn unbox(value: Value) -> Bool {
        Bool(jl_unbox_int8(value.unwrap(Private).cast()))
    }
}

unsafe impl Unbox for bool {
    type Output = Bool;

    #[inline(always)]
    unsafe fn unbox(value: Value) -> Bool {
        Bool(jl_unbox_int8(value.unwrap(Private).cast()))
    }
}

unsafe impl ConstructType for Bool {
    fn base_type<'target, T>(target: &T) -> crate::data::managed::value::Value<'target, 'static>
    where
        T: Target<'target>,
    {
        unsafe { <bool as crate::convert::into_julia::IntoJulia>::julia_type(target).as_value() }
    }

    fn construct_type<'target, 'current, 'borrow, T>(
        target: ExtendedTarget<'target, 'current, 'borrow, T>,
    ) -> DataTypeData<'target, T>
    where
        T: Target<'target>,
    {
        let (target, _) = target.split();
        <bool as crate::convert::into_julia::IntoJulia>::julia_type(target)
    }
}

impl_ccall_arg!(Bool);
