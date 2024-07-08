//! Managed type for `Module`, which provides access to Julia's modules and their content.
//!
//! In Julia, each module introduces a separate global scope. There are three important "root"
//! modules, `Main`, `Base` and `Core`. Any Julia code that you include in jlrs is made available
//! relative to the `Main` module.

// todo: jl_new_module

use std::{any::TypeId, marker::PhantomData, ptr::NonNull};

use jl_sys::{
    jl_base_module, jl_core_module, jl_get_global, jl_is_const, jl_is_imported, jl_main_module,
    jl_module_t, jl_module_type, jl_set_const, jlrs_module_name, jlrs_module_parent,
    jlrs_set_global,
};
use jlrs_macros::julia_version;
use rustc_hash::FxHashMap;

use super::{
    erase_scope_lifetime,
    function::FunctionData,
    value::{ValueData, ValueResult, ValueUnbound},
    Managed, Ref,
};
use crate::{
    call::Call,
    catch::{catch_exceptions, unwrap_exc},
    convert::to_symbol::ToSymbol,
    data::{
        layout::nothing::Nothing,
        managed::{
            function::Function, private::ManagedPriv, symbol::Symbol, union_all::UnionAll,
            value::Value,
        },
        static_data::StaticRef,
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
    #[cfg_attr(
        not(any(
            feature = "local-rt",
            feature = "async-rt",
            feature = "multi-rt",
            feature = "ccall"
        )),
        allow(unused)
    )]
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
/// If you include your own Julia code with [`Julia::include`] or [`AsyncHandle::include`], its
/// contents are made available relative to `Main`.
///
/// The most important methods offered are those that let you access submodules, functions, and
/// other global values defined in the module.
///
/// [`Julia::include`]: crate::runtime::sync_rt::Julia::include
/// [`AsyncHandle::include`]: crate::runtime::handle::async_handle::AsyncHandle::include
#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Module<'scope>(NonNull<jl_module_t>, PhantomData<&'scope ()>);

