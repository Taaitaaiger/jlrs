//! Wrapper for `Symbol`. Symbols represent identifiers like module and function names.

use crate::{
    error::{JlrsError, JlrsResult},
    impl_debug, impl_julia_typecheck,
    memory::{global::Global, output::Output},
    private::Private,
    wrappers::ptr::{private::WrapperPriv, value::LeakedValue},
};
use cfg_if::cfg_if;
use jl_sys::{jl_sym_t, jl_symbol_n, jl_symbol_name_ as jl_symbol_name, jl_symbol_type};
use std::{ffi::CStr, marker::PhantomData, ptr::NonNull};

cfg_if! {
    if #[cfg(any(not(feature = "lts"), feature = "all-features-override"))] {
        use jl_sys::jl_value_t;
        use crate::wrappers::ptr::atomic_value;
        use std::sync::atomic::Ordering;
    }
}

/// `Symbol`s are used Julia to represent identifiers, `:x` represents the `Symbol` `x`. Things
/// that can be accessed using a `Symbol` include submodules, functions, and globals. However,
/// the methods that provide this functionality in jlrs can use strings instead. They're also used
/// as the building-block of expressions.
///
/// One special property of `Symbol`s is that they're never freed by the garbage collector after
/// they've been created.
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

    /// Extend the `Symbol`'s lifetime. A `Symbol` is never freed by the garbage collector, its
    /// lifetime can be safely extended.
    ///
    /// [`Value`]: crate::wrappers::ptr::value::Value
    pub fn extend<'global>(self, _: Global<'global>) -> Symbol<'global> {
        unsafe { Symbol::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }

    /// The hash of this `Symbol`.
    pub fn hash(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().hash }
    }

    /// `Symbol`s are stored using an invasive binary tree, this returns the left branch of the
    /// current node.
    pub fn left(self) -> Option<Symbol<'scope>> {
        cfg_if! {
            if #[cfg(all(feature = "lts", not(feature = "all-features-override")))] {
                unsafe {
                    let right = self.unwrap_non_null(Private).as_ref().left;

                    if right.is_null() {
                        return None;
                    }

                    Some(Symbol::wrap(right, Private))
                }
            } else {
                unsafe {
                    let left = atomic_value::<jl_value_t>(&self.unwrap_non_null(Private).as_mut().left as *const _);
                    let ptr = left.load(Ordering::Relaxed);

                    if ptr.is_null() {
                        return None;
                    }

                    Some(Symbol::wrap(ptr.cast(), Private))
                }
            }
        }
    }

    /// `Symbol`s are stored using an invasive binary tree, this returns the right branch of the
    /// current node.
    pub fn right(self) -> Option<Symbol<'scope>> {
        cfg_if! {
            if #[cfg(all(feature = "lts", not(feature = "all-features-override")))] {
                unsafe {
                    let right = self.unwrap_non_null(Private).as_ref().right;

                    if right.is_null() {
                        return None;
                    }

                    Some(Symbol::wrap(right, Private))
                }
            } else {
                unsafe {
                    let left = atomic_value::<jl_value_t>(&self.unwrap_non_null(Private).as_mut().right as *const _);
                    let ptr = left.load(Ordering::Relaxed);

                    if ptr.is_null() {
                        return None;
                    }

                    Some(Symbol::wrap(ptr.cast(), Private))
                }
            }
        }
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
            symbol.to_str().map_err(|_| Box::new(JlrsError::NotUTF8))
        }
    }

    /// View `self` as a `Cstr`.
    pub fn as_cstr(self) -> &'scope CStr {
        unsafe {
            let ptr = jl_symbol_name(self.unwrap(Private)).cast();
            &CStr::from_ptr(ptr)
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

    /// Use the `Output` to extend the lifetime of this data. This is never nevessary
    /// because a `Symbol` is never freed by the garbage collector.
    pub fn root<'target>(self, output: Output<'target>) -> Symbol<'target> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<Symbol>(ptr);
            Symbol::wrap_non_null(ptr, Private)
        }
    }
}

impl_julia_typecheck!(Symbol<'scope>, jl_symbol_type, 'scope);
impl_debug!(Symbol<'_>);

impl<'scope> WrapperPriv<'scope, '_> for Symbol<'scope> {
    type Wraps = jl_sym_t;
    const NAME: &'static str = "Symbol";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(Symbol, 1);
