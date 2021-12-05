//! Wrapper for `Module`, which provides access to Julia's modules and their contents.
//!
//! In Julia, each module introduces a separate global scope. There are three important "root"
//! modules, `Main`, `Base` and `Core`. Any Julia code that you include in jlrs is made available
//! relative to the `Main` module, just like in Julia itself.

use crate::{
    convert::temporary_symbol::TemporarySymbol,
    error::{JlrsError, JlrsResult, CANNOT_DISPLAY_VALUE},
    impl_debug, impl_julia_typecheck, impl_valid_layout,
    memory::{frame::Frame, global::Global, scope::Scope},
    private::Private,
    wrappers::ptr::{
        call::Call,
        function::Function,
        symbol::Symbol,
        value::{LeakedValue, Value},
        FunctionRef, ModuleRef, ValueRef, Wrapper as _,
    },
};

#[cfg(not(all(target_os = "windows", feature = "lts")))]
use crate::error::{JuliaResult, JuliaResultRef};

use jl_sys::{
    jl_base_module, jl_core_module, jl_get_global, jl_is_imported, jl_main_module, jl_module_t,
    jl_module_type, jl_set_const, jl_set_global,
};

#[cfg(not(all(target_os = "windows", feature = "lts")))]
use jl_sys::{jlrs_result_tag_t_JLRS_RESULT_ERR, jlrs_set_const, jlrs_set_global};
use std::marker::PhantomData;
use std::ptr::NonNull;

use super::private::Wrapper;

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
/// [`Julia::include`]: crate::julia::Julia::include
/// [`AsyncJulia::include`]: crate::extensions::multitask::runtime::AsyncJulia::include
/// [`AsyncJulia::try_include`]: crate::extensions::multitask::runtime::AsyncJulia::try_include
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Module<'scope>(NonNull<jl_module_t>, PhantomData<&'scope ()>);