impl<'scope> Module<'scope> {
    /// Returns the name of this module.
    #[inline]
    pub fn name(self) -> Symbol<'scope> {
        // Safety: the pointer points to valid data, the name is never null
        unsafe {
            let sym = jlrs_module_name(self.unwrap(Private));
            Symbol::wrap_non_null(NonNull::new_unchecked(sym), Private)
        }
    }

    /// Returns the parent of this module.
    #[inline]
    pub fn parent(self) -> Module<'scope> {
        // Safety: the pointer points to valid data, the parent is never null
        unsafe {
            let parent = jlrs_module_parent(self.unwrap(Private));
            Module(NonNull::new_unchecked(parent), PhantomData)
        }
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
            "JlrsCore" => Module::jlrs_core(&target),
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
    /// [`Julia::include`] or [`AsyncHandle::include`] its contents are made available relative to
    /// `Main`.
    ///
    /// [`Julia::include`]: crate::runtime::sync_rt::Julia::include
    /// [`AsyncHandle::include`]: crate::runtime::handle::async_handle::AsyncHandle::include
    #[inline]
    pub fn main<Tgt: Target<'scope>>(_: &Tgt) -> Self {
        // Safety: the Main module is globally rooted
        unsafe { Module::wrap_non_null(NonNull::new_unchecked(jl_main_module), Private) }
    }

    /// Returns a handle to Julia's `Core`-module.
    #[inline]
    pub fn core<Tgt: Target<'scope>>(_: &Tgt) -> Self {
        // Safety: the Core module is globally rooted
        unsafe { Module::wrap_non_null(NonNull::new_unchecked(jl_core_module), Private) }
    }

    /// Returns a handle to Julia's `Base`-module.
    #[inline]
    pub fn base<Tgt: Target<'scope>>(_: &Tgt) -> Self {
        // Safety: the Base module is globally rooted
        unsafe { Module::wrap_non_null(NonNull::new_unchecked(jl_base_module), Private) }
    }

    /// Returns a handle to the `JlrsCore`-module.
    #[inline]
    pub fn jlrs_core<Tgt: Target<'scope>>(target: &Tgt) -> Self {
        // This won't be called until jlrs has been initialized, which loads the JlrsCore module.
        static JLRS_CORE: StaticRef<Module> = StaticRef::new("Base.loaded_modules[Base.PkgId(Base.UUID(\"29be08bc-e5fd-4da2-bbc1-72011c6ea2c9\"), \"JlrsCore\")]");
        unsafe { JLRS_CORE.get_or_eval(target) }
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

    /// Returns the submodule named `name` relative to this module. You have to visit this level
    /// by level: you can't access `Main.A.B` by calling this function with `"A.B"`, but have to
    /// access `A` first and then `B`.
    ///
    /// Returns an error if the submodule doesn't exist.
    pub fn submodule<'target, N, Tgt>(
        self,
        target: Tgt,
        name: N,
    ) -> JlrsResult<ModuleData<'target, Tgt>>
    where
        N: ToSymbol,
        Tgt: Target<'target>,
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
    pub fn package_root_module<'target, N: ToSymbol, Tgt: Target<'target>>(
        target: &Tgt,
        name: N,
    ) -> Option<Module<'target>> {
        static FUNC: GcSafeOnceLock<unsafe extern "C" fn(Symbol) -> Value> = GcSafeOnceLock::new();
        unsafe {
            let func = FUNC.get_or_init(|| {
                let ptr = Module::jlrs_core(&target)
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
    pub unsafe fn set_global<'target, N, Tgt>(
        self,
        target: Tgt,
        name: N,
        value: Value<'_, 'static>,
    ) -> TargetException<'target, 'static, (), Tgt>
    where
        N: ToSymbol,
        Tgt: Target<'target>,
    {
        let symbol = name.to_symbol_priv(Private);

        let callback = || {
            jlrs_set_global(
                self.unwrap(Private),
                symbol.unwrap(Private),
                value.unwrap(Private),
            )
        };

        let res = catch_exceptions(callback, unwrap_exc);
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

        jlrs_set_global(
            self.unwrap(Private),
            symbol.unwrap(Private),
            value.unwrap(Private),
        );
    }

    /// Set a constant in this module. If Julia throws an exception it's caught and rooted in the
    /// current frame, if the exception can't be rooted a `JlrsError::AllocError` is returned. If
    /// no exception is thrown an unrooted reference to the constant is returned.
    pub fn set_const<'target, N, Tgt>(
        self,
        target: Tgt,
        name: N,
        value: Value<'_, 'static>,
    ) -> TargetException<'target, 'static, Value<'scope, 'static>, Tgt>
    where
        N: ToSymbol,
        Tgt: Target<'target>,
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

            let res = match catch_exceptions(callback, unwrap_exc) {
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
    pub fn global<'target, N, Tgt>(
        self,
        target: Tgt,
        name: N,
    ) -> JlrsResult<ValueData<'target, 'static, Tgt>>
    where
        N: ToSymbol,
        Tgt: Target<'target>,
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
    pub fn function<'target, N, Tgt>(
        self,
        target: Tgt,
        name: N,
    ) -> JlrsResult<FunctionData<'target, 'static, Tgt>>
    where
        N: ToSymbol,
        Tgt: Target<'target>,
    {
        // Safety: the pointer points to valid data, the result is checked.
        unsafe {
            let symbol = name.to_symbol_priv(Private);
            let func = self.global(&target, symbol)?.as_managed();

            if !func.is::<Function>() {
                let name = symbol.as_str().unwrap_or("<Non-UTF8 string>").into();
                let ty = func.datatype_name().into();
                Err(TypeError::NotAFunction { name, ty: ty })?;
            }

            Ok(target.data_from_ptr(func.unwrap_non_null(Private).cast(), Private))
        }
    }

    /// Returns `true` if `name` is a constant in this module.
    pub fn is_const<N>(self, name: N) -> bool
    where
        N: ToSymbol,
    {
        unsafe {
            let symbol = name.to_symbol_priv(Private);
            jl_is_const(self.unwrap(Private), symbol.unwrap(Private)) != 0
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
    pub unsafe fn require<'target, Tgt, N>(
        self,
        target: Tgt,
        module: N,
    ) -> ValueResult<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
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
    type WithLifetimes<'target, 'da> = Module<'target>;
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

/// `Module` or `ModuleRef`, depending on the target type `Tgt`.
pub type ModuleData<'target, Tgt> = <Tgt as TargetType<'target>>::Data<'static, Module<'target>>;

/// `JuliaResult<Module>` or `JuliaResultRef<ModuleRef>`, depending on the target type `Tgt`.
pub type ModuleResult<'target, Tgt> = TargetResult<'target, 'static, Module<'target>, Tgt>;

impl_ccall_arg_managed!(Module, 1);
impl_into_typed!(Module);

pub struct JlrsCore;

impl JlrsCore {
    #[inline]
    pub fn module<'target, Tgt>(target: &Tgt) -> Module<'target>
    where
        Tgt: Target<'target>,
    {
        Module::jlrs_core(target)
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
    pub fn color<'target, Tgt>(target: &Tgt) -> Value<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(COLOR, Value, "JlrsCore.color", target)
    }

    #[inline]
    pub fn wait_main<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(
            SET_POOL_SIZE,
            Function,
            "JlrsCore.Threads.wait_main",
            target
        )
    }

    #[inline]
    pub fn notify_main<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(
            SET_POOL_SIZE,
            Function,
            "JlrsCore.Threads.notify_main",
            target
        )
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
    #[julia_version(since = "1.9")]
    pub fn delegated_task<'target, Tgt>(target: &Tgt) -> DataType<'target>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(DELEGATED_TASK, DataType, "JlrsCore.DelegatedTask", target)
    }

    #[inline]
    pub fn background_task<'target, Tgt>(target: &Tgt) -> UnionAll<'target>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(BACKGROUND_TASK, UnionAll, "JlrsCore.BackgroundTask", target)
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

    pub(crate) fn api_version<'target, Tgt>(target: &Tgt) -> isize
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(JLRS_API_VERSION, Value, "JlrsCore.JLRS_API_VERSION", target)
            .unbox::<isize>()
            .unwrap()
    }
}

pub struct Main;

impl Main {
    #[inline]
    pub fn include<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(INCLUDE, Function, "Main.include", target)
    }
}
