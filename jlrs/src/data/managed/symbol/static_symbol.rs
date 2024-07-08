//! Static references to `Symbol`s.

use std::{
    ffi::{c_char, c_void},
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ptr::NonNull,
};

use jl_sys::jl_symbol_n;

use crate::{
    data::{
        managed::{private::ManagedPriv, symbol::Symbol},
        types::construct_type::{ConstructType, TypeVarName},
    },
    memory::target::{unrooted::Unrooted, Target},
    prelude::Managed,
    private::Private,
};

/// Define a new implementation of `StaticSymbol`.
///
/// `StaticSymbol`s are the fastest way that `Symbol`s can be accessed in jlrs, you use them
/// by calling [`StaticSymbol::get_symbol`].
///
/// Example:
///
/// ```
/// # use jlrs::prelude::*;
/// # use jlrs::data::managed::array::{TypedArray, TypedRankedArray};
/// # use jlrs::define_static_symbol;
/// # use jlrs::data::managed::symbol::static_symbol::{StaticSymbol, sym};
///
/// struct Bar;
///
/// // Define for existing struct.
/// define_static_symbol!(for Bar, "Bar");
///
/// // Define for new struct. The struct is defined as a unit type, i.e. `struct Where;`
/// define_static_symbol!(Where, "where");
///
/// // Attributes and visibility modifiers are accepted when a new struct is defined.
/// define_static_symbol!(
///     /// The subtype operator `:<:`.
///     pub SubtypeOperator,
///     "<:"
/// );
///
/// # fn main() {
/// # let mut julia = Builder::new().start_local().unwrap();
/// julia
///     .local_scope::<_, 0>(|frame| {
///         let where_sym = Bar::get_symbol(&frame);
///         let sym = sym::<Bar, _>(&frame);
///         assert_eq!(where_sym, sym);
///     });
/// # }
/// ```
#[macro_export]
macro_rules! define_static_symbol {
    (for $name:ident, $sym:literal) => {
        unsafe impl $crate::data::managed::symbol::static_symbol::StaticSymbol for $name {
            #[inline]
            fn get_symbol<'target, Tgt: $crate::memory::target::Target<'target>>(_: &Tgt) -> $crate::data::managed::symbol::Symbol<'target> {
                static PTR: ::std::sync::atomic::AtomicPtr<::std::ffi::c_void> = ::std::sync::atomic::AtomicPtr::new(::std::ptr::null_mut());

                #[cold]
                #[inline(never)]
                unsafe fn init() -> $crate::data::managed::symbol::Symbol<'static> {
                    const N: usize = $sym.as_bytes().len();
                    const INNER_PTR: *mut ::std::ffi::c_char = $sym.as_ptr() as *const ::std::ffi::c_char as *mut ::std::ffi::c_char;
                    let ptr = $crate::data::managed::symbol::static_symbol::new_symbol(INNER_PTR, N);
                    PTR.store(ptr, ::std::sync::atomic::Ordering::Relaxed);
                    $crate::data::managed::symbol::static_symbol::convert_void_ptr(ptr)
                }

                fn inner() -> $crate::data::managed::symbol::Symbol<'static> {
                    let ptr = PTR.load(::std::sync::atomic::Ordering::Relaxed);
                    unsafe {
                        if ptr.is_null() {
                            init()
                        } else {
                            $crate::data::managed::symbol::static_symbol::convert_void_ptr(ptr)
                        }
                    }
                }

                inner()
            }
        }
    };
    ($(#[$meta:meta])* $vis:vis $name:ident, $sym:literal) => {
        $(#[$meta])*
        $vis struct $name;
        $crate::define_static_symbol!(for $name, $sym);
    };
}

/// Same as [`define_static_symbol`] but accepts byte string literals instead of string literals.
#[macro_export]
macro_rules! define_static_binary_symbol {
    (for $name:ident, $sym:literal) => {
        unsafe impl $crate::data::managed::symbol::static_symbol::StaticSymbol for $name {
            #[inline]
            fn get_symbol<'target, Tgt: $crate::memory::target::Target<'target>>(_: &Tgt) -> $crate::data::managed::symbol::Symbol<'target> {
                static PTR: ::std::sync::atomic::AtomicPtr<::std::ffi::c_void> = ::std::sync::atomic::AtomicPtr::new(::std::ptr::null_mut());

                #[cold]
                #[inline(never)]
                unsafe fn init() -> $crate::data::managed::symbol::Symbol<'static> {
                    const N: usize = $sym.len();
                    const INNER_PTR: *mut ::std::ffi::c_char = $sym.as_ptr() as *const ::std::ffi::c_char as *mut ::std::ffi::c_char;
                    let ptr = $crate::data::managed::symbol::static_symbol::new_symbol(INNER_PTR, N);
                    PTR.store(ptr, ::std::sync::atomic::Ordering::Relaxed);
                    $crate::data::managed::symbol::static_symbol::convert_void_ptr(ptr)
                }

                fn inner() -> $crate::data::managed::symbol::Symbol<'static> {
                    let ptr = PTR.load(::std::sync::atomic::Ordering::Relaxed);
                    unsafe {
                        if ptr.is_null() {
                            init()
                        } else {
                            $crate::data::managed::symbol::static_symbol::convert_void_ptr(ptr)
                        }
                    }
                };

                inner()
            }
        }
    };
    ($(#[$meta:meta])* $vis:vis $name:ident, $sym:literal) => {
        $(#[$meta])*
        $vis struct $name;
        $crate::define_static_binary_symbol!(for $name, $sym);
    };
}

/// Trait implemented by types that encode a `Symbol`.
///
/// New implementations of this trait must be created with [`define_static_symbol`].
pub unsafe trait StaticSymbol: 'static {
    /// Returns the symbol encoded by this type.
    fn get_symbol<'target, Tgt: Target<'target>>(target: &Tgt) -> Symbol<'target>;
}