impl<'scope> Module<'scope> {
    /// Returns the name of this module.
    pub fn name(self) -> Symbol<'scope> {
        unsafe {
            let sym = NonNull::new_unchecked(self.unwrap_non_null(Private).as_ref().name);
            Symbol::wrap_non_null(sym, Private)
        }
    }

    /// Returns the parent of this module.
    pub fn parent<'target, F>(self, frame: &mut F) -> JlrsResult<Module<'target>>
    where
        F: Frame<'target>,
    {
        unsafe {
            let parent = self.unwrap_non_null(Private).as_ref().parent;
            debug_assert!(!parent.is_null());
            frame
                .push_root(NonNull::new_unchecked(parent.cast()), Private)
                .map(|v| v.cast_unchecked::<Module>())
                .map_err(Into::into)
        }
    }

    /// Returns the parent of this module without rooting it.
    pub fn parent_ref(self) -> ModuleRef<'scope> {
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
    /// [`Julia::include`]: crate::julia::Julia::include
    /// [`AsyncJulia::include`]: crate::extensions::multitask::runtime::AsyncJulia::include
    /// [`AsyncJulia::try_include`]: crate::extensions::multitask::runtime::AsyncJulia::try_include
    pub fn main(_: Global<'scope>) -> Self {
        unsafe { Module::wrap(jl_main_module, Private) }
    }

    /// Returns a handle to Julia's `Core`-module.
    pub fn core(_: Global<'scope>) -> Self {
        unsafe { Module::wrap(jl_core_module, Private) }
    }

    /// Returns a handle to Julia's `Base`-module.
    pub fn base(_: Global<'scope>) -> Self {
        unsafe { Module::wrap(jl_base_module, Private) }
    }

    /// Returns `true` if `self` has imported `sym`.
    pub fn is_imported<N: TemporarySymbol>(self, sym: N) -> bool {
        unsafe {
            let sym = sym.temporary_symbol(Private);
            jl_is_imported(self.unwrap(Private), sym.unwrap(Private)) != 0
        }
    }

    /// Returns the submodule named `name` relative to this module. You have to visit this level
    /// by level: you can't access `Main.A.B` by calling this function with `"A.B"`, but have to
    /// access `A` first and then `B`.
    ///
    /// Returns an error if the submodule doesn't exist.
    pub fn submodule<'target, N, F>(self, frame: &mut F, name: N) -> JlrsResult<Module<'target>>
    where
        N: TemporarySymbol,
        F: Frame<'target>,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);
            let submodule = jl_get_global(self.unwrap(Private), symbol.unwrap(Private));
            if submodule.is_null() {
                Err(JlrsError::GlobalNotFound {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    module: self.name().as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            let submodule = Value::wrap_non_null(NonNull::new_unchecked(submodule), Private);

            if submodule.is::<Self>() {
                frame
                    .push_root(submodule.unwrap_non_null(Private), Private)
                    .map(|v| v.cast_unchecked())
                    .map_err(Into::into)
            } else {
                Err(JlrsError::NotAModule {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
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
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);
            let submodule = jl_get_global(self.unwrap(Private), symbol.unwrap(Private));
            if submodule.is_null() {
                Err(JlrsError::GlobalNotFound {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    module: self.name().as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            let submodule = Value::wrap_non_null(NonNull::new_unchecked(submodule), Private);

            if let Ok(submodule) = submodule.cast::<Self>() {
                Ok(submodule.as_ref())
            } else {
                Err(JlrsError::NotAModule {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }
        }
    }

    /// Set a global value in this module. Note that if this global already exists, this can
    /// make the old value unreachable. If an excection is thrown, it's caught, rooted and
    /// returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn set_global<'frame, N, F>(
        self,
        frame: &mut F,
        name: N,
        value: Value<'_, 'static>,
    ) -> JlrsResult<JuliaResult<'frame, 'static, ValueRef<'scope, 'static>>>
    where
        N: TemporarySymbol,
        F: Frame<'frame>,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);

            let res = jlrs_set_global(
                self.unwrap(Private),
                symbol.unwrap(Private),
                value.unwrap(Private),
            );

            let data = res.data;

            if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                let v = frame
                    .push_root(NonNull::new_unchecked(data), Private)
                    .map_err(|e| JlrsError::AllocError(e))?;
                Ok(Err(v))
            } else {
                Ok(Ok(ValueRef::wrap(data)))
            }
        }
    }

    /// Set a global value in this module. Note that if this global already exists, this can
    /// make the old value unreachable. If an exception is thrown it's caught but not rooted and
    /// returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn set_global_unrooted<N>(
        self,
        name: N,
        value: Value<'_, 'static>,
    ) -> JuliaResultRef<'scope, 'static>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);

            let res = jlrs_set_global(
                self.unwrap(Private),
                symbol.unwrap(Private),
                value.unwrap(Private),
            );

            let data = res.data;

            if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                Err(ValueRef::wrap(data))
            } else {
                Ok(ValueRef::wrap(data))
            }
        }
    }

    /// Set a global value in this module. Note that if this global already exists, this can
    /// make the old value unreachable. If an exception is thrown the process aborts.
    pub fn set_global_unchecked<N>(
        self,
        name: N,
        value: Value<'_, 'static>,
    ) -> ValueRef<'scope, 'static>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);

            jl_set_global(
                self.unwrap(Private),
                symbol.unwrap(Private),
                value.unwrap(Private),
            );

            ValueRef::wrap(value.unwrap(Private))
        }
    }

    /// Set a constant in this module. If Julia throws an exception it's caught and rooted in the
    /// current frame, if the exception can't be rooted a `JlrsError::AllocError` is returned. If
    /// no exception is thrown an unrooted reference to the constant is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn set_const<'frame, N, F>(
        self,
        frame: &mut F,
        name: N,
        value: Value<'_, 'static>,
    ) -> JlrsResult<JuliaResult<'frame, 'static, ValueRef<'scope, 'static>>>
    where
        N: TemporarySymbol,
        F: Frame<'frame>,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);

            let res = jlrs_set_const(
                self.unwrap(Private),
                symbol.unwrap(Private),
                value.unwrap(Private),
            );

            let data = res.data;

            if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                let v = frame
                    .push_root(NonNull::new_unchecked(data), Private)
                    .map_err(|e| JlrsError::AllocError(e))?;
                Ok(Err(v))
            } else {
                Ok(Ok(ValueRef::wrap(data)))
            }
        }
    }

    /// Set a constant in this module. If Julia throws an exception it's caught. Otherwise an
    /// unrooted reference to the constant is returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn set_const_unrooted<N>(
        self,
        name: N,
        value: Value<'_, 'static>,
    ) -> JuliaResultRef<'scope, 'static>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);

            let res = jlrs_set_const(
                self.unwrap(Private),
                symbol.unwrap(Private),
                value.unwrap(Private),
            );

            let data = res.data;

            if res.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                Err(ValueRef::wrap(data))
            } else {
                Ok(ValueRef::wrap(data))
            }
        }
    }

    /// Set a constant in this module. If the constant already exists the process aborts,
    /// otherwise an unrooted reference to the constant is returned.
    pub fn set_const_unchecked<N>(
        self,
        name: N,
        value: Value<'_, 'static>,
    ) -> ValueRef<'scope, 'static>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);

            jl_set_const(
                self.unwrap(Private),
                symbol.unwrap(Private),
                value.unwrap(Private),
            );

            ValueRef::wrap(value.unwrap(Private))
        }
    }

    /// Returns the global named `name` in this module.
    /// Returns an error if the global doesn't exist.
    pub fn global<'target, N, F>(
        self,
        frame: &mut F,
        name: N,
    ) -> JlrsResult<Value<'target, 'static>>
    where
        N: TemporarySymbol,
        F: Frame<'target>,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);

            let func = jl_get_global(self.unwrap(Private), symbol.unwrap(Private));
            if func.is_null() {
                Err(JlrsError::GlobalNotFound {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    module: self.name().as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?;
            }

            frame
                .push_root(NonNull::new_unchecked(func), Private)
                .map_err(Into::into)
        }
    }

    /// Returns the global named `name` in this module without rooting it.
    /// Returns an error if the global doesn't exist.
    pub fn global_ref<N>(self, name: N) -> JlrsResult<ValueRef<'scope, 'static>>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);

            let global = jl_get_global(self.unwrap(Private), symbol.unwrap(Private));
            if global.is_null() {
                Err(JlrsError::GlobalNotFound {
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
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);

            // there doesn't seem to be a way to check if this is actually a
            // function...
            let func = jl_get_global(self.unwrap(Private), symbol.unwrap(Private));
            if func.is_null() {
                Err(JlrsError::GlobalNotFound {
                    name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                    module: self.name().as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?;
            }

            Ok(LeakedValue::wrap(func))
        }
    }

    /// Returns the function named `name` in this module.
    /// Returns an error if the function doesn't exist or if it's not a subtype of `Function`.
    pub fn function<'target, N, F>(
        self,
        frame: &mut F,
        name: N,
    ) -> JlrsResult<Function<'target, 'static>>
    where
        N: TemporarySymbol,
        F: Frame<'target>,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);
            let func = self.global(frame, symbol)?;

            if !func.is::<Function>() {
                let name = symbol.as_str().unwrap_or("<Non-UTF8 string>").into();
                let ty = func.datatype().display_string_or(CANNOT_DISPLAY_VALUE);
                Err(JlrsError::NotAFunction { name, ty })?;
            }

            Ok(func.cast_unchecked::<Function>())
        }
    }

    /// Returns the function named `name` in this module without rooting it.
    /// Returns an error if the function doesn't exist or if it's not a subtype of `Function`.
    pub fn function_ref<N>(self, name: N) -> JlrsResult<FunctionRef<'scope, 'static>>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = name.temporary_symbol(Private);
            let func = self.global_ref(symbol)?.wrapper_unchecked();

            if !func.is::<Function>() {
                let name = symbol.as_str().unwrap_or("<Non-UTF8 string>").into();
                let ty = func.datatype_name().unwrap_or("<Non-UTF8 string>").into();
                Err(JlrsError::NotAFunction { name, ty })?;
            }

            Ok(FunctionRef::wrap(func.unwrap(Private)))
        }
    }

    /// Convert `self` to a `LeakedValue`.
    pub fn as_leaked(self) -> LeakedValue {
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
    pub fn require<'target, 'current, S, F, N>(
        self,
        scope: S,
        module: N,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
        N: TemporarySymbol,
    {
        unsafe {
            Module::wrap(jl_base_module, Private)
                .function_ref("require")
                .unwrap()
                .wrapper_unchecked()
                .call2(
                    scope,
                    self.as_value(),
                    module.temporary_symbol(Private).as_value(),
                )
        }
    }

    /// Load a module by calling `Base.require` and return this module if it has been loaded
    /// successfully. This method can be used to load parts of the standard library like
    /// `LinearAlgebra`. Unlike `Module::require`, this method will panic if the module cannot
    /// be loaded. Note that the loaded module is not made available in the module used to call
    /// this method, you can use `Module::set_global` to do so.
    ///
    /// Note that when you want to call `using Submodule` in the `Main` module, you can do so by
    /// evaluating the using-statement with [`Value::eval_string`].
    pub fn require_ref<S>(self, global: Global<'scope>, module: S) -> ModuleRef<'scope>
    where
        S: TemporarySymbol,
    {
        unsafe {
            Module::base(global)
                .function_ref("require")
                .unwrap()
                .wrapper_unchecked()
                .call2_unrooted(
                    global,
                    self.as_value(),
                    module.temporary_symbol(Private).as_value(),
                )
                .expect(&format!(
                    "Could not load ${:?}",
                    module.temporary_symbol(Private)
                ))
                .wrapper_unchecked()
                .cast_unchecked::<Module>()
                .as_ref()
        }
    }
}

impl_julia_typecheck!(Module<'target>, jl_module_type, 'target);
impl_debug!(Module<'_>);
impl_valid_layout!(Module<'target>, 'target);

impl<'scope> Wrapper<'scope, '_> for Module<'scope> {
    type Wraps = jl_module_t;
    const NAME: &'static str = "Module";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}
