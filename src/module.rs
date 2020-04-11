//! Access Julia modules and the globals and functions defined in them.

use crate::error::{JlrsError, JlrsResult};
use crate::traits::Frame;
use crate::value::Value;
use jl_sys::{
    jl_base_module, jl_core_module, jl_get_global, jl_main_module, jl_module_t, jl_module_type,
    jl_symbol_n, jl_typeis,
};
use std::marker::PhantomData;

/// Functionality in Julia can be accessed through its module system. You can get a handle to the
/// three standard modules, `Main`, `Base`, and `Core` and access their submodules through them.
/// If you include your own Julia code with [`Julia::include`], its contents are made available
/// relative to `Main`.
///
/// [`Julia::include`]: ../struct.Julia.html#method.include
#[derive(Copy, Clone)]
pub struct Module<'scope>(*mut jl_module_t, PhantomData<&'scope ()>);

impl<'scope> Module<'scope> {
    /// Returns a handle to Julia's `Main`-module. If you include your own Julia code by calling
    /// [`Julia::include`], handles to functions, globals, and submodules defined in these
    /// included files are available through this module.
    ///
    /// [`Julia::include`]: ../struct.Julia.html#method.include
    pub fn main<'base: 'frame, 'frame, F: Frame<'base, 'frame>>(_: &mut F) -> Module<'base> {
        unsafe { Module(jl_main_module, PhantomData) }
    }

    /// Returns a handle to Julia's `Core`-module.
    pub fn core<'base: 'frame, 'frame, F: Frame<'base, 'frame>>(_: &mut F) -> Module<'base> {
        unsafe { Module(jl_core_module, PhantomData) }
    }

    /// Returns a handle to Julia's `Base`-module.
    pub fn base<'base: 'frame, 'frame, F: Frame<'base, 'frame>>(_: &mut F) -> Module<'base> {
        unsafe { Module(jl_base_module, PhantomData) }
    }

    /// Returns the submodule named `name` relative to this module. You have to visit this level
    /// by level: you can't access `Main.A.B` by calling this function with `"A.B"`, but have to
    /// access `A` first and then `B`.
    ///
    /// Returns an error if the submodule doesn't exist.
    pub fn submodule<N: AsRef<str>>(self, name: N) -> JlrsResult<Self> {
        unsafe {
            // safe because jl_symbol_n copies the contents
            let name_str = name.as_ref();
            let name_ptr = name_str.as_ptr();
            let symbol = jl_symbol_n(name_ptr as _, name_str.as_bytes().len() as _);
            let submodule = jl_get_global(self.0, symbol);

            if jl_typeis(submodule, jl_module_type) {
                Ok(Module(submodule as *mut jl_module_t, PhantomData))
            } else {
                Err(JlrsError::NotAModule(name.as_ref().into()).into())
            }
        }
    }

    /// Returns the global named `name` in this module.
    /// Returns an error if the global doesn't exist.
    pub fn global<N: AsRef<str>>(self, name: N) -> JlrsResult<Value<'scope, 'static>> {
        unsafe {
            // safe because jl_symbol_n copies the contents
            let name_str = name.as_ref();
            let name_ptr = name_str.as_ptr();
            let symbol = jl_symbol_n(name_ptr as _, name_str.as_bytes().len() as _);

            // there doesn't seem to be a way to check if this is actually a
            // function...
            let func = jl_get_global(self.0, symbol);
            if func.is_null() {
                return Err(JlrsError::FunctionNotFound(name_str.into()).into());
            }

            Ok(Value::wrap(func as _))
        }
    }

    /// Returns the function named `name` in this module. Note that all globals defined within the
    /// module will be successfully resolved into a function; Julia will throw an exception if you
    /// try to call something that isn't a function. This means that this method is just an alias
    /// for `Module::global`.
    /// 
    /// Returns an error if th function doesn't exist.
    pub fn function<N: AsRef<str>>(self, name: N) -> JlrsResult<Value<'scope, 'static>> {
        self.global(name)
    }
}