/// Helper struct that wraps a `StaticSymbol`
///
/// This type implements [`ToSymbol`], [`ConstructType`], and [`TypeVarName`], `Hash`, and
/// `PartialEq`. In the case of `PartialEq`, a `Sym` can be compared with `Symbols`, other `Sym`s,
/// and types that implement `StaticSymbol`. The hash is guaranteed to be the same as the
/// hash of the symbol.
///
/// [`ToSymbol`]: crate::convert::to_symbol::ToSymbol
#[repr(transparent)]
pub struct Sym<'target, S>(S, PhantomData<&'target ()>);

impl<'target, S: StaticSymbol> Sym<'target, S> {
    /// Convert an instance of an implementation of [`StaticSymbol`] to `Sym`.
    ///
    /// If you want to use the type rather than an instance, use [`sym`] instead.
    ///
    /// Safety: Must be called from a thread known to Julia, the result must only be used while
    /// Julia is active.
    #[inline]
    pub fn new<Tgt>(_: &Tgt, s: S) -> Self
    where
        Tgt: Target<'target>,
    {
        Sym(s, PhantomData)
    }

    /// Extract the instance of an implementation of [`StaticSymbol`] from a `self`.
    #[inline]
    pub fn take(self) -> S {
        self.0
    }
}

/// Convert `S` to `Sym<PhantomData<S>>`.
#[inline]
pub fn sym<'target, S, Tgt>(_: &Tgt) -> Sym<'target, PhantomData<S>>
where
    S: StaticSymbol,
    Tgt: Target<'target>,
{
    Sym(PhantomData, PhantomData)
}

unsafe impl<S: StaticSymbol> ConstructType for Sym<'_, S> {
    type Static = Sym<'static, S>;

    const CACHEABLE: bool = false;

    #[inline]
    fn construct_type_uncached<'target, Tgt>(
        target: Tgt,
    ) -> crate::prelude::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        S::get_symbol(&target).as_value().root(target)
    }

    #[inline]
    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _: &crate::data::types::construct_type::TypeVarEnv,
    ) -> crate::prelude::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        S::get_symbol(&target).as_value().root(target)
    }

    #[inline]
    fn base_type<'target, Tgt>(_: &Tgt) -> Option<crate::prelude::Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        None
    }
}

unsafe impl<S: StaticSymbol> ConstructType for Sym<'_, PhantomData<S>> {
    type Static = Sym<'static, S>;

    const CACHEABLE: bool = false;

    #[inline]
    fn construct_type_uncached<'target, Tgt>(
        target: Tgt,
    ) -> crate::prelude::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        S::get_symbol(&target).as_value().root(target)
    }

    #[inline]
    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _: &crate::data::types::construct_type::TypeVarEnv,
    ) -> crate::prelude::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        S::get_symbol(&target).as_value().root(target)
    }

    #[inline]
    fn base_type<'target, Tgt>(_: &Tgt) -> Option<crate::prelude::Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        None
    }
}

impl<S: StaticSymbol> TypeVarName for Sym<'static, S> {
    #[inline]
    fn symbol<'target, Tgt: Target<'target>>(target: &Tgt) -> Symbol<'target> {
        S::get_symbol(target)
    }
}

