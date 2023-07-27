//! Managed type for `Symbol`. Symbols represent identifiers like module and function names.

use std::{
    ffi::CStr,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ptr::NonNull,
};

use fxhash::FxHashMap;
use jl_sys::{
    jl_gensym, jl_sym_t, jl_symbol_n, jl_symbol_name_ as jl_symbol_name, jl_symbol_type,
    jl_tagged_gensym,
};

use super::Ref;
use crate::{
    catch::catch_exceptions,
    data::managed::private::ManagedPriv,
    error::{JlrsError, JlrsResult},
    gc_safe::{GcSafeOnceLock, GcSafeRwLock},
    impl_julia_typecheck,
    memory::target::{Target, TargetException, TargetResult},
    prelude::Value,
    private::Private,
};

struct SymbolCache {
    data: GcSafeRwLock<FxHashMap<Vec<u8>, Symbol<'static>>>,
}

impl SymbolCache {
    fn new() -> Self {
        SymbolCache {
            data: GcSafeRwLock::default(),
        }
    }
}

unsafe impl Send for SymbolCache {}
unsafe impl Sync for SymbolCache {}

static CACHE: GcSafeOnceLock<SymbolCache> = GcSafeOnceLock::new();

pub(crate) unsafe fn init_symbol_cache() {
    CACHE.set(SymbolCache::new()).ok();
}

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
    #[inline]
    pub fn new<S, T>(_: &T, symbol: S) -> Self
    where
        S: AsRef<str>,
        T: Target<'scope>,
    {
        let bytes = symbol.as_ref().as_bytes();
        let data = unsafe { &CACHE.get_unchecked().data };

        {
            if let Some(sym) = data.read().get(bytes) {
                return *sym;
            }
        }

        // Safety: Can only be called from a thread known to Julia, symbols are globally rooted
        unsafe {
            let sym = jl_symbol_n(bytes.as_ptr().cast(), bytes.len());
            let sym = Symbol::wrap_non_null(NonNull::new_unchecked(sym), Private);
            data.write().insert(bytes.to_vec(), sym);
            sym
        }
    }

    /// Convert the given byte slice to a `Symbol`.
    pub fn new_bytes<N, T>(target: T, symbol: N) -> TargetException<'scope, 'static, Self, T>
    where
        N: AsRef<[u8]>,
        T: Target<'scope>,
    {
        let bytes = symbol.as_ref();
        let data = unsafe { &CACHE.get_unchecked().data };

        {
            if let Some(sym) = data.read().get(bytes) {
                unsafe {
                    return target.exception_from_ptr(Ok(*sym), Private);
                }
            }
        }

        unsafe {
            let callback = || jl_symbol_n(bytes.as_ptr().cast(), bytes.len());

            let exc = |err: Value| err.unwrap_non_null(Private);
            // let exc = |err: Value| Ok(Ref::<Value>::wrap(err.unwrap_non_null(Private)));

            match catch_exceptions(callback, exc) {
                Ok(sym) => {
                    let sym = Symbol::wrap_non_null(NonNull::new_unchecked(sym), Private);
                    data.write().insert(bytes.to_vec(), sym);

                    Ok(sym)
                }
                Err(e) => target.exception_from_ptr(Err(e), Private),
            }
        }
    }

    /// Convert the given byte slice to a `Symbol`.
    ///
    /// Safety: if `symbol` contains `0`, an error is thrown which is not caught.
    #[inline]
    pub unsafe fn new_bytes_unchecked<S, T>(_: &T, symbol: S) -> Self
    where
        S: AsRef<[u8]>,
        T: Target<'scope>,
    {
        let bytes = symbol.as_ref();
        let data = unsafe { &CACHE.get_unchecked().data };

        {
            if let Some(sym) = data.read().get(bytes) {
                return *sym;
            }
        }

        // Safety: Can only be called from a thread known to Julia, symbols are globally rooted
        unsafe {
            let sym = jl_symbol_n(bytes.as_ptr().cast(), bytes.len());
            let sym = Symbol::wrap_non_null(NonNull::new_unchecked(sym), Private);

            data.write().insert(bytes.to_vec(), sym);

            sym
        }
    }

    /// Generate a new unique `Symbol`.
    #[inline]
    pub fn generate<T>(_: &T) -> Self
    where
        T: Target<'scope>,
    {
        unsafe {
            let sym = jl_gensym();
            Symbol::wrap_non_null(NonNull::new_unchecked(sym), Private)
        }
    }

    /// Generate a new unique tagged `Symbol`.
    #[inline]
    pub fn generate_tagged<S, T>(_: &T, tag: S) -> Self
    where
        S: AsRef<str>,
        T: Target<'scope>,
    {
        unsafe {
            let tag = tag.as_ref().as_bytes();
            let sym = jl_tagged_gensym(tag.as_ptr() as _, tag.len());
            Symbol::wrap_non_null(NonNull::new_unchecked(sym), Private)
        }
    }

    /// Extend the `Symbol`'s lifetime. A `Symbol` is never freed by the garbage collector, its
    /// lifetime can be safely extended.
    ///
    /// [`Value`]: crate::data::managed::value::Value
    #[inline]
    pub fn extend<'target, T>(self, _: &T) -> Symbol<'target>
    where
        T: Target<'target>,
    {
        // Safety: symbols are globally rooted
        unsafe { Symbol::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }

    /// The hash of this `Symbol`.
    #[inline]
    pub fn hash(self) -> usize {
        // Safety: symbols are globally rooted
        unsafe { self.unwrap_non_null(Private).as_ref().hash }
    }

    /// Convert `self` to a `String`.
    #[inline]
    pub fn as_string(self) -> JlrsResult<String> {
        self.as_str().map(Into::into)
    }

    /// View `self` as a string slice. Returns an error if the symbol is not valid UTF8.
    #[inline]
    pub fn as_str(self) -> JlrsResult<&'scope str> {
        // Safety: symbols are globally rooted
        unsafe {
            let ptr = jl_symbol_name(self.unwrap(Private)).cast();
            let symbol = CStr::from_ptr(ptr);
            Ok(symbol.to_str().map_err(JlrsError::other)?)
        }
    }

    /// View `self` as a `Cstr`.
    #[inline]
    pub fn as_cstr(self) -> &'scope CStr {
        // Safety: symbols are globally rooted
        unsafe {
            let ptr = jl_symbol_name(self.unwrap(Private));
            &CStr::from_ptr(ptr.cast())
        }
    }

    /// View `self` as an slice of bytes without the trailing null.
    #[inline]
    pub fn as_bytes(self) -> &'scope [u8] {
        // Safety: symbols are globally rooted
        unsafe {
            let ptr = jl_symbol_name(self.unwrap(Private)).cast();
            let symbol = CStr::from_ptr(ptr);
            symbol.to_bytes()
        }
    }
}

