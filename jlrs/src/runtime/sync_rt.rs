//! Use Julia without support for multitasking.
//!
//! This module is only available if the `local-rt` feature is enabled, it provides the sync
//! runtime which initializes Julia on the current thread. It has been deprecated in favor
//! of [`LocalHandle`].
//!
//! [`LocalHandle`]: crate::runtime::handle::local_handle::LocalHandle

use std::{
    ffi::{c_void, CString},
    marker::PhantomData,
    path::Path,
};

use jl_sys::{jl_atexit_hook, jl_init, jl_init_with_image, jl_is_initialized};

use crate::{
    call::Call,
    convert::into_jlrs_result::IntoJlrsResult,
    data::managed::{module::Module, string::JuliaString, value::Value, Managed},
    error::{IOError, JlrsResult, RuntimeError},
    init_jlrs,
    memory::{
        context::stack::Stack,
        scope::{LocalScope, Scope},
        stack_frame::{PinnedFrame, StackFrame},
        target::{frame::GcFrame, unrooted::Unrooted},
    },
    runtime::{builder::Builder, state::can_init},
    INSTALL_METHOD,
};

/// A pending Julia instance.
///
/// This pending instance can be activated by calling [`PendingJulia::instance`].
pub struct PendingJulia {
    _not_send_sync: PhantomData<*mut c_void>,
}

impl PendingJulia {
    /// Activate the pending instance.
    ///
    /// The provided `StackFrame` should be allocated on the stack.
    pub fn instance<'ctx>(&'ctx mut self, frame: &'ctx mut StackFrame<0>) -> Julia<'ctx> {
        unsafe {
            // Is popped when Julia is dropped.
            let mut pinned = frame.pin();

            let install_method = INSTALL_METHOD.get().unwrap();
            init_jlrs(install_method);

            let frame = pinned.stack_frame();
            let context = frame.sync_stack();
            let wrapped: Julia<'ctx> = Julia {
                stack: context,
                _frame: pinned,
            };

            wrapped
        }
    }

    pub(crate) unsafe fn init(builder: Builder) -> JlrsResult<Self> {
        if !can_init() {
            Err(RuntimeError::AlreadyInitialized)?;
        }

        if let Some((julia_bindir, image_path)) = builder.image {
            let julia_bindir_str = julia_bindir.as_os_str().as_encoded_bytes();
            let image_path_str = image_path.as_os_str().as_encoded_bytes();

            let bindir = CString::new(julia_bindir_str).unwrap();
            let im_rel_path = CString::new(image_path_str).unwrap();

            jl_init_with_image(bindir.as_ptr(), im_rel_path.as_ptr());
        } else {
            jl_init();
        }

        assert!(jl_is_initialized() != 0);

        let install_method = builder.install_jlrs_core.clone();
        INSTALL_METHOD.get_or_init(|| install_method);

        Ok(PendingJulia {
            _not_send_sync: PhantomData,
        })
    }
}

impl Drop for PendingJulia {
    fn drop(&mut self) {
        unsafe {
            jl_atexit_hook(0);
        }
    }
}

/// An active Julia instance.
pub struct Julia<'context> {
    _frame: PinnedFrame<'context, 0>,
    stack: &'context Stack,
}

impl<'a> Julia<'a> {
    /// Enable or disable colored error messages originating from Julia. If this is enabled the
    /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
    /// disabled by default.
    ///
    /// [`JlrsError::Exception`]: crate::error::JlrsError::Exception
    pub fn error_color(&mut self, enable: bool) -> JlrsResult<()> {
        unsafe {
            let unrooted = Unrooted::new();
            let enable = if enable {
                Value::true_v(&unrooted)
            } else {
                Value::false_v(&unrooted)
            };

            // FIXME: make atomic
            Module::jlrs_core(&unrooted)
                .global(&unrooted, "color")?
                .as_value()
                .set_field_unchecked("x", enable)?;
        };

        Ok(())
    }

    /// Calls `include` in the `Main` module in Julia, which executes the file's contents in that
    /// module. This has the same effect as calling `include` in the Julia REPL.
    ///
    /// This is unsafe because the content of the file is evaluated.
    ///
    /// Example:
    ///
    /// ```no_run
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::test::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    /// # let mut frame = StackFrame::new();
    /// # let mut julia = julia.instance(&mut frame);
    /// unsafe {
    ///     julia.include("Path/To/MyJuliaCode.jl").unwrap();
    /// }
    /// # });
    /// # }
    /// ```
    pub unsafe fn include<P: AsRef<Path>>(&mut self, path: P) -> JlrsResult<()> {
        if path.as_ref().exists() {
            return self.scope(|mut frame| {
                let path_jl_str = JuliaString::new(&mut frame, path.as_ref().to_string_lossy());
                Module::main(&frame)
                    .function(&frame, "include")?
                    .as_managed()
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

    pub fn returning<T>(&mut self) -> &mut impl Scope<'a, T> {
        self
    }
}

impl<'ctx, T> Scope<'ctx, T> for Julia<'ctx> {
    fn scope<F>(&mut self, func: F) -> T
    where
        for<'scope> F: FnOnce(GcFrame<'scope>) -> T,
    {
        unsafe {
            let frame = GcFrame::base(&self.stack);

            let ret = func(frame);
            self.stack.pop_roots(0);
            ret
        }
    }
}

impl<'ctx, T> LocalScope<'ctx, T> for Julia<'ctx> {}
