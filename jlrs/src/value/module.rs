//! Access Julia modules and the globals and functions defined in them.

use crate::error::{JlrsError, JlrsResult};
use crate::value::Value;
use crate::{
    convert::{cast::Cast, temporary_symbol::TemporarySymbol},
    memory::{
        global::Global,
        traits::{frame::Frame, scope::Scope},
    },
    value::symbol::Symbol,
};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{
    jl_base_module, jl_core_module, jl_get_global, jl_main_module, jl_module_t, jl_module_type,
    jl_set_const, jl_set_global, jl_typeis,
};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;

use super::{
    traits::{call::Call, private::Internal},
    LeakedValue,
};

/// Functionality in Julia can be accessed through its module system. You can get a handle to the
/// three standard modules, `Main`, `Base`, and `Core` and access their submodules through them.
/// If you include your own Julia code with [`Julia::include`], its contents are made available
/// relative to `Main`.
///
/// This struct implements [`JuliaTypecheck`] and [`Cast`]. It can be used in combination with
/// [`DataType::is`] and [`Value::is`]; if the check returns `true` the [`Value`] can be cast to
///  `Module`.
///
/// [`Julia::include`]: ../../struct.Julia.html#method.include
/// [`JuliaTypecheck`]: ../../layout/julia_typecheck/trait.JuliaTypecheck.html
/// [`DataType::is`]: ../datatype/struct.DataType.html#method.is
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Module<'base>(*mut jl_module_t, PhantomData<&'base ()>);

