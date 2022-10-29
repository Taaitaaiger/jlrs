//! Wrapper for `Symbol`. Symbols represent identifiers like module and function names.

use crate::{
    error::{JlrsError, JlrsResult},
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{private::WrapperPriv, value::LeakedValue},
};
use jl_sys::{jl_sym_t, jl_symbol_n, jl_symbol_name_ as jl_symbol_name, jl_symbol_type};
use std::{
    ffi::CStr,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ptr::NonNull,
};

#[cfg(not(all(target_os = "windows", feature = "lts")))]
use crate::catch::catch_exceptions;
#[cfg(not(all(target_os = "windows", feature = "lts")))]
use std::mem::MaybeUninit;

use super::Ref;

/// `Symbol`s are used Julia to represent identifiers, `:x` represents the `Symbol` `x`. Things
/// that can be accessed using a `Symbol` include submodules, functions, and globals. However,
/// the methods that provide this functionality in jlrs can use strings instead. They're also used
/// as the building-block of expressions.
///
/// One special property of `Symbol`s is that they're never freed by the garbage collector after
/// they've been created.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Symbol<'scope>(NonNull<jl_sym_t>, PhantomData<&'scope ()>);

impl<'scope> Symbol<'scope> {
    /// Convert the given string to a `Symbol`.
    pub fn new<S, T>(_: &T, symbol: S) -> Self
    where
        S: AsRef<str>,
        T: Target<'scope>,
    {
        let bytes = symbol.as_ref().as_bytes();
        // Safety: Can only be called from a thread known to Julia, symbols are globally rooted
        unsafe {
            let sym = jl_symbol_n(bytes.as_ptr().cast(), bytes.len());
            Symbol::wrap(sym, Private)
        }
    }

    /// Convert the given byte slice to a `Symbol`.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]

    pub fn new_bytes<N, T>(target: T, symbol: N) -> T::Exception<'static, Self>
    where
        N: AsRef<[u8]>,
        T: Target<'scope>,
    {
        unsafe {
            let symbol = symbol.as_ref();

            let mut callback = |result: &mut MaybeUninit<*mut jl_sym_t>| {
                let sym = jl_symbol_n(symbol.as_ptr().cast(), symbol.len());
                result.write(sym);
                Ok(())
            };

            let res = match catch_exceptions(&mut callback).unwrap() {
                Ok(sym) => Ok(Symbol::wrap(sym, Private)),
                Err(e) => Err(NonNull::new_unchecked(e.ptr())),
            };

            target.exception_from_ptr(res, Private)
        }
    }

    /// Convert the given byte slice to a `Symbol`.
    ///
    /// Safety: if `symbol` contains `0`, an error is thrown which is not caught.
    pub unsafe fn new_bytes_unchecked<S, T>(_: &T, symbol: S) -> Self
    where
        S: AsRef<[u8]>,
        T: Target<'scope>,
    {
        let sym_b = symbol.as_ref();
        let sym = jl_symbol_n(sym_b.as_ptr().cast(), sym_b.len());
        Symbol::wrap(sym, Private)
    }

    /// Extend the `Symbol`'s lifetime. A `Symbol` is never freed by the garbage collector, its
    /// lifetime can be safely extended.
    ///
    /// [`Value`]: crate::wrappers::ptr::value::Value
    pub fn extend<'target, T>(self, _: &T) -> Symbol<'target>
    where
        T: Target<'target>,
    {
        // Safety: symbols are globally rooted
        unsafe { Symbol::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }

    /// The hash of this `Symbol`.
    pub fn hash(self) -> usize {
        // Safety: symbols are globally rooted
        unsafe { self.unwrap_non_null(Private).as_ref().hash }
    }

    /// Convert `self` to a `LeakedValue`.
    pub fn as_leaked(self) -> LeakedValue {
        // Safety: symbols are globally rooted
        unsafe { LeakedValue::wrap_non_null(self.unwrap_non_null(Private).cast()) }
    }

    /// Convert `self` to a `String`.
    pub fn as_string(self) -> JlrsResult<String> {
        self.as_str().map(Into::into)
    }

    /// View `self` as a string slice. Returns an error if the symbol is not valid UTF8.
    pub fn as_str(self) -> JlrsResult<&'scope str> {
        // Safety: symbols are globally rooted
        unsafe {
            let ptr = jl_symbol_name(self.unwrap(Private)).cast();
            let symbol = CStr::from_ptr(ptr);
            Ok(symbol.to_str().map_err(JlrsError::other)?)
        }
    }

    /// View `self` as a `Cstr`.
    pub fn as_cstr(self) -> &'scope CStr {
        // Safety: symbols are globally rooted
        unsafe {
            let ptr = jl_symbol_name(self.unwrap(Private));
            &CStr::from_ptr(ptr.cast())
        }
    }

    /// View `self` as an slice of bytes without the trailing null.
    pub fn as_bytes(self) -> &'scope [u8] {
        // Safety: symbols are globally rooted
        unsafe {
            let ptr = jl_symbol_name(self.unwrap(Private)).cast();
            let symbol = CStr::from_ptr(ptr);
            symbol.to_bytes()
        }
    }

    /// Use the target to reroot this data. This is never nevessary
    /// because a `Symbol` is never freed by the garbage collector.
    pub fn root<'target, T>(self, target: T) -> SymbolData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the data is valid.
        // TODO: don' root
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
    }
}

impl Hash for Symbol<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize((*self).hash())
    }
}

impl_julia_typecheck!(Symbol<'scope>, jl_symbol_type, 'scope);
impl_debug!(Symbol<'_>);

impl<'scope> WrapperPriv<'scope, '_> for Symbol<'scope> {
    type Wraps = jl_sym_t;
    type TypeConstructorPriv<'target, 'da> = Symbol<'target>;
    const NAME: &'static str = "Symbol";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(Symbol, 1);

/// A reference to a [`Symbol`] that has not been explicitly rooted.
pub type SymbolRef<'scope> = Ref<'scope, 'static, Symbol<'scope>>;
impl_valid_layout!(SymbolRef, Symbol);
impl_ref_root!(Symbol, SymbolRef, 1);

use crate::memory::target::target_type::TargetType;
pub type SymbolData<'target, T> = <T as TargetType<'target>>::Data<'static, Symbol<'target>>;
pub type SymbolResult<'target, T> = <T as TargetType<'target>>::Result<'static, Symbol<'target>>;
