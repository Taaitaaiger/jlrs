//! Managed type for `Module`, which provides access to Julia's modules and their content.
//!
//! In Julia, each module introduces a separate global scope. There are three important "root"
//! modules, `Main`, `Base` and `Core`. Any Julia code that you include in jlrs is made available
//! relative to the `Main` module.

// todo: jl_new_module

use std::{any::TypeId, marker::PhantomData, ptr::NonNull};

use fxhash::FxHashMap;
#[julia_version(since = "1.8", until = "1.9")]
use jl_sys::jl_binding_type as jl_get_binding_type;
#[julia_version(since = "1.10")]
use jl_sys::jl_get_binding_type;
use jl_sys::{
    jl_base_module, jl_core_module, jl_get_global, jl_is_imported, jl_main_module, jl_module_t,
    jl_module_type, jl_set_const, jl_set_global,
};
use jlrs_macros::julia_version;

use super::{
    erase_scope_lifetime,
    function::FunctionData,
    union_all::UnionAll,
    value::{ValueData, ValueResult, ValueUnbound},
    Managed, Ref,
};
use crate::{
    call::Call,
    catch::catch_exceptions,
    convert::to_symbol::ToSymbol,
    data::{
        layout::nothing::Nothing,
        managed::{function::Function, private::ManagedPriv, symbol::Symbol, value::Value},
        types::{construct_type::ConstructType, typecheck::Typecheck},
    },
    error::{AccessError, JlrsResult, TypeError},
    gc_safe::{GcSafeOnceLock, GcSafeRwLock},
    impl_julia_typecheck, inline_static_ref,
    memory::target::{Target, TargetException, TargetResult},
    prelude::DataType,
    private::Private,
};

struct GlobalCache {
    // FxHashMap is significantly faster than HashMap with default hasher
    // Boxed slice is faster to hash than a Vec
    data: GcSafeRwLock<FxHashMap<Box<[u8]>, (TypeId, ValueUnbound)>>,
}

impl GlobalCache {
    fn new() -> Self {
        GlobalCache {
            data: GcSafeRwLock::default(),
        }
    }
}

unsafe impl Send for GlobalCache {}
unsafe impl Sync for GlobalCache {}

static CACHE: GcSafeOnceLock<GlobalCache> = GcSafeOnceLock::new();

pub(crate) unsafe fn init_global_cache() {
    CACHE.set(GlobalCache::new()).ok();
}

/// Functionality in Julia can be accessed through its module system. You can get a handle to the
/// three standard modules, `Main`, `Base`, and `Core` and access their submodules through them.
/// If you include your own Julia code with [`Julia::include`] or [`AsyncJulia::include`], its
/// contents are made available relative to `Main`.
///
/// The most important methods offered are those that let you access submodules, functions, and
/// other global values defined in the module.
///
/// [`Julia::include`]: crate::runtime::sync_rt::Julia::include
/// [`AsyncJulia::include`]: crate::runtime::async_rt::AsyncJulia::include
#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Module<'scope>(NonNull<jl_module_t>, PhantomData<&'scope ()>);

