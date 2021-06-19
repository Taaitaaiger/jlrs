//! Wrapper for `Core.Symbol`. Symbols represent identifiers like module and function names.

use crate::{
    error::{JlrsError, JlrsResult},
    impl_debug,
    memory::global::Global,
};
use crate::{impl_julia_typecheck, impl_valid_layout};
use crate::{private::Private, wrappers::ptr::value::LeakedValue};
use jl_sys::{jl_sym_t, jl_symbol_n, jl_symbol_name, jl_symbol_type};
use std::ffi::CStr;

use std::marker::PhantomData;
use std::ptr::NonNull;

use super::private::Wrapper;

/// `Symbol`s are used Julia to represent identifiers, `:x` represents the `Symbol` `x`. Things
/// that can be accessed using a `Symbol` include submodules, functions, and globals. However,
/// the methods that provide this functionality in jlrs can use strings instead.
///
/// This struct can be used in combination with [`DataType::is`] and [`Value::is`], if the check
/// returns` true` the [`Value`] can be cast to `Symbol`:
///
/// ```
/// # use jlrs::prelude::*;
/// # use jlrs::util::JULIA;
/// # fn main() {
/// # JULIA.with(|j| {
/// # let mut julia = j.borrow_mut();
/// julia.scope(|global, frame| {
///     let symbol_v = Symbol::new(global, "+").as_value();
///     assert!(symbol_v.is::<Symbol>());
///
///     let symbol = symbol_v.cast::<Symbol>()?;
///     assert!(Module::base(global).global(&mut *frame, symbol).is_ok());
///     Ok(())
/// }).unwrap();
/// # });
/// # }
/// ```
///
/// [`value::is`]: crate::wrappers::builtin::value::Value::is
/// [`DataType::is`]: crate::wrappers::builtin::datatype::DataType::is
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Symbol<'scope>(NonNull<jl_sym_t>, PhantomData<&'scope ()>);

impl<'scope> Symbol<'scope> {
    /// Convert the given string to a `Symbol`.
    pub fn new<S: AsRef<str>>(_: Global<'scope>, symbol: S) -> Self {
        unsafe {
            let sym_b = symbol.as_ref().as_bytes();
            let sym = jl_symbol_n(sym_b.as_ptr().cast(), sym_b.len());
            Symbol::wrap(sym, Private)
        }
    }

    /// Extend the `Symbol`'s lifetime. `Symbol`s are not garbage collected, but a `Symbol`
    /// returned as a [`Value`] from a Julia function inherits the frame's lifetime when it's cast
    /// to a `Symbol`. Its lifetime can be safely extended from `'scope` to `'global` using this
    /// method.
    pub fn extend<'global>(self, _: Global<'global>) -> Symbol<'global> {
        unsafe { Symbol::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }

    /// The hash of this `Symbol`.
    pub fn hash(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().hash }
    }

    /// `Symbol`s are stored using an invasive binary tree, this returns the left branch of the
    /// current node. This method is unsafe because it's not accessible from Julia except through
    /// the C API.
    pub unsafe fn left(self) -> Option<Symbol<'scope>> {
        let nn_self = self.unwrap_non_null(Private);
        let ref_self = nn_self.as_ref();

        if ref_self.left.is_null() {
            return None;
        }

        Some(Symbol::wrap(ref_self.left, Private))
    }

    /// `Symbol`s are stored using an invasive binary tree, this returns the right branch of the
    /// current node. This method is unsafe because it's not accessible from Julia except through
    /// the C API.
    pub unsafe fn right(self) -> Option<Symbol<'scope>> {
        let nn_self = self.unwrap_non_null(Private);
        let ref_self = nn_self.as_ref();

        if ref_self.right.is_null() {
            return None;
        }

        Some(Symbol::wrap(ref_self.right, Private))
    }

    /// Convert `self` to a `LeakedValue`.
    pub fn as_leaked(self) -> LeakedValue {
        unsafe { LeakedValue::wrap_non_null(self.unwrap_non_null(Private).cast()) }
    }

    /// Convert `self` to a `String`.
    pub fn as_string(self) -> JlrsResult<String> {
        self.as_str().map(Into::into)
    }

    /// View `self` as a string slice. Returns an error if the symbol is not valid UTF8.
    pub fn as_str(self) -> JlrsResult<&'scope str> {
        unsafe {
            let ptr = jl_symbol_name(self.unwrap(Private)).cast();
            let symbol = CStr::from_ptr(ptr);
            symbol.to_str().map_err(|_| Box::new(JlrsError::NotUnicode))
        }
    }

    /// View `self` as an slice of bytes without the trailing null.
    pub fn as_slice(self) -> &'scope [u8] {
        unsafe {
            let ptr = jl_symbol_name(self.unwrap(Private)).cast();
            let symbol = CStr::from_ptr(ptr);
            symbol.to_bytes()
        }
    }
}

impl_julia_typecheck!(Symbol<'scope>, jl_symbol_type, 'scope);
impl_debug!(Symbol<'_>);
impl_valid_layout!(Symbol<'scope>, 'scope);

impl<'scope> Wrapper<'scope, '_> for Symbol<'scope> {
    type Internal = jl_sym_t;
    const NAME: &'static str = "Symbol";

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
