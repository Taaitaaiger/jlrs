//! Automatically convert strings to symbols.
//!
//! Many things in Julia are accessed with [`Symbol`]s, the [`TemporarySymbol`] trait
//! allows for strings to be used instead.

use crate::wrappers::ptr::string::JuliaString;
use crate::wrappers::ptr::symbol::Symbol;

/// Trait implemented by types that can be converted to a temporary [`Symbol`].
pub trait TemporarySymbol: private::TemporarySymbol {}
impl<T: AsRef<str>> TemporarySymbol for T {}
impl TemporarySymbol for Symbol<'_> {}
impl TemporarySymbol for JuliaString<'_> {}

pub(crate) mod private {
    use crate::private::Private;
    use crate::wrappers::ptr::private::Wrapper;
    use crate::wrappers::ptr::string::JuliaString;
    use crate::wrappers::ptr::symbol::Symbol;
    use jl_sys::{jl_symbol, jl_symbol_n};
    use std::ptr::NonNull;

    pub unsafe trait TemporarySymbol {
        unsafe fn temporary_symbol<'symbol>(&self, _: Private) -> Symbol<'symbol>;
    }

    unsafe impl<T: AsRef<str>> TemporarySymbol for T {
        unsafe fn temporary_symbol<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ref().as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.as_ref().len());
            Symbol::wrap_non_null(NonNull::new_unchecked(symbol), Private)
        }
    }

    unsafe impl TemporarySymbol for JuliaString<'_> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            let symbol_ptr = self.as_c_str();
            let symbol = jl_symbol(symbol_ptr.as_ptr());
            Symbol::wrap_non_null(NonNull::new_unchecked(symbol), Private)
        }
    }

    unsafe impl TemporarySymbol for Symbol<'_> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            Symbol::wrap_non_null(self.unwrap_non_null(Private), Private)
        }
    }
}
