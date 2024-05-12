//! A handle that lets you call into Julia from the current thread.

use std::{fmt, marker::PhantomData, path::Path};

use jl_sys::jl_atexit_hook;

use super::IsActive;
use crate::{
    call::Call,
    convert::into_jlrs_result::IntoJlrsResult,
    data::managed::module::Main,
    error::{IOError, JlrsResult},
    memory::scope::{LocalReturning, LocalScope},
    prelude::{JuliaString, Managed, Value},
    runtime::state::set_exit,
};

/// A handle that lets you call into Julia from the current thread.
///
/// An `LocalHandle` can be created by calling [`Builder::start_local`]. Julia exits when this
/// handle is dropped.
///
/// [`Builder::start_local`]: crate::runtime::builder::Builder::start_local
pub struct LocalHandle {
    _marker: PhantomData<*mut ()>,
}

impl LocalHandle {
    /// Calls `include` in the `Main` module in Julia, which executes the file's contents in that
    /// module. This has the same effect as calling `include` in the Julia REPL.
    ///
    /// This is unsafe because the content of the file is evaluated.
    ///
    /// Example:
    ///
    /// ```no_run
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// unsafe {
    ///     julia.include("Path/To/MyJuliaCode.jl").unwrap();
    /// }
    /// # }
    /// ```
    pub unsafe fn include<P: AsRef<Path>>(&self, path: P) -> JlrsResult<()> {
        if path.as_ref().exists() {
            return self.local_scope::<_, 2>(|mut frame| {
                let path_jl_str = JuliaString::new(&mut frame, path.as_ref().to_string_lossy());
                Main::include(&frame)
                    .call1(&mut frame, path_jl_str.as_value())
                    .into_jlrs_result()
                    .map(|_| ())
            });
        }

        Err(IOError::NotFound {
            path: path.as_ref().to_string_lossy().into(),
        })?
    }

    /// Evaluate `using {module_name}`.
    ///
    /// Safety: `module_name` must be a valid module or package name.
    pub unsafe fn using<S: AsRef<str>>(&self, module_name: S) -> JlrsResult<()> {
        return self.local_scope::<_, 1>(|mut frame| {
            let cmd = format!("using {}", module_name.as_ref());
            Value::eval_string(&mut frame, cmd)
                .map(|_| ())
                .into_jlrs_result()
        });
    }

    pub(crate) unsafe fn new() -> Self {
        LocalHandle {
            _marker: PhantomData,
        }
    }
}

impl Drop for LocalHandle {
    fn drop(&mut self) {
        unsafe {
            jl_atexit_hook(0);
            set_exit();
        }
    }
}

impl fmt::Debug for LocalHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalHandle").finish()
    }
}

impl IsActive for LocalHandle {}

impl<'ctx> LocalReturning<'ctx> for LocalHandle {
    fn returning<T>(&mut self) -> &mut impl LocalScope<'ctx, T> {
        self
    }
}

impl<'ctx, T> LocalScope<'ctx, T> for LocalHandle {}