impl<S: StaticSymbol> TypeVarName for Sym<'static, PhantomData<S>> {
    #[inline]
    fn symbol<'target, Tgt: Target<'target>>(target: &Tgt) -> Symbol<'target> {
        S::get_symbol(target)
    }
}

impl<S: StaticSymbol> Hash for Sym<'_, S> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            let unrooted = Unrooted::new();
            let s = S::get_symbol(&unrooted);
            <Symbol as Hash>::hash(&s, state)
        }
    }
}

impl<S: StaticSymbol> Hash for Sym<'_, PhantomData<S>> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            let unrooted = Unrooted::new();
            let s = S::get_symbol(&unrooted);
            <Symbol as Hash>::hash(&s, state)
        }
    }
}

impl<S: StaticSymbol> Debug for Sym<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_tuple("Sym")
                .field(&S::get_symbol(&Unrooted::new()))
                .finish()
        }
    }
}

impl<S: StaticSymbol> Debug for Sym<'_, PhantomData<S>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_tuple("Sym")
                .field(&S::get_symbol(&Unrooted::new()))
                .finish()
        }
    }
}

impl<S: StaticSymbol, T: StaticSymbol> PartialEq<T> for Sym<'_, S> {
    fn eq(&self, _: &T) -> bool {
        unsafe {
            let unrooted = Unrooted::new();
            let this = S::get_symbol(&unrooted);
            let other = T::get_symbol(&unrooted);
            this == other
        }
    }
}

impl<S: StaticSymbol, T: StaticSymbol> PartialEq<Sym<'_, T>> for Sym<'_, S> {
    fn eq(&self, _: &Sym<T>) -> bool {
        unsafe {
            let unrooted = Unrooted::new();
            let this = S::get_symbol(&unrooted);
            let other = T::get_symbol(&unrooted);
            this == other
        }
    }
}

impl<S: StaticSymbol, T: StaticSymbol> PartialEq<Sym<'_, PhantomData<T>>> for Sym<'_, S> {
    fn eq(&self, _: &Sym<PhantomData<T>>) -> bool {
        unsafe {
            let unrooted = Unrooted::new();
            let this = S::get_symbol(&unrooted);
            let other = T::get_symbol(&unrooted);
            this == other
        }
    }
}

impl<S: StaticSymbol, T: StaticSymbol> PartialEq<T> for Sym<'_, PhantomData<S>> {
    fn eq(&self, _: &T) -> bool {
        unsafe {
            let unrooted = Unrooted::new();
            let this = S::get_symbol(&unrooted);
            let other = T::get_symbol(&unrooted);
            this == other
        }
    }
}

impl<S: StaticSymbol, T: StaticSymbol> PartialEq<Sym<'_, T>> for Sym<'_, PhantomData<S>> {
    fn eq(&self, _: &Sym<T>) -> bool {
        unsafe {
            let unrooted = Unrooted::new();
            let this = S::get_symbol(&unrooted);
            let other = T::get_symbol(&unrooted);
            this == other
        }
    }
}

impl<S: StaticSymbol, T: StaticSymbol> PartialEq<Sym<'_, PhantomData<T>>>
    for Sym<'_, PhantomData<S>>
{
    fn eq(&self, _: &Sym<PhantomData<T>>) -> bool {
        unsafe {
            let unrooted = Unrooted::new();
            let this = S::get_symbol(&unrooted);
            let other = T::get_symbol(&unrooted);
            this == other
        }
    }
}

impl<S: StaticSymbol> PartialEq<Symbol<'_>> for Sym<'_, S> {
    fn eq(&self, other: &Symbol<'_>) -> bool {
        unsafe {
            let unrooted = Unrooted::new();
            let this = S::get_symbol(&unrooted);
            this.0 == other.0
        }
    }
}

impl<S: StaticSymbol> PartialEq<Symbol<'_>> for Sym<'_, PhantomData<S>> {
    fn eq(&self, other: &Symbol<'_>) -> bool {
        unsafe {
            let unrooted = Unrooted::new();
            let this = S::get_symbol(&unrooted);
            this.0 == other.0
        }
    }
}

// Converts a void pointer to a symbol, the pointer must have been returned by `new_symbol`.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn convert_void_ptr(ptr: *mut c_void) -> Symbol<'static> {
    Symbol::wrap_non_null(NonNull::new_unchecked(ptr as *mut _), Private)
}

// Creates a new symbol, ptr and len must the pointer and length of a string slice `&str`.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn new_symbol<'target>(ptr: *mut c_char, len: usize) -> *mut c_void {
    jl_symbol_n(ptr, len) as *mut _
}

define_static_symbol!(pub NSym, "N");
define_static_symbol!(pub TSym, "T");
