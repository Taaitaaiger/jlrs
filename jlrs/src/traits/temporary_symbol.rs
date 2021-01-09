use crate::value::string::JuliaString;
use crate::value::symbol::Symbol;
use std::borrow::Cow;

macro_rules! impl_temporary_symbol {
    ($type:ty, $($bounds:tt)+) => {
        unsafe impl<$($bounds)+> TemporarySymbol for $type {}
    };
    ($type:ty) => {
        unsafe impl TemporarySymbol for $type {}
    };
}
/// Trait implemented by types that can be converted to a temporary [`Symbol`].
///
/// [`Symbol`]: ../value/symbol/struct.Symbol.html
pub unsafe trait TemporarySymbol: private::TemporarySymbol {}

impl_temporary_symbol!(String);
impl_temporary_symbol!(&dyn AsRef<str>);
impl_temporary_symbol!(&'a str, 'a);
impl_temporary_symbol!(Cow<'a, str>, 'a);
impl_temporary_symbol!(Symbol<'s>, 's);
impl_temporary_symbol!(JuliaString<'frame>, 'frame);

pub(crate) mod private {
    use super::super::private::Internal;
    use crate::value::string::JuliaString;
    use crate::value::symbol::Symbol;
    use jl_sys::{jl_symbol, jl_symbol_n};
    use std::borrow::Cow;

    // safety: never return the symbol to the user without assigning the 'base lifetime.
    pub trait TemporarySymbol {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol>;
    }

    impl<'a> TemporarySymbol for &'a str {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ptr();
            let symbol = jl_symbol_n(symbol_ptr.cast(), self.len());
            Symbol::wrap(symbol)
        }
    }

    impl<'a> TemporarySymbol for Cow<'a, str> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.len());
            Symbol::wrap(symbol)
        }
    }

    impl TemporarySymbol for String {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.len());
            Symbol::wrap(symbol)
        }
    }

    impl TemporarySymbol for &dyn AsRef<str> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ref().as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.as_ref().len());
            Symbol::wrap(symbol)
        }
    }

    impl<'frame> TemporarySymbol for JuliaString<'frame> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_c_str();
            let symbol = jl_symbol(symbol_ptr.as_ptr());
            Symbol::wrap(symbol)
        }
    }

    impl<'frame> TemporarySymbol for Symbol<'frame> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            Symbol::wrap(self.ptr())
        }
    }
}