impl Hash for Symbol<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize((*self).hash())
    }
}

impl_julia_typecheck!(Symbol<'scope>, jl_symbol_type, 'scope);
impl_debug!(Symbol<'_>);

impl<'scope> ManagedPriv<'scope, '_> for Symbol<'scope> {
    type Wraps = jl_sym_t;
    type TypeConstructorPriv<'target, 'da> = Symbol<'target>;
    const NAME: &'static str = "Symbol";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_construct_type_managed!(Symbol, 1, jl_symbol_type);

/// A reference to a [`Symbol`] that has not been explicitly rooted.
pub type SymbolRef<'scope> = Ref<'scope, 'static, Symbol<'scope>>;

/// A [`SymbolRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`Symbol`].
pub type SymbolRet = Ref<'static, 'static, Symbol<'static>>;

impl_valid_layout!(SymbolRef, Symbol, jl_symbol_type);

use crate::memory::target::TargetType;

/// `Task` or `TaskRef`, depending on the target type `T`.
pub type SymbolData<'target, T> = <T as TargetType<'target>>::Data<'static, Symbol<'target>>;

/// `JuliaResult<Task>` or `JuliaResultRef<TaskRef>`, depending on the target type `T`.
pub type SymbolResult<'target, T> = TargetResult<'target, 'static, Symbol<'target>, T>;

pub type SymbolUnbound = Symbol<'static>;

impl_ccall_arg_managed!(Symbol, 1);
impl_into_typed!(Symbol);