impl<'scope> Module<'scope> {
    /// Returns the name of this module.
    #[inline]
    pub fn name(self) -> Symbol<'scope> {
        // Safety: the pointer points to valid data, the name is never null
        unsafe {
            let sym = NonNull::new_unchecked(self.unwrap_non_null(Private).as_ref().name);
            Symbol::wrap_non_null(sym, Private)
        }
    }

    /// Returns the parent of this module.
    #[inline]
    pub fn parent(self) -> Module<'scope> {
        // Safety: the pointer points to valid data, the parent is never null
        unsafe {
            let parent = self.unwrap_non_null(Private).as_ref().parent;
            Module(NonNull::new_unchecked(parent), PhantomData)
        }
    }

    /// Extend the lifetime of this module. This is safe as long as the module is never redefined.
    #[inline]
    pub unsafe fn extend<'target, T>(self, _: &T) -> Module<'target>
    where
        T: Target<'target>,
    {
        Module::wrap_non_null(self.unwrap_non_null(Private), Private)
    }

    /// Access the global at `path`. The result is cached for faster lookup in the future.
    ///
    /// Safety:
    ///
    /// This method assumes the global remains globally rooted. Only use this method to access
    /// module constants and globals which are never replaced with another value.
    #[inline(never)]
    pub unsafe fn typed_global_cached<'target, T, S, Tgt>(target: &Tgt, path: S) -> JlrsResult<T>
    where
        T: ConstructType + Managed<'target, 'static> + Typecheck,
        S: AsRef<str>,
        Tgt: Target<'target>,
    {
        let tid = T::type_id();
        let data = &CACHE.get_unchecked().data;
        let path = path.as_ref();

        {
            if let Some(cached) = data.read().get(path.as_bytes()) {
                if cached.0 == tid {
                    return Ok(cached.1.cast_unchecked());
                } else {
                    let ty = T::construct_type(target).as_value();
                    Err(TypeError::NotA {
                        value: cached.1.display_string_or("<Cannot display value>"),
                        field_type: ty.display_string_or("<Cannot display type>"),
                    })?
                }
            }
        }

        let mut parts = path.split('.');
        let n_parts = parts.clone().count();
        let module_name = parts.next().unwrap();

        let mut module = match module_name {
            "Main" => Module::main(&target),
            "Base" => Module::base(&target),
            "Core" => Module::core(&target),
            "JlrsCore" => JlrsCore::module(&target),
            module => {
                if let Some(module) = Module::package_root_module(&target, module) {
                    module
                } else {
                    Err(AccessError::ModuleNotFound {
                        module: module_name.into(),
                    })?
                }
            }
        };

        let item = match n_parts {
            1 => module.as_value().cast::<T>()?,
            2 => module
                .global(&target, parts.next().unwrap())?
                .as_value()
                .cast::<T>()?,
            n => {
                for _ in 1..n - 1 {
                    module = module
                        .submodule(&target, parts.next().unwrap())?
                        .as_managed();
                }

                module
                    .global(&target, parts.next().unwrap())?
                    .as_value()
                    .cast::<T>()?
            }
        };

        {
            data.write().insert(
                path.as_bytes().into(),
                (tid, erase_scope_lifetime(item.as_value())),
            );
        }

        Ok(item)
    }

    /// Returns a handle to Julia's `Main`-module. If you include your own Julia code with
    /// [`Julia::include`] or [`AsyncJulia::include`] its contents are made available relative to
    /// `Main`.
    ///
    /// [`Julia::include`]: crate::runtime::sync_rt::Julia::include
    /// [`AsyncJulia::include`]: crate::runtime::async_rt::AsyncJulia::include
    #[inline]
    pub fn main<T: Target<'scope>>(_: &T) -> Self {
        // Safety: the Main module is globally rooted
        unsafe { Module::wrap_non_null(NonNull::new_unchecked(jl_main_module), Private) }
    }

    /// Returns a handle to Julia's `Core`-module.
    #[inline]
    pub fn core<T: Target<'scope>>(_: &T) -> Self {
        // Safety: the Core module is globally rooted
        unsafe { Module::wrap_non_null(NonNull::new_unchecked(jl_core_module), Private) }
    }

    /// Returns a handle to Julia's `Base`-module.
    #[inline]
    pub fn base<T: Target<'scope>>(_: &T) -> Self {
        // Safety: the Base module is globally rooted
        unsafe { Module::wrap_non_null(NonNull::new_unchecked(jl_base_module), Private) }
    }

    /// Returns `true` if `self` has imported `sym`.
    #[inline]
    pub fn is_imported<N: ToSymbol>(self, sym: N) -> bool {
        // Safety: the pointer points to valid data, the C API function is called with
        // valid arguments.
        unsafe {
            let sym = sym.to_symbol_priv(Private);
            jl_is_imported(self.unwrap(Private), sym.unwrap(Private)) != 0
        }
    }

    #[julia_version(since = "1.8")]
    /// Returns the type of the binding in this module with the name `var`,
    #[inline]
    pub fn binding_type<'target, N, T>(
        self,
        target: T,
        var: N,
    ) -> Option<ValueData<'target, 'static, T>>
    where
        N: ToSymbol,
        T: Target<'target>,
    {
        let ptr = self.unwrap(Private);
        unsafe {
            let sym = var.to_symbol_priv(Private);
            let ty = jl_get_binding_type(ptr, sym.unwrap(Private));
            let ty = NonNull::new(ty)?;
            Some(target.data_from_ptr(ty, Private))
        }
    }

    /// Returns the submodule named `name` relative to this module. You have to visit this level
    /// by level: you can't access `Main.A.B` by calling this function with `"A.B"`, but have to
    /// access `A` first and then `B`.
    ///
    /// Returns an error if the submodule doesn't exist.
    pub fn submodule<'target, N, T>(self, target: T, name: N) -> JlrsResult<ModuleData<'target, T>>
    where
        N: ToSymbol,
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data, the C API function is called with
        // valid arguments and its result is checked.
        unsafe {
            let symbol = name.to_symbol_priv(Private);
            let submodule = jl_get_global(self.unwrap(Private), symbol.unwrap(Private));
            if submodule.is_null() {
                Err(AccessError::GlobalNotFound {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    module: self.name().as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            let submodule_nn = NonNull::new_unchecked(submodule);
            let submodule_v = Value::wrap_non_null(submodule_nn, Private);
            if !submodule_v.is::<Self>() {
                Err(TypeError::NotAModule {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    ty: submodule_v.datatype().name().into(),
                })?
            }

            Ok(target.data_from_ptr(submodule_nn.cast(), Private))
        }
    }

    /// Returns the root module of the package named `name`.
    ///
    /// All loaded packages can be accessed with this method. If the package doesn't exist or
    /// hasn't been loaded yet, `None` is returned.
    pub fn package_root_module<'target, N: ToSymbol, T: Target<'target>>(
        target: &T,
        name: N,
    ) -> Option<Module<'target>> {
        static FUNC: GcSafeOnceLock<unsafe extern "C" fn(Symbol) -> Value> = GcSafeOnceLock::new();
        unsafe {
            let func = FUNC.get_or_init(|| {
                let ptr = Module::main(&target)
                    .submodule(&target, "JlrsCore")
                    .unwrap()
                    .as_managed()
                    .global(&target, "root_module_c")
                    .unwrap()
                    .as_value()
                    .data_ptr()
                    .cast()
                    .as_ptr();

                *ptr
            });

            let name = name.to_symbol(&target);
            let module = func(name);
            if module.is::<Nothing>() {
                return None;
            }

            Some(module.cast_unchecked())
        }
    }

    /// Set a global value in this module. Note that if this global already exists, this can
    /// make the old value unreachable. If an excection is thrown, it's caught, rooted and
    /// returned.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    pub unsafe fn set_global<'target, N, T>(
        self,
        target: T,
        name: N,
        value: Value<'_, 'static>,
    ) -> TargetException<'target, 'static, (), T>
    where
        N: ToSymbol,
        T: Target<'target>,
    {
        let symbol = name.to_symbol_priv(Private);

        let callback = || {
            jl_set_global(
                self.unwrap(Private),
                symbol.unwrap(Private),
                value.unwrap(Private),
            )
        };

        let exc = |err: Value| err.unwrap_non_null(Private);
        let res = catch_exceptions(callback, exc);
        target.exception_from_ptr(res, Private)
    }

    /// Set a global value in this module. Note that if this global already exists, this can
    /// make the old value unreachable.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    #[inline]
    pub unsafe fn set_global_unchecked<N>(self, name: N, value: Value<'_, 'static>)
    where
        N: ToSymbol,
    {
        let symbol = name.to_symbol_priv(Private);

        jl_set_global(
            self.unwrap(Private),
            symbol.unwrap(Private),
            value.unwrap(Private),
        );
    }

    /// Set a constant in this module. If Julia throws an exception it's caught and rooted in the
    /// current frame, if the exception can't be rooted a `JlrsError::AllocError` is returned. If
    /// no exception is thrown an unrooted reference to the constant is returned.
    pub fn set_const<'target, N, T>(
        self,
        target: T,
        name: N,
        value: Value<'_, 'static>,
    ) -> TargetException<'target, 'static, Value<'scope, 'static>, T>
    where
        N: ToSymbol,
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data, the C API function is called with
        // valid arguments and its result is checked. if an exception is thrown it's caught
        // and returned
        unsafe {
            let symbol = name.to_symbol_priv(Private);

            let callback = || {
                jl_set_const(
                    self.unwrap(Private),
                    symbol.unwrap(Private),
                    value.unwrap(Private),
                );
            };

            let exc = |err: Value| err.unwrap_non_null(Private);

            let res = match catch_exceptions(callback, exc) {
                Ok(_) => Ok(Value::wrap_non_null(
                    value.unwrap_non_null(Private),
                    Private,
                )),
                Err(e) => Err(e),
            };

            target.exception_from_ptr(res, Private)
        }
    }

    /// Set a constant in this module. If the constant already exists the process aborts,
    /// otherwise an unrooted reference to the constant is returned.
    ///
    /// Safety: This method must not throw an error if called from a `ccall`ed function.
    #[inline]
    pub unsafe fn set_const_unchecked<N>(
        self,
        name: N,
        value: Value<'_, 'static>,
    ) -> Value<'scope, 'static>
    where
        N: ToSymbol,
    {
        let symbol = name.to_symbol_priv(Private);

        jl_set_const(
            self.unwrap(Private),
            symbol.unwrap(Private),
            value.unwrap(Private),
        );

        Value::wrap_non_null(value.unwrap_non_null(Private), Private)
    }

    /// Returns the global named `name` in this module.
    /// Returns an error if the global doesn't exist.
    pub fn global<'target, N, T>(
        self,
        target: T,
        name: N,
    ) -> JlrsResult<ValueData<'target, 'static, T>>
    where
        N: ToSymbol,
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data, the C API function is called with
        // valid arguments and its result is checked.
        unsafe {
            let symbol = name.to_symbol_priv(Private);

            let global = jl_get_global(self.unwrap(Private), symbol.unwrap(Private));
            if global.is_null() {
                Err(AccessError::GlobalNotFound {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    module: self.name().as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?;
            }

            Ok(target.data_from_ptr(NonNull::new_unchecked(global), Private))
        }
    }

    /// Returns the function named `name` in this module.
    /// Returns an error if the function doesn't exist or if it's not a subtype of `Function`.
    pub fn function<'target, N, T>(
        self,
        target: T,
        name: N,
    ) -> JlrsResult<FunctionData<'target, 'static, T>>
    where
        N: ToSymbol,
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data, the result is checked.
        unsafe {
            let symbol = name.to_symbol_priv(Private);
            let func = self.global(&target, symbol)?.as_managed();

            if !func.is::<Function>() {
                let name = symbol.as_str().unwrap_or("<Non-UTF8 string>").into();
                let ty = func.datatype_name().unwrap_or("<Non-UTF8 string>").into();
                Err(TypeError::NotAFunction { name, ty: ty })?;
            }

            Ok(target.data_from_ptr(func.unwrap_non_null(Private).cast(), Private))
        }
    }

    /// Load a module by calling `Base.require` and return this module if it has been loaded
    /// successfully. This method can be used to load parts of the standard library like
    /// `LinearAlgebra`. This requires one slot on the GC stack. Note that the loaded module is
    /// not made available in the module used to call this method, you can use
    /// `Module::set_global` to do so.
    ///
    /// Note that when you want to call `using Submodule` in the `Main` module, you can do so by
    /// evaluating the using-statement with [`Value::eval_string`].
    ///
    /// Safety: This method can execute arbitrary Julia code depending on the module that is
    /// loaded.
    pub unsafe fn require<'target, T, N>(
        self,
        target: T,
        module: N,
    ) -> ValueResult<'target, 'static, T>
    where
        T: Target<'target>,
        N: ToSymbol,
    {
        Module::typed_global_cached::<Value, _, _>(&target, "Base.require")
            .unwrap()
            .call2(
                target,
                self.as_value(),
                module.to_symbol_priv(Private).as_value(),
            )
    }
}

