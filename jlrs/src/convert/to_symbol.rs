//! Convert strings to symbols.
//!
//! Many things in Julia are accessed with [`Symbol`]s, the [`ToSymbol`] trait allows for
//! strings to be used instead.

use crate::wrappers::ptr::symbol::Symbol;
use crate::{memory::global::Global, private::Private, wrappers::ptr::string::JuliaString};

/// Trait implemented by types that can be converted to a [`Symbol`].
pub trait ToSymbol: private::ToSymbol {
    /// Convert `self` to a `Symbol`.
    fn to_symbol<'global>(&self, _: Global<'global>) -> Symbol<'global> {
        // Requiring a Global guarantees Julia has been initialized
        unsafe { self.to_symbol_priv(Private) }
    }
}
impl<T: AsRef<str>> ToSymbol for T {}
impl ToSymbol for Symbol<'_> {}
impl ToSymbol for JuliaString<'_> {}

pub(crate) mod private {
    use crate::private::Private;
    use crate::wrappers::ptr::private::Wrapper;
    use crate::wrappers::ptr::string::JuliaString;
    use crate::wrappers::ptr::symbol::Symbol;
    use jl_sys::{jl_symbol, jl_symbol_n};
    use std::ptr::NonNull;

    pub trait ToSymbol {
        // Safety: don't call this method before Julia has been initialized.
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol>;
    }

    impl<T: AsRef<str>> ToSymbol for T {
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ref().as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.as_ref().len());
            Symbol::wrap_non_null(NonNull::new_unchecked(symbol), Private)
        }
    }

    impl ToSymbol for JuliaString<'_> {
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            let symbol_ptr = self.as_c_str();
            let symbol = jl_symbol(symbol_ptr.as_ptr());
            Symbol::wrap_non_null(NonNull::new_unchecked(symbol), Private)
        }
    }

    impl ToSymbol for Symbol<'_> {
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            Symbol::wrap_non_null(self.unwrap_non_null(Private), Private)
        }
    }
}
