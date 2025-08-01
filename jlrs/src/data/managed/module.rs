//! Managed type for `Module`, which provides access to Julia's modules and their content.
//!
//! In Julia, each module introduces a separate global scope. There are three important "root"
//! modules, `Main`, `Base` and `Core`. Any Julia code that you include in jlrs is made available
//! relative to the `Main` module.

use std::{any::TypeId, marker::PhantomData, ptr::NonNull};

use jl_sys::{
    jl_base_module, jl_core_module, jl_get_global, jl_is_const, jl_main_module, jl_module_t,
    jl_module_type, jl_set_const, jl_set_global, jlrs_module_name, jlrs_module_parent,
};
use rustc_hash::FxHashMap;

use super::{
    erase_scope_lifetime,
    function::FunctionData,
    value::{ValueData, ValueResult, ValueUnbound},
    Managed, Weak,
};
use crate::{
    call::Call,
    catch::{catch_exceptions, unwrap_exc},
    convert::to_symbol::ToSymbol,
    data::{
        cache::Cache,
        layout::nothing::Nothing,
        managed::{
            function::Function, private::ManagedPriv, symbol::Symbol, union_all::UnionAll,
            value::Value,
        },
        static_data::StaticRef,
        types::{construct_type::ConstructType, typecheck::Typecheck},
    },
    error::{AccessError, JlrsResult, TypeError},
    gc_safe::GcSafeOnceLock,
    impl_julia_typecheck, inline_static_ref,
    memory::{
        target::{Target, TargetException, TargetResult},
        PTls,
    },
    prelude::DataType,
    private::Private,
};

#[rustversion::before(1.85)]
type CacheImpl =
    crate::gc_safe::GcSafeOnceLock<Cache<FxHashMap<Box<[u8]>, (TypeId, ValueUnbound)>>>;
#[rustversion::since(1.85)]
type CacheImpl = Cache<FxHashMap<Box<[u8]>, (TypeId, ValueUnbound)>>;

#[rustversion::before(1.85)]
static CACHE: CacheImpl = CacheImpl::new();
#[rustversion::since(1.85)]
static CACHE: CacheImpl = CacheImpl::new({
    let hasher = rustc_hash::FxBuildHasher;
    std::collections::HashMap::with_hasher(hasher)
});

#[rustversion::before(1.85)]
pub(crate) unsafe fn init_global_cache() {
    CACHE.set(Default::default()).ok();
}

#[rustversion::since(1.85)]
pub(crate) unsafe fn init_global_cache() {}

#[rustversion::before(1.85)]
pub(crate) unsafe fn mark_global_cache(ptls: PTls, full: bool) {
    CACHE.get().map(|cache| cache.mark(ptls, full));
}

#[rustversion::since(1.85)]
pub(crate) unsafe fn mark_global_cache(ptls: PTls, full: bool) {
    CACHE.mark(ptls, full);
}

