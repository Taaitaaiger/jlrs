//! Convert strings to symbols.
//!
//! Many things in Julia are accessed with [`Symbol`]s, the [`ToSymbol`] trait allows for
//! strings to be used instead.

use crate::memory::target::Target;
use crate::wrappers::ptr::symbol::Symbol;
use crate::{private::Private, wrappers::ptr::string::JuliaString};

/// Trait implemented by types that can be converted to a [`Symbol`].
pub trait ToSymbol: private::ToSymbolPriv {
    /// Convert `self` to a `Symbol`.
    ///
    /// This method only needs a reference to a target because `Symbol` are globally rooted.
    fn to_symbol<'target, T: Target<'target, 'static>>(&self, _: &T) -> Symbol<'target> {
        // Safety: Requiring a reference to a target guarantees this method can only be called
        // from a thread known to Julia.
        unsafe { self.to_symbol_priv(Private) }
    }
}

impl<T: AsRef<str>> ToSymbol for T {}
impl ToSymbol for Symbol<'_> {}
impl ToSymbol for JuliaString<'_> {}

pub(crate) mod private {
    use crate::private::Private;
    use crate::wrappers::ptr::private::WrapperPriv;
    use crate::wrappers::ptr::string::JuliaString;
    use crate::wrappers::ptr::symbol::Symbol;
    use jl_sys::{jl_symbol, jl_symbol_n};
    use std::ptr::NonNull;

    pub trait ToSymbolPriv {
        // Safety: this method must only be called from a thread known to Julia
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol>;
    }

    impl<T: AsRef<str>> ToSymbolPriv for T {
        #[inline]
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ref().as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.as_ref().len());
            Symbol::wrap_non_null(NonNull::new_unchecked(symbol), Private)
        }
    }

    impl ToSymbolPriv for JuliaString<'_> {
        #[inline]
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            let symbol_ptr = self.as_c_str();
            let symbol = jl_symbol(symbol_ptr.as_ptr());
            Symbol::wrap_non_null(NonNull::new_unchecked(symbol), Private)
        }
    }

    impl ToSymbolPriv for Symbol<'_> {
        #[inline]
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            Symbol::wrap_non_null(self.unwrap_non_null(Private), Private)
        }
    }
}
