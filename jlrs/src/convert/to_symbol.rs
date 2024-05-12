//! Convert strings to symbols.
//!
//! Many things in Julia are accessed with [`Symbol`]s, the [`ToSymbol`] trait allows for
//! strings to be used instead.

use std::marker::PhantomData;

use crate::{
    data::managed::{
        string::JuliaString,
        symbol::{
            static_symbol::{StaticSymbol, Sym},
            Symbol,
        },
    },
    memory::target::Target,
    private::Private,
};

/// Trait implemented by types that can be converted to a [`Symbol`].
pub trait ToSymbol: private::ToSymbolPriv {
    /// Convert `self` to a `Symbol`.
    ///
    /// This method only needs a reference to a target because `Symbol` are globally rooted.
    #[inline]
    fn to_symbol<'target, Tgt: Target<'target>>(&self, _: &Tgt) -> Symbol<'target> {
        // Safety: Requiring a reference to a target guarantees this method can only be called
        // from a thread known to Julia.
        unsafe { self.to_symbol_priv(Private) }
    }
}

impl<T: AsRef<str>> ToSymbol for T {}
impl ToSymbol for Symbol<'_> {}
impl ToSymbol for JuliaString<'_> {}
impl<S: StaticSymbol> ToSymbol for Sym<'_, PhantomData<S>> {}
impl<S: StaticSymbol> ToSymbol for Sym<'_, S> {}

pub(crate) mod private {
    use std::marker::PhantomData;

    use super::{StaticSymbol, Sym};
    use crate::{
        data::managed::{private::ManagedPriv, string::JuliaString, symbol::Symbol},
        memory::target::unrooted::Unrooted,
        private::Private,
    };

    pub trait ToSymbolPriv {
        // Safety: this method must only be called from a thread known to Julia
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol>;
    }

    impl<T: AsRef<str>> ToSymbolPriv for T {
        #[inline]
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            let unrooted = Unrooted::new();
            Symbol::new(&unrooted, self)
        }
    }

    impl<S: StaticSymbol> ToSymbolPriv for Sym<'_, S> {
        #[inline]
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            let unrooted = Unrooted::new();
            S::get_symbol(&unrooted)
        }
    }

    impl<S: StaticSymbol> ToSymbolPriv for Sym<'_, PhantomData<S>> {
        #[inline]
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            let unrooted = Unrooted::new();
            S::get_symbol(&unrooted)
        }
    }

    impl ToSymbolPriv for JuliaString<'_> {
        #[inline]
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            let symbol_ptr = self.as_bytes();
            let unrooted = Unrooted::new();
            Symbol::new_bytes(&unrooted, symbol_ptr).unwrap()
        }
    }

    impl ToSymbolPriv for Symbol<'_> {
        #[inline]
        unsafe fn to_symbol_priv<'symbol>(&self, _: Private) -> Symbol<'symbol> {
            Symbol::wrap_non_null(self.unwrap_non_null(Private), Private)
        }
    }
}
