//! Acquire handles to Julia modules, globals and functions.

use crate::context::Scope;
use crate::error::{JlrsError, JlrsResult};
use crate::handles::GlobalHandle;
use jl_sys::{
    jl_base_module, jl_core_module, jl_get_global, jl_main_module, jl_module_t, jl_module_type,
    jl_symbol_n, jl_typeis,
};
use std::marker::PhantomData;

/// Functionality in Julia can be accessed through its module system. You can
/// get a handle to the three standard modules, `Main`, `Base`, and `Core`,
/// through either a `Session` or an `ExecutionContext`. In both cases these
/// handles will be valid until the session ends.
///
/// If you include your own Julia code with `Runtime::include`, its contents
/// are made available relative to `Main`. If your code is defined in its own
/// module, you have to acquire a handle to that module first by calling
/// `submodule` one or more times.
#[derive(Copy, Clone)]
pub struct Module<'scope>(*mut jl_module_t, PhantomData<&'scope Scope>);

impl<'scope> Module<'scope> {
    pub(crate) unsafe fn main() -> Self {
        Module(jl_main_module, PhantomData)
    }

    pub(crate) unsafe fn core() -> Self {
        Module(jl_core_module, PhantomData)
    }

    pub(crate) unsafe fn base() -> Self {
        Module(jl_base_module, PhantomData)
    }

    /// Get the submodule named `name` relative to this module. You have to visit
    /// this level by level, ie you can't access `Main.A.B` by calling this
    /// function with `"A.B"`, but have to access `A` first, and then `B`.
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

    /// Get the global named `name` in this module.
    pub fn global<N: AsRef<str>>(self, name: N) -> JlrsResult<GlobalHandle<'scope>> {
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

            return Ok(GlobalHandle::new(func, self.1));
        }
    }

    /// Get the function named `name` in this module. Note that all globals defined within the
    /// module will be successfully resolved into a function; Julia will throw an exception if you
    /// try to call something that isn't a function. This means that `Module::global` and
    /// `Module::function` do exactly the same thing; this function mostly exists for clarity.
    pub fn function<N: AsRef<str>>(self, name: N) -> JlrsResult<GlobalHandle<'scope>> {
        self.global(name)
    }
}
