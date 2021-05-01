//! Symbols represent identifiers like module and function names.

use super::{LeakedValue, Value};
use crate::convert::cast::Cast;
use crate::{
    error::{JlrsError, JlrsResult},
    memory::global::Global,
};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_sym_t, jl_symbol_n, jl_symbol_name, jl_symbol_type};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::{convert::TryInto, ffi::CStr};

/// `Symbol`s are used Julia to represent identifiers, `:x` represents the `Symbol` `x`. Things
/// that can be accessed using a `Symbol` include submodules, functions, and globals. However,
/// the methods that provide this functionality in jlrs can use strings instead.
///
/// This struct implements [`JuliaTypecheck`] and [`Cast`]. It can be used in combination with
/// [`DataType::is`] and [`Value::is`]; if the check returns` true` the [`Value`] can be cast to
/// `Symbol`:
///
/// ```
/// # use jlrs::prelude::*;
/// # use jlrs::util::JULIA;
/// # fn main() {
/// # JULIA.with(|j| {
/// # let mut julia = j.borrow_mut();
/// julia.scope(|global, frame| {
///     let symbol_func = Module::core(global).function("Symbol")?;
///     let symbol_str = Value::new(&mut *frame, "+")?;
///     let symbol_val = symbol_func.call1(&mut *frame, symbol_str)?.unwrap();
///     assert!(symbol_val.is::<Symbol>());
///
///     let symbol = symbol_val.cast::<Symbol>()?;
///     assert!(Module::base(global).function(symbol).is_ok());
///     Ok(())
/// }).unwrap();
/// # });
/// # }
/// ```
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Symbol<'base>(NonNull<jl_sym_t>, PhantomData<&'base ()>);

impl<'base> Symbol<'base> {
    pub(crate) unsafe fn wrap(symbol: *mut jl_sym_t) -> Self {
        debug_assert!(!symbol.is_null());
        Symbol(NonNull::new_unchecked(symbol), PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_sym_t> {
        self.0
    }

    /// Convert the given string to a `Symbol`.
    pub fn new<S: AsRef<str>>(global: Global<'base>, symbol: S) -> Self {
        Symbol::from((global, symbol))
    }

    /// Extend the `Symbol`'s lifetime. `Symbol`s are not garbage collected, but a `Symbol`
    /// returned as a [`Value`] from a Julia function inherits the frame's lifetime when it's cast
    /// to a `Symbol`. Its lifetime can be safely extended from `'frame` to `'global` using this
    /// method.
    pub fn extend<'global>(self, _: Global<'global>) -> Symbol<'global> {
        unsafe { Symbol::wrap(self.inner().as_ptr()) }
    }

    /// The hash of this `Symbol`. This method is unsafe because it's not accessible from Julia
    /// except through the C API.
    pub unsafe fn hash(self) -> usize {
        (&*self.inner().as_ptr()).hash
    }

    /// `Symbol`s are stored using an invasive binary tree, this returns the left branch of the
    /// current node. This method is unsafe because it's not accessible from Julia except through
    /// the C API.
    pub unsafe fn left(self) -> Option<Symbol<'base>> {
        let ref_self = &*self.inner().as_ptr();
        if ref_self.left.is_null() {
            return None;
        }

        Some(Symbol::wrap(ref_self.left))
    }

    /// `Symbol`s are stored using an invasive binary tree, this returns the right branch of the
    /// current node. This method is unsafe because it's not accessible from Julia except through
    /// the C API.
    pub unsafe fn right(self) -> Option<Symbol<'base>> {
        let ref_self = &*self.inner().as_ptr();
        if ref_self.right.is_null() {
            return None;
        }

        Some(Symbol::wrap(ref_self.right))
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'base, 'static> {
        self.into()
    }

    /// Convert `self` to a `LeakedValue`.
    pub fn as_leaked(self) -> LeakedValue {
        unsafe { LeakedValue::wrap(self.inner().as_ptr().cast()) }
    }

    /// Convert `self` to a `String`.
    pub fn as_string(self) -> JlrsResult<String> {
        self.as_str().map(Into::into)
    }

    /// View `self` as a string slice. Returns an error if the symbol is not valid UTF8.
    pub fn as_str(self) -> JlrsResult<&'base str> {
        self.try_into()
    }

    /// View `self` as an slice of bytes without the trailing null.
    pub fn as_slice(self) -> &'base [u8] {
        unsafe {
            let ptr = jl_symbol_name(self.inner().as_ptr()).cast();
            let symbol = CStr::from_ptr(ptr);
            symbol.to_bytes()
        }
    }
}

impl<'base> TryInto<&'base str> for Symbol<'base> {
    type Error = Box<JlrsError>;
    fn try_into(self) -> JlrsResult<&'base str> {
        unsafe {
            let ptr = jl_symbol_name(self.inner().as_ptr()).cast();
            let symbol = CStr::from_ptr(ptr);
            symbol.to_str().map_err(|_| Box::new(JlrsError::NotUnicode))
        }
    }
}

impl<'base> Into<Value<'base, 'static>> for Symbol<'base> {
    fn into(self) -> Value<'base, 'static> {
        unsafe { Value::wrap(self.inner().as_ptr().cast()) }
    }
}

impl<'base, S> From<(Global<'base>, S)> for Symbol<'base>
where
    S: AsRef<str>,
{
    fn from((_, symbol): (Global<'base>, S)) -> Self {
        unsafe {
            let symbol_str = symbol.as_ref();
            let symbol_ptr = symbol_str.as_ptr();
            let symbol = jl_symbol_n(symbol_ptr.cast(), symbol_str.as_bytes().len());
            Symbol::wrap(symbol)
        }
    }
}

impl<'scope> Debug for Symbol<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        unsafe {
            let ptr = jl_symbol_name(self.inner().as_ptr()).cast();
            let symbol = CStr::from_ptr(ptr);
            f.debug_tuple("Symbol").field(&symbol).finish()
        }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Symbol<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotASymbol)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.inner().as_ptr().cast())
    }
}

impl_julia_typecheck!(Symbol<'frame>, jl_symbol_type, 'frame);
impl_julia_type!(Symbol<'frame>, jl_symbol_type, 'frame);
impl_valid_layout!(Symbol<'frame>, 'frame);