impl<'base> Module<'base> {
    pub(crate) unsafe fn wrap(module: *mut jl_module_t) -> Self {
        Module(module, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_module_t {
        self.0
    }

    /// Returns the name of this module.
    pub fn name(self) -> Symbol<'base> {
        unsafe { Symbol::wrap((&*(self.ptr())).name) }
    }

    /// Returns the parent of this module.
    pub fn parent(self) -> Option<Self> {
        unsafe {
            let parent = (&*(self.ptr())).parent;
            if parent.is_null() {
                return None;
            }

            Some(Self::wrap(parent))
        }
    }

    /// Extend the lifetime of this module; if `self` has originally been created by calling some
    /// Julia function the lifetime will be limited to the frame the function is called with. This
    /// can be extended to the lifetime of `Global` by calling this method.
    pub fn extend<'global>(self, _: Global<'global>) -> Module<'global> {
        unsafe { Module::wrap(self.ptr()) }
    }

    /// Returns a handle to Julia's `Main`-module. If you include your own Julia code by calling
    /// [`Julia::include`], handles to functions, globals, and submodules defined in these
    /// included files are available through this module.
    ///
    /// [`Julia::include`]: ../../struct.Julia.html#method.include
    pub fn main(_: Global<'base>) -> Self {
        unsafe { Module::wrap(jl_main_module) }
    }

    /// Returns a handle to Julia's `Core`-module.
    pub fn core(_: Global<'base>) -> Self {
        unsafe { Module::wrap(jl_core_module) }
    }

    /// Returns a handle to Julia's `Base`-module.
    pub fn base(_: Global<'base>) -> Self {
        unsafe { Module::wrap(jl_base_module) }
    }

    /// Returns the submodule named `name` relative to this module. You have to visit this level
    /// by level: you can't access `Main.A.B` by calling this function with `"A.B"`, but have to
    /// access `A` first and then `B`.
    ///
    /// Returns an error if the submodule doesn't exist.
    pub fn submodule<N>(self, name: N) -> JlrsResult<Self>
    where
        N: TemporarySymbol,
    {
        unsafe {
            // safe because jl_symbol_n copies the contents
            let symbol = name.temporary_symbol(Internal);

            let submodule = jl_get_global(self.ptr(), symbol.ptr());

            if !submodule.is_null() && jl_typeis(submodule, jl_module_type) {
                Ok(Module(submodule as *mut jl_module_t, PhantomData))
            } else {
                Err(JlrsError::NotAModule(symbol.into()).into())
            }
        }
    }

    /// Set a global value in this module. This is unsafe because if another global value was
    /// previously assigned to this name, this previous value can become eligible for garbage
    /// collection. Don't use the previous value after calling this method.
    pub unsafe fn set_global<'frame, N>(
        self,
        name: N,
        value: Value<'frame, 'static>,
    ) -> Value<'base, 'static>
    where
        N: TemporarySymbol,
    {
        jl_set_global(
            self.ptr(),
            name.temporary_symbol(Internal).ptr(),
            value.ptr(),
        );
        Value::wrap(value.ptr())
    }

    /// Set a constant in this module.
    pub fn set_const<'frame, N>(
        self,
        name: N,
        value: Value<'frame, 'static>,
    ) -> JlrsResult<Value<'base, 'static>>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = name.temporary_symbol(Internal);
            if self.global(symbol).is_ok() {
                Err(JlrsError::ConstAlreadyExists(symbol.into()))?;
            }

            jl_set_const(self.ptr(), symbol.ptr(), value.ptr());

            Ok(Value::wrap(value.ptr()))
        }
    }

    /// Returns the global named `name` in this module.
    /// Returns an error if the global doesn't exist.
    pub fn global<N>(self, name: N) -> JlrsResult<Value<'base, 'static>>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = name.temporary_symbol(Internal);

            // there doesn't seem to be a way to check if this is actually a
            // function...
            let func = jl_get_global(self.ptr(), symbol.ptr());
            if func.is_null() {
                return Err(JlrsError::FunctionNotFound(symbol.into()).into());
            }

            Ok(Value::wrap(func.cast()))
        }
    }

    /// Returns the global named `name` in this module as a [`LeakedValue`].
    /// Returns an error if the global doesn't exist.
    pub fn leaked_global<N>(self, name: N) -> JlrsResult<LeakedValue>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let symbol = name.temporary_symbol(Internal);

            // there doesn't seem to be a way to check if this is actually a
            // function...
            let func = jl_get_global(self.ptr(), symbol.ptr());
            if func.is_null() {
                return Err(JlrsError::FunctionNotFound(symbol.into()).into());
            }

            Ok(LeakedValue::wrap(func))
        }
    }

    /// Returns the function named `name` in this module. Note that all globals defined within the
    /// module will be successfully resolved into a function; Julia will throw an exception if you
    /// try to call something that isn't a function. This means that this method is just an alias
    /// for `Module::global`.
    ///
    /// Returns an error if the function doesn't exist.
    pub fn function<N>(self, name: N) -> JlrsResult<Value<'base, 'static>>
    where
        N: TemporarySymbol,
    {
        self.global(name)
    }

    /// Returns the function named `name` in this module as a [`LeakedValue`].
    /// Returns an error if the function doesn't exist.
    pub fn leaked_function<N>(self, name: N) -> JlrsResult<LeakedValue>
    where
        N: TemporarySymbol,
    {
        self.leaked_global(name)
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'base, 'static> {
        self.into()
    }

    /// Convert `self` to a `LeakedValue`.
    pub fn as_leaked(self) -> LeakedValue {
        unsafe { LeakedValue::wrap(self.ptr().cast()) }
    }

    /// Load a module by calling `Base.require` and return this module if it has been loaded
    /// successfully. This method can be used to load parts of the standard library like
    /// `LinearAlgebra`. This requires one slot on the GC stack. Note that the loaded module is
    /// not made available in the module used to call this method, you can use
    /// `Module::set_global` to do so.
    pub fn require<'scope, 'frame, S, F, M>(self, scope: S, module: M) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'static, F>,
        F: Frame<'frame>,
        M: TemporarySymbol,
    {
        unsafe {
            Module::wrap(jl_base_module)
                .function("require")
                .unwrap()
                .call2(
                    scope,
                    self.as_value(),
                    module.temporary_symbol(Internal).as_value(),
                )
        }
    }

    /// Load a module by calling `Base.require` and return this module if it has been loaded
    /// successfully. This method can be used to load parts of the standard library like
    /// `LinearAlgebra`. Unlike `Module::require`, this method will panic if the module cannot
    /// be loaded. Note that the loaded module is not made available in the module used to call
    /// this method, you can use `Module::set_global` to do so.
    pub fn require_or_panic<S>(self, global: Global<'base>, module: S) -> JlrsResult<Self>
    where
        S: TemporarySymbol,
    {
        unsafe {
            let out = Module::base(global)
                .function("require")
                .unwrap()
                .call2_unprotected(
                    global,
                    self.as_value(),
                    module.temporary_symbol(Internal).as_value(),
                )
                .expect(&format!(
                    "Could not load ${:?}",
                    module.temporary_symbol(Internal)
                ))
                .cast_unchecked::<Module>();

            Ok(out)
        }
    }
}

impl<'base> Into<Value<'base, 'static>> for Module<'base> {
    fn into(self) -> Value<'base, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Module<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAModule("This".to_string()))?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(Module<'frame>, jl_module_type, 'frame);
impl_julia_type!(Module<'frame>, jl_module_type, 'frame);
impl_valid_layout!(Module<'frame>, 'frame);

impl<'frame, 'data> Debug for Module<'frame> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let name: String = self.name().into();
        f.debug_tuple("Module").field(&name).finish()
    }
}
