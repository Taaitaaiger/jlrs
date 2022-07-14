//! Wrapper for `Module`, which provides access to Julia's modules and their contents.
//!
//! In Julia, each module introduces a separate global scope. There are three important "root"
//! modules, `Main`, `Base` and `Core`. Any Julia code that you include in jlrs is made available
//! relative to the `Main` module.

use crate::{
    call::Call,
    convert::to_symbol::ToSymbol,
    error::{AccessError, JlrsResult, JuliaResult, TypeError, CANNOT_DISPLAY_VALUE},
    impl_debug, impl_julia_typecheck,
    memory::{global::Global, output::Output, scope::PartialScope},
    private::Private,
    wrappers::ptr::{
        function::Function,
        function::FunctionRef,
        private::WrapperPriv,
        symbol::Symbol,
        value::ValueRef,
        value::{LeakedValue, Value},
        Wrapper as _,
    },
};
use cfg_if::cfg_if;
use jl_sys::{
    jl_base_module, jl_core_module, jl_get_global, jl_is_imported, jl_main_module, jl_module_t,
    jl_module_type, jl_set_const, jl_set_global,
};
use std::{marker::PhantomData, ptr::NonNull};

use super::Ref;

cfg_if! {
    if #[cfg(not(all(target_os = "windows", all(feature = "lts", not(feature = "all-features-override")))))] {
        use crate::error::JuliaResultRef;
    }
}

/// Functionality in Julia can be accessed through its module system. You can get a handle to the
/// three standard modules, `Main`, `Base`, and `Core` and access their submodules through them.
/// If you include your own Julia code with [`Julia::include`], [`AsyncJulia::include`], or
/// [`AsyncJulia::try_include`] its contents are made available relative to `Main`.
///
/// The most important methods offered by this wrapper are those that let you access submodules,
/// functions, and other global values defined in the module. These come in two variants: one that
/// roots the result and one that doesn't. If you never redefine the module, it's safe to leave
/// named functions, constants and submodules unrooted when you use them from Rust. The same holds
/// true for other global values that are never redefined to point at another value.
///
/// [`Julia::include`]: crate::runtime::sync_rt::Julia::include
/// [`AsyncJulia::include`]: crate::runtime::async_rt::AsyncJulia::include
/// [`AsyncJulia::try_include`]: crate::runtime::async_rt::AsyncJulia::try_include
#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Module<'scope>(NonNull<jl_module_t>, PhantomData<&'scope ()>);

