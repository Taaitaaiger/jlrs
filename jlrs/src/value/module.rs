//! Access Julia modules and the globals and functions defined in them.

use crate::error::{JlrsError, JlrsResult};
use crate::global::Global;
use crate::traits::{private::Internal, TemporarySymbol, Cast};
use crate::value::Value;
use crate::{impl_julia_type, impl_julia_typecheck};
use jl_sys::{
    jl_base_module, jl_core_module, jl_get_global, jl_main_module, jl_module_t, jl_module_type,
    jl_set_const, jl_set_global, jl_typeis,
};
use std::marker::PhantomData;

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
/// [`JuliaTypecheck`]: ../../traits/trait.JuliaTypecheck.html
/// [`Cast`]: ../../traits/trait.Cast.html
/// [`DataType::is`]: ../datatype/struct.DataType.html#method.is
/// [`Value::is`]: ../struct.Value.html#method.is
/// [`Value`]: ../struct.Value.html
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

    /// Returns the function named `name` in this module. Note that all globals defined within the
    /// module will be successfully resolved into a function; Julia will throw an exception if you
    /// try to call something that isn't a function. This means that this method is just an alias
    /// for `Module::global`.
    ///
    /// Returns an error if th function doesn't exist.
    pub fn function<N>(self, name: N) -> JlrsResult<Value<'base, 'static>>
    where
        N: TemporarySymbol,
    {
        self.global(name)
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