/// Functionality in Julia can be accessed through its module system. You can get a handle to the
/// three standard modules, `Main`, `Base`, and `Core` and access their submodules through them.
/// If you include your own Julia code with [`Runtime::include`], its
/// contents are made available relative to `Main`.
///
/// The most important methods offered are those that let you access submodules, functions, and
/// other global values defined in the module.
///
/// [`Runtime::include`]: crate::runtime::Runtime::include
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
        let cache = &CACHE.get_unchecked();

        let path = path.as_ref();
        let res = cache.read(|cache| -> JlrsResult<Option<_>> {
            if let Some(cached) = cache.cache().get(path.as_bytes()) {
                if cached.0 == tid {
                    return Ok(Some(cached.1.cast_unchecked()));
                } else {
                    let ty = T::construct_type(target).as_value();
                    Err(TypeError::NotA {
                        value: cached.1.display_string_or("<Cannot display value>"),
                        field_type: ty.display_string_or("<Cannot display type>"),
                    })?
                }
            }
            Ok(None)
        })?;

        if let Some(res) = res {
            return Ok(res);
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

        cache.write(|cache| {
            cache.roots_mut().insert(item.as_value());
            cache.cache_mut().insert(
                path.as_bytes().into(),
                (tid, erase_scope_lifetime(item.as_value())),
            );
        });

        Ok(item)
    }

    /// Returns a handle to Julia's `Main`-module. If you include your own Julia code with
    /// [`Runtime::include`] its contents are made available relative to
    /// `Main`.
    ///
    /// [`Runtime::include`]: crate::runtime::Runtime::include
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
        // Safety: We don't hit a safepoint before the data has been rooted.
        match self.global(&target, name) {
            Ok(v) => unsafe { Ok(v.as_value().cast::<Module>()?.root(target)) },
            Err(e) => Err(e)?,
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

    /// Set a global value in this module. Creating new globals at runtime is not supported for
    /// Julia 1.12+
    ///
    /// If an excection is thrown, it's caught and returned.
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
            jl_set_global(
                self.unwrap(Private),
                symbol.unwrap(Private),
                value.unwrap(Private),
            )
        };

        let res = catch_exceptions(callback, unwrap_exc);
        target.exception_from_ptr(res, Private)
    }

    /// Set a global value in this module. Creating new globals at runtime is not supported for
    /// Julia 1.12+.
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

    /// Set a constant in this module.
    ///
    /// If Julia throws an exception it's caught and returned.
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

    /// Set a constant in this module.
    ///
    /// If the constant already exists the process aborts, otherwise the constant is returned.
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
    ///
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
        unsafe {
            let name = name.to_symbol(&target);

            let func = || {
                let name = name.to_symbol(&target);
                match self.global_unchecked(target, name) {
                    Some(x) => Ok(x),
                    None => Err(AccessError::GlobalNotFound {
                        name: name.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                        module: self.name().as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    })?,
                }
            };

            let res = catch_exceptions(func, |_| {
                AccessError::GlobalNotFound {
                    name: name.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    module: self.name().as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                }
                .into()
            });

            match res {
                Ok(Ok(x)) => Ok(x),
                Ok(Err(e)) => Err(e),
                Err(e) => Err(e),
            }
        }
    }

    /// Returns the global named `name` in this module.
    ///
    /// Safety: If the global doesn't exist, an exception is thrown
    pub unsafe fn global_unchecked<'target, N, Tgt>(
        self,
        target: Tgt,
        name: N,
    ) -> Option<ValueData<'target, 'static, Tgt>>
    where
        N: ToSymbol,
        Tgt: Target<'target>,
    {
        unsafe {
            let symbol = name.to_symbol(&target);
            let value = jl_get_global(self.unwrap(Private), symbol.unwrap(Private));
            let ptr = NonNull::new(value)?;
            Some(Value::wrap_non_null(ptr, Private).root(target))
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
    /// successfully.
    ///
    /// This method can be used to load parts of the standard library like
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

/// A [`Module`] that has not been explicitly rooted.
pub type WeakModule<'scope> = Weak<'scope, 'static, Module<'scope>>;

/// A [`WeakModule`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`Module`].
pub type ModuleRet = WeakModule<'static>;

impl_valid_layout!(WeakModule, Module, jl_module_type);

use crate::memory::target::TargetType;

/// `Module` or `WeakModule`, depending on the target type `Tgt`.
pub type ModuleData<'target, Tgt> = <Tgt as TargetType<'target>>::Data<'static, Module<'target>>;

/// `JuliaResult<Module>` or `WeakJuliaResult<WeakModule>`, depending on the target type `Tgt`.
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
    pub fn set_error_color<'target, Tgt>(target: &Tgt) -> Function<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        inline_static_ref!(
            SET_ERROR_COLOR,
            Function,
            "JlrsCore.set_error_color",
            target
        )
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
