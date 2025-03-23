//! Embed Julia in a Rust application.
//!
//! There are several ways Julia can be embedded in a Rust application using jlrs, see the
//! [crate-level docs] for more information. To start the Julia runtime runtime, you must
//! use a [`Builder`]. See the [`builder`] module for more information.
//!
//! [`Builder`]: crate::runtime::builder::Builder

use std::path::Path;

use handle::IsActive;
use private::RuntimePriv;

#[cfg(feature = "async-rt")]
use self::handle::async_handle::{dispatch::Dispatch, message::Message, AsyncHandle};
use crate::{
    call::Call,
    data::managed::module::JlrsCore,
    error::{IOError, JlrsResult},
    prelude::{IntoJlrsResult, JuliaString, LocalScope, Managed, Module, Target, Value},
    weak_handle_unchecked,
};

#[cfg(any(feature = "local-rt", feature = "async-rt", feature = "multi-rt"))]
pub mod builder;
#[cfg(feature = "async")]
pub mod executor;
pub mod handle;
pub mod state;

/// Provides access to the `Runtime` trait to targets.
pub struct RuntimeSettings<T>(T);

impl<'target, 'borrow, T> RuntimeSettings<&'borrow T>
where
    T: Target<'target>,
{
    pub(crate) fn new(target: &'borrow T) -> Self {
        RuntimeSettings(target)
    }
}

/// Set global options, load custom code.
///
/// There are three kinds of types that can call these methods:
///
/// - Active handles, i.e. implementors of [`IsActive`].
/// - Targets, i.e. implementors of [`Target`], when they are wrapped in [`RuntimeSettings`].
/// - Handles to an async runtime, i.e. [`AsyncHandle`].
///
/// The first two can execute these operations on the current thread, `AsyncHandle` returns
/// dispatchers to send them to a thread that can call Julia code.
pub unsafe trait Runtime: RuntimePriv {
    /// Type returned by `error_color` and `using`.
    ///
    /// For every implementation except [`AsyncHandle`], `Out<T> = T`. `AsyncHandle` wraps `T` in
    /// [`Dispatch`].
    type Out<'a, T>
    where
        T: 'static,
        Self: 'a;

    /// Type returned by `include`.
    ///
    /// For every implementation except [`AsyncHandle`], `FallibleOut<T> = T`. `AsyncHandle` wraps
    /// `T` in a [`JlrsResult`] and a [`Dispatch`]. This is done to avoid using a nested
    /// `JlrsResult` in the general case.
    type FallibleOut<'a, T>
    where
        T: 'static,
        Self: 'a;

    /// Enable or disable colors when printing error messages from Julia in Rust.
    fn error_color<'a>(&'a self, enable: bool) -> Self::Out<'a, ()>;

    /// Include a file.
    ///
    /// This calls `include` in the `Main` module in Julia, which executes the file's contents in
    /// that module. This has the same effect as calling `include` in the Julia REPL.
    ///
    /// Safety: the content of the file must be safe to evaluate.
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
    unsafe fn include<'a, P: AsRef<Path>>(
        &'a self,
        path: P,
    ) -> JlrsResult<Self::FallibleOut<'a, ()>>;

    /// Use a module.
    ///
    /// This evaluates `using {module_name}`. It can be used to load any installed package.
    ///
    /// Safety: `module_name` must be a valid module or package name.
    unsafe fn using<'a, S: AsRef<str>>(&'a self, module_name: S) -> Self::Out<'a, JlrsResult<()>>;
}

unsafe impl<T: RuntimePriv + IsActive> Runtime for T {
    type Out<'a, O>
        = O
    where
        O: 'static,
        Self: 'a;

    type FallibleOut<'a, O>
        = O
    where
        O: 'static,
        Self: 'a;

    fn error_color(&self, enable: bool) {
        // Safety: called with a target or an active handle
        unsafe { set_error_color(enable) }
    }