impl<'scope> Module<'scope> {
    /// Returns the name of this module.
    pub fn name(self) -> Symbol<'scope> {
        // Safety: the pointer points to valid data, the name is never null
        unsafe {
            let sym = NonNull::new_unchecked(self.unwrap_non_null(Private).as_ref().name);
            Symbol::wrap_non_null(sym, Private)
        }
    }

    /// Returns the parent of this module.
    pub fn parent<'target, S>(self, scope: S) -> JlrsResult<Module<'target>>
    where
        S: PartialScope<'target>,
    {
        // Safety: the pointer points to valid data, the parent is never null
        unsafe {
            let parent = self.unwrap_non_null(Private).as_ref().parent;
            scope.value(NonNull::new_unchecked(parent), Private)
        }
    }

    /// Returns the parent of this module without rooting it.
    pub fn parent_ref(self) -> ModuleRef<'scope> {
        // Safety: the pointer points to valid data, the parent is never null
        unsafe { ModuleRef::wrap(self.unwrap_non_null(Private).as_ref().parent) }
    }

    /// Extend the lifetime of this module. This is safe as long as the module is never redefined.
    pub unsafe fn extend<'global>(self, _: Global<'global>) -> Module<'global> {
        Module::wrap(self.unwrap(Private), Private)
    }

    /// Returns a handle to Julia's `Main`-module. If you include your own Julia code with
    /// [`Julia::include`], [`AsyncJulia::include`], or [`AsyncJulia::try_include`] its contents
    ///  are made available relative to `Main`.
    ///
    /// [`Julia::include`]: crate::runtime::sync_rt::Julia::include
    /// [`AsyncJulia::include`]: crate::runtime::async_rt::AsyncJulia::include
    /// [`AsyncJulia::try_include`]: crate::runtime::async_rt::AsyncJulia::try_include
    pub fn main(_: Global<'scope>) -> Self {
        // Safety: the Main module is globally rooted
        unsafe { Module::wrap(jl_main_module, Private) }
    }

    /// Returns a handle to Julia's `Core`-module.
    pub fn core(_: Global<'scope>) -> Self {
        // Safety: the Core module is globally rooted
        unsafe { Module::wrap(jl_core_module, Private) }
    }

    /// Returns a handle to Julia's `Base`-module.
    pub fn base(_: Global<'scope>) -> Self {
        // Safety: the Base module is globally rooted
        unsafe { Module::wrap(jl_base_module, Private) }
    }

    /// Returns `true` if `self` has imported `sym`.
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
    pub fn submodule<'target, N, S>(self, scope: S, name: N) -> JlrsResult<Module<'target>>
    where
        N: ToSymbol,
        S: PartialScope<'target>,
    {
        let symbol = name.to_symbol(scope.global());

        // Safety: the pointer points to valid data, the C API function is called with
        // valid arguments and its result is checked.
        unsafe {
            let submodule = jl_get_global(self.unwrap(Private), symbol.unwrap(Private));
            if submodule.is_null() {
                Err(AccessError::GlobalNotFound {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    module: self.name().as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            let submodule = Value::wrap_non_null(NonNull::new_unchecked(submodule), Private);

            if submodule.is::<Self>() {
                scope.value(submodule.unwrap_non_null(Private).cast(), Private)
            } else {
                Err(TypeError::NotAModule {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    type_str: submodule.datatype().name().into(),
                })?
            }
        }
    }

    /// Returns the submodule named `name` relative to this module without rooting it. You have to
    /// access this level by level: you can't access `Main.A.B` by calling this function with
    /// `"A.B"`, but have to access `A` first and then `B`.
    ///
    /// Returns an error if the submodule doesn't exist.
    pub fn submodule_ref<N>(self, name: N) -> JlrsResult<ModuleRef<'scope>>
    where
        N: ToSymbol,
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

            let submodule = Value::wrap_non_null(NonNull::new_unchecked(submodule), Private);

            if let Ok(submodule) = submodule.cast::<Self>() {
                Ok(submodule.as_ref())
            } else {
                Err(TypeError::NotAModule {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    type_str: submodule.datatype().name().into(),
                })?
            }
        }
    }

    /// Set a global value in this module. Note that if this global already exists, this can
    /// make the old value unreachable. If an excection is thrown, it's caught, rooted and
    /// returned.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    #[cfg(not(all(
        target_os = "windows",
        all(feature = "lts", not(feature = "all-features-override"))
    )))]
    pub unsafe fn set_global<'frame, N, S>(
        self,
        scope: S,
        name: N,
        value: Value<'_, 'static>,
    ) -> JlrsResult<JuliaResult<'frame, 'static, ()>>
    where
        N: ToSymbol,
        S: PartialScope<'frame>,
    {
        use crate::catch::catch_exceptions;
        use std::mem::MaybeUninit;
        let symbol = name.to_symbol_priv(Private);

        let mut callback = |_: &mut MaybeUninit<()>| {
            jl_set_global(
                self.unwrap(Private),
                symbol.unwrap(Private),
                value.unwrap(Private),
            );

            Ok(())
        };

        match catch_exceptions(&mut callback)? {
            Ok(_) => Ok(Ok(())),
            Err(e) => Ok(Err(e.root(scope)?)),
        }
    }

    /// Set a global value in this module. Note that if this global already exists, this can
    /// make the old value unreachable. If an exception is thrown it's caught but not rooted and
    /// returned.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
    #[cfg(not(all(
        target_os = "windows",
        all(feature = "lts", not(feature = "all-features-override"))
    )))]
    pub unsafe fn set_global_unrooted<N>(
        self,
        name: N,
        value: Value<'_, 'static>,
    ) -> JlrsResult<JuliaResultRef<'scope, 'static, ()>>
    where
        N: ToSymbol,
    {
        use crate::catch::catch_exceptions;
        use std::mem::MaybeUninit;
        let symbol = name.to_symbol_priv(Private);

        let mut callback = |_: &mut MaybeUninit<()>| {
            jl_set_global(
                self.unwrap(Private),
                symbol.unwrap(Private),
                value.unwrap(Private),
            );

            Ok(())
        };

        match catch_exceptions(&mut callback)? {
            Ok(_) => Ok(Ok(())),
            Err(e) => Ok(Err(e)),
        }
    }

    /// Set a global value in this module. Note that if this global already exists, this can
    /// make the old value unreachable.
    ///
    /// Safety: Mutating Julia data is generally unsafe because it can't be guaranteed mutating
    /// this value is allowed.
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
    #[cfg(not(all(
        target_os = "windows",
        all(feature = "lts", not(feature = "all-features-override"))
    )))]
    pub fn set_const<'frame, N, S>(
        self,
        scope: S,
        name: N,
        value: Value<'_, 'static>,
    ) -> JlrsResult<JuliaResult<'frame, 'static, ()>>
    where
        N: ToSymbol,
        S: PartialScope<'frame>,
    {
        // Safety: the pointer points to valid data, the C API function is called with
        // valid arguments and its result is checked. if an exception is thrown it's caught
        // and returned
        unsafe {
            use crate::catch::catch_exceptions;
            use std::mem::MaybeUninit;
            let symbol = name.to_symbol_priv(Private);

            let mut callback = |_: &mut MaybeUninit<()>| {
                jl_set_const(
                    self.unwrap(Private),
                    symbol.unwrap(Private),
                    value.unwrap(Private),
                );

                Ok(())
            };

            match catch_exceptions(&mut callback)? {
                Ok(_) => Ok(Ok(())),
                Err(e) => Ok(Err(e.root(scope)?)),
            }
        }
    }

    /// Set a constant in this module. If Julia throws an exception it's caught. Otherwise an
    /// unrooted reference to the constant is returned.
    #[cfg(not(all(
        target_os = "windows",
        all(feature = "lts", not(feature = "all-features-override"))
    )))]
    pub fn set_const_unrooted<N>(
        self,
        name: N,
        value: Value<'_, 'static>,
    ) -> JlrsResult<JuliaResultRef<'scope, 'static, ()>>
    where
        N: ToSymbol,
    {
        // Safety: the pointer points to valid data, the C API function is called with
        // valid arguments and its result is checked. if an exception is thrown it's caught
        // and returned
        unsafe {
            use crate::catch::catch_exceptions;
            use std::mem::MaybeUninit;
            let symbol = name.to_symbol_priv(Private);

            let mut callback = |_: &mut MaybeUninit<()>| {
                jl_set_const(
                    self.unwrap(Private),
                    symbol.unwrap(Private),
                    value.unwrap(Private),
                );

                Ok(())
            };

            match catch_exceptions(&mut callback)? {
                Ok(_) => Ok(Ok(())),
                Err(e) => Ok(Err(e)),
            }
        }
    }

    /// Set a constant in this module. If the constant already exists the process aborts,
    /// otherwise an unrooted reference to the constant is returned.
    ///
    /// Safety: This method must not throw an error if called from a `ccall`ed function.
    pub unsafe fn set_const_unchecked<N>(
        self,
        name: N,
        value: Value<'_, 'static>,
    ) -> ValueRef<'scope, 'static>
    where
        N: ToSymbol,
    {
        let symbol = name.to_symbol_priv(Private);

        jl_set_const(
            self.unwrap(Private),
            symbol.unwrap(Private),
            value.unwrap(Private),
        );

        ValueRef::wrap(value.unwrap(Private))
    }

    /// Returns the global named `name` in this module.
    /// Returns an error if the global doesn't exist.
    pub fn global<'target, N, S>(self, scope: S, name: N) -> JlrsResult<Value<'target, 'static>>
    where
        N: ToSymbol,
        S: PartialScope<'target>,
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

            scope.value(NonNull::new_unchecked(global), Private)
        }
    }

    /// Returns the global named `name` in this module without rooting it.
    /// Returns an error if the global doesn't exist.
    pub fn global_ref<N>(self, name: N) -> JlrsResult<ValueRef<'scope, 'static>>
    where
        N: ToSymbol,
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

            Ok(ValueRef::wrap(global))
        }
    }

    /// Returns the global named `name` in this module as a [`LeakedValue`].
    /// Returns an error if the global doesn't exist.
    pub fn leaked_global<N>(self, name: N) -> JlrsResult<LeakedValue>
    where
        N: ToSymbol,
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

            Ok(LeakedValue::wrap(global))
        }
    }

    /// Returns the function named `name` in this module.
    /// Returns an error if the function doesn't exist or if it's not a subtype of `Function`.
    pub fn function<'target, N, S>(
        self,
        scope: S,
        name: N,
    ) -> JlrsResult<Function<'target, 'static>>
    where
        N: ToSymbol,
        S: PartialScope<'target>,
    {
        // Safety: the pointer points to valid data, the result is checked.
        unsafe {
            let symbol = name.to_symbol_priv(Private);
            let func = self.global(scope, symbol)?;

            if !func.is::<Function>() {
                let name = symbol.as_str().unwrap_or("<Non-UTF8 string>").into();
                let ty = func.datatype().display_string_or(CANNOT_DISPLAY_VALUE);
                Err(TypeError::NotAFunction { name, type_str: ty })?;
            }

            Ok(func.cast_unchecked::<Function>())
        }
    }

    /// Returns the function named `name` in this module without rooting it.
    /// Returns an error if the function doesn't exist or if it's not a subtype of `Function`.
    pub fn function_ref<N>(self, name: N) -> JlrsResult<FunctionRef<'scope, 'static>>
    where
        N: ToSymbol,
    {
        // Safety: the pointer points to valid data, the result is checked.
        unsafe {
            let symbol = name.to_symbol_priv(Private);
            let func = self.global_ref(symbol)?.wrapper_unchecked();

            if !func.is::<Function>() {
                let name = symbol.as_str().unwrap_or("<Non-UTF8 string>").into();
                let ty = func.datatype_name().unwrap_or("<Non-UTF8 string>").into();
                Err(TypeError::NotAFunction { name, type_str: ty })?;
            }

            Ok(FunctionRef::wrap(func.unwrap(Private)))
        }
    }

    /// Convert `self` to a `LeakedValue`.
    pub fn as_leaked(self) -> LeakedValue {
        // Safety: the pointer points to valid data
        unsafe { LeakedValue::wrap_non_null(self.unwrap_non_null(Private).cast()) }
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
    pub unsafe fn require<'target, S, N>(
        self,
        scope: S,
        module: N,
    ) -> JlrsResult<JuliaResult<'target, 'static>>
    where
        S: PartialScope<'target>,
        N: ToSymbol,
    {
        Module::wrap(jl_base_module, Private)
            .function_ref("require")
            .unwrap()
            .wrapper_unchecked()
            .call2(
                scope,
                self.as_value(),
                module.to_symbol_priv(Private).as_value(),
            )
    }

    /// Load a module by calling `Base.require` and return this module if it has been loaded
    /// successfully. This method can be used to load parts of the standard library like
    /// `LinearAlgebra`. Unlike `Module::require`, this method will panic if the module cannot
    /// be loaded. Note that the loaded module is not made available in the module used to call
    /// this method, you can use `Module::set_global` to do so.
    ///
    /// Note that when you want to call `using Submodule` in the `Main` module, you can do so by
    /// evaluating the using-statement with [`Value::eval_string`].
    ///
    /// Safety: This method can execute arbitrary Julia code depending on the module that is
    /// loaded.
    pub unsafe fn require_unrooted<S>(self, global: Global<'scope>, module: S) -> ModuleRef<'scope>
    where
        S: ToSymbol,
    {
        Module::base(global)
            .function_ref("require")
            .unwrap()
            .wrapper_unchecked()
            .call2_unrooted(
                global,
                self.as_value(),
                module.to_symbol_priv(Private).as_value(),
            )
            .expect(&format!(
                "Could not load ${:?}",
                module.to_symbol_priv(Private)
            ))
            .wrapper_unchecked()
            .cast_unchecked::<Module>()
            .as_ref()
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> Module<'target> {
        // safety: the pointer points to valid data
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<Module>(ptr);
            Module::wrap_non_null(ptr, Private)
        }
    }
}

impl_julia_typecheck!(Module<'target>, jl_module_type, 'target);
impl_debug!(Module<'_>);

impl<'scope> WrapperPriv<'scope, '_> for Module<'scope> {
    type Wraps = jl_module_t;
    const NAME: &'static str = "Module";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(Module, 1);

/// A reference to a [`Module`] that has not been explicitly rooted.
pub type ModuleRef<'scope> = Ref<'scope, 'static, Module<'scope>>;
impl_valid_layout!(ModuleRef, Module);
impl_ref_root!(Module, ModuleRef, 1);
