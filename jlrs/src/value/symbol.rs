//! Symbols are interned strings in Julia, used in jlrs when accessing globals.

use crate::global::Global;
use jl_sys::{jl_sym_t, jl_symbol_n, jl_symbol_name};
use std::ffi::CStr;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;

/// In Julia many things are built from `Symbol`s. In jlrs, the only current use is accessing
/// globals.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Symbol<'base>(*mut jl_sym_t, PhantomData<&'base ()>);

impl<'base> Symbol<'base> {
    pub(crate) unsafe fn wrap(ptr: *mut jl_sym_t) -> Self {
        Symbol(ptr, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_sym_t {
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
        unsafe { Symbol::wrap(self.ptr()) }
    }
}

impl<'base> Into<String> for Symbol<'base> {
    fn into(self) -> String {
        unsafe {
            let ptr = jl_symbol_name(self.ptr()).cast();
            let symbol = CStr::from_ptr(ptr);
            symbol.to_str().unwrap().into()
        }
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
            let ptr = jl_symbol_name(self.ptr()).cast();
            let symbol = CStr::from_ptr(ptr);
            f.debug_tuple("Symbol").field(&symbol).finish()
        }
    }
}