    unsafe fn include<'a, P: AsRef<Path>>(&'a self, path: P) -> JlrsResult<()> {
        // Safety: called with a target or an active handle
        unsafe { include(path) }
    }

    unsafe fn using<'a, S: AsRef<str>>(&'a self, module_name: S) -> JlrsResult<()> {
        // Safety: called with a target or an active handle
        unsafe { using(module_name) }
    }
}

unsafe impl<'tgt, T: Target<'tgt>> Runtime for RuntimeSettings<T> {
    type Out<'a, O>
        = O
    where
        O: 'static,
        Self: 'a;

    type FallibleOut<'a, O>
        = O
    where
        O: 'static,
        Self: 'a;

    fn error_color(&self, enable: bool) {
        // Safety: called with a target or an active handle
        unsafe { set_error_color(enable) }
    }

    unsafe fn include<'a, P: AsRef<Path>>(&'a self, path: P) -> JlrsResult<()> {
        // Safety: called with a target or an active handle
        unsafe { include(path) }
    }

    unsafe fn using<'a, S: AsRef<str>>(&'a self, module_name: S) -> JlrsResult<()> {
        // Safety: called with a target or an active handle
        unsafe { using(module_name) }
    }
}

#[cfg(feature = "async-rt")]
unsafe impl Runtime for AsyncHandle {
    type Out<'a, T>
        = Dispatch<'a, Message, T>
    where
        T: 'static,
        Self: 'a;

    type FallibleOut<'a, T>
        = Dispatch<'a, Message, JlrsResult<T>>
    where
        T: 'static,
        Self: 'a;

    fn error_color<'a>(&'a self, enable: bool) -> Dispatch<'a, Message, ()> {
        self.error_color(enable)
    }

    unsafe fn include<'a, P: AsRef<Path>>(
        &'a self,
        path: P,
    ) -> JlrsResult<Dispatch<'a, Message, JlrsResult<()>>> {
        self.include(path)
    }

    unsafe fn using<'a, S: AsRef<str>>(
        &'a self,
        module_name: S,
    ) -> Dispatch<'a, Message, JlrsResult<()>> {
        self.using(module_name.as_ref().into())
    }
}

// safety: must only be called from a thread that can call into Julia
unsafe fn set_error_color(enable: bool) {
    unsafe {
        let handle = weak_handle_unchecked!();
        let enable = if enable {
            Value::true_v(&handle)
        } else {
            Value::false_v(&handle)
        };

        JlrsCore::set_error_color(&handle)
            .call1(&handle, enable)
            .ok();
    };
}

// safety: must only be called from a thread that can call into Julia
unsafe fn include<P: AsRef<Path>>(path: P) -> JlrsResult<()> {
    if path.as_ref().exists() {
        unsafe {
            let handle = weak_handle_unchecked!();
            return handle.local_scope::<2>(|mut frame| {
                let path_jl_str = JuliaString::new(&mut frame, path.as_ref().to_string_lossy());
                Module::main(&frame)
                    .function(&frame, "include")?
                    .as_managed()
                    .call1(&mut frame, path_jl_str.as_value())
                    .into_jlrs_result()
                    .map(|_| ())
            });
        }
    }

    Err(IOError::NotFound {
        path: path.as_ref().to_string_lossy().into(),
    })?
}

// safety: must only be called from a thread that can call into Julia
unsafe fn using<S: AsRef<str>>(module_name: S) -> JlrsResult<()> {
    unsafe {
        weak_handle_unchecked!().local_scope::<1>(|mut frame| {
            let cmd = format!("import {}", module_name.as_ref());
            Value::eval_string(&mut frame, cmd)
                .map(|_| ())
                .into_jlrs_result()
        })
    }
}

mod private {
    #[cfg(feature = "async-rt")]
    use super::handle::async_handle::AsyncHandle;
    use super::{handle::IsActive, RuntimeSettings};
    use crate::prelude::Target;

    pub trait RuntimePriv {}

    impl<T: IsActive> RuntimePriv for T {}

    #[cfg(feature = "async-rt")]
    impl RuntimePriv for AsyncHandle {}

    impl<'tgt, T: Target<'tgt>> RuntimePriv for RuntimeSettings<T> {}
}