impl_julia_typecheck!(Module<'target>, jl_module_type, 'target);
impl_debug!(Module<'_>);

impl<'scope> ManagedPriv<'scope, '_> for Module<'scope> {
    type Wraps = jl_module_t;
    type TypeConstructorPriv<'target, 'da> = Module<'target>;
    const NAME: &'static str = "Module";

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

impl_construct_type_managed!(Module, 1, jl_module_type);

/// A reference to a [`Module`] that has not been explicitly rooted.
pub type ModuleRef<'scope> = Ref<'scope, 'static, Module<'scope>>;

/// A [`ModuleRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`Module`].
pub type ModuleRet = Ref<'static, 'static, Module<'static>>;

impl_valid_layout!(ModuleRef, Module, jl_module_type);

use crate::memory::target::TargetType;

/// `Module` or `ModuleRef`, depending on the target type `T`.
pub type ModuleData<'target, T> = <T as TargetType<'target>>::Data<'static, Module<'target>>;

/// `JuliaResult<Module>` or `JuliaResultRef<ModuleRef>`, depending on the target type `T`.
pub type ModuleResult<'target, T> = TargetResult<'target, 'static, Module<'target>, T>;

impl_ccall_arg_managed!(Module, 1);
impl_into_typed!(Module);

pub struct JlrsCore;

impl JlrsCore {
    #[inline]
    pub fn module<'target, Tgt>(target: &Tgt) -> Module<'target>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(MODULE, Module, "JlrsCore", target)
    }

    #[inline]
    pub fn borrow_error<'target, Tgt>(target: &Tgt) -> DataType<'target>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(BORROW_ERROR, DataType, "JlrsCore.BorrowError", target)
    }

    #[inline]
    pub fn jlrs_error<'target, Tgt>(target: &Tgt) -> DataType<'target>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(JLRS_ERROR, DataType, "JlrsCore.JlrsError", target)
    }

    #[inline]
    pub fn rust_result<'target, Tgt>(target: &Tgt) -> UnionAll<'target>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(RUST_RESULT, UnionAll, "JlrsCore.RustResult", target)
    }

    #[inline]
    pub fn set_pool_size<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(SET_POOL_SIZE, Function, "JlrsCore.set_pool_size", target)
    }

    #[inline]
    pub fn value_string<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(VALUE_STRING, Function, "JlrsCore.valuestring", target)
    }

    #[inline]
    pub fn error_string<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(ERROR_STRING, Function, "JlrsCore.errorstring", target)
    }

    #[inline]
    pub fn call_catch_wrapper<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(
            CALL_CATCH_WRAPPER,
            Function,
            "JlrsCore.call_catch_wrapper",
            target
        )
    }

    #[inline]
    pub fn call_catch_wrapper_c<'target, Tgt>(target: &Tgt) -> Value<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(
            CALL_CATCH_WRAPPER,
            Value,
            "JlrsCore.call_catch_wrapper_c",
            target
        )
    }

    #[cfg(feature = "async")]
    #[inline]
    pub(crate) fn async_call<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(ASYNC_CALL, Function, "JlrsCore.Threads.asynccall", target)
    }

    #[cfg(feature = "async")]
    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
    #[inline]
    pub(crate) fn interactive_call<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(
            INTERACTIVE_CALL,
            Function,
            "JlrsCore.Threads.interactivecall",
            target
        )
    }

    #[cfg(feature = "async")]
    #[inline]
    pub(crate) fn schedule_async<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(
            SCHEDULE_ASYNC,
            Function,
            "JlrsCore.Threads.scheduleasync",
            target
        )
    }

    #[cfg(feature = "async")]
    #[inline]
    pub(crate) fn schedule_async_local<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(
            SCHEDULE_ASYNC_LOCAL,
            Function,
            "JlrsCore.Threads.scheduleasynclocal",
            target
        )
    }

    #[cfg(feature = "async")]
    #[inline]
    pub(crate) fn post_blocking<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(
            POST_BLOCKING,
            Function,
            "JlrsCore.Threads.postblocking",
            target
        )
    }
}
