//! Use Julia without support for multitasking.
//!
//! This module is only available if the `sync-rt` feature is enabled, it provides the sync
//! runtime which initializes Julia on the current thread.

use std::{ffi::c_void, marker::PhantomData, path::Path, sync::atomic::Ordering};

use jl_sys::{jl_atexit_hook, jl_init, jl_init_with_image, jl_is_initialized};

use crate::{
    call::Call,
    convert::into_jlrs_result::IntoJlrsResult,
    data::managed::{module::Module, string::JuliaString, value::Value, Managed},
    error::{IOError, JlrsResult, RuntimeError},
    init_jlrs,
    memory::{
        context::stack::Stack,
        stack_frame::{PinnedFrame, StackFrame},
        target::frame::GcFrame,
    },
    runtime::{builder::RuntimeBuilder, INIT},
};

/// A pending Julia instance.
///
/// This pending instance can be activated by calling [`PendingJulia::instance`].
pub struct PendingJulia {
    _not_send_sync: PhantomData<*mut c_void>,
}

impl PendingJulia {
    pub(crate) unsafe fn init(builder: RuntimeBuilder) -> JlrsResult<Self> {
        if jl_is_initialized() != 0 || INIT.swap(true, Ordering::Relaxed) {
            Err(RuntimeError::AlreadyInitialized)?;
        }

        if let Some((julia_bindir, image_path)) = builder.image {
            let julia_bindir_str = julia_bindir.to_string_lossy().to_string();
            let image_path_str = image_path.to_string_lossy().to_string();

            if !julia_bindir.exists() {
                return Err(IOError::NotFound {
                    path: julia_bindir_str,
                })?;
            }

            if !image_path.exists() {
                return Err(IOError::NotFound {
                    path: image_path_str,
                })?;
            }

            let bindir = std::ffi::CString::new(julia_bindir_str).unwrap();
            let im_rel_path = std::ffi::CString::new(image_path_str).unwrap();

            jl_init_with_image(bindir.as_ptr(), im_rel_path.as_ptr());
        } else {
            jl_init();
        }

        assert!(jl_is_initialized() != 0);

        Ok(PendingJulia {
            _not_send_sync: PhantomData,
        })
    }

    /// Activate the pending instance.
    ///
    /// The provided `StackFrame` should be allocated on the stack.
    pub fn instance<'ctx>(&'ctx mut self, frame: &'ctx mut StackFrame<0>) -> Julia<'ctx> {
        unsafe {
            // Is popped when Julia is dropped.
            let mut pinned = frame.pin();

            init_jlrs(&mut pinned);

            let frame = pinned.stack_frame();
            let context = frame.sync_stack();
            let wrapped: Julia<'ctx> = Julia {
                stack: context,
                _frame: pinned,
            };

            wrapped
        }
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

impl Julia<'_> {
    /// Enable or disable colored error messages originating from Julia. If this is enabled the
    /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
    /// disabled by default.
    ///
    /// [`JlrsError::Exception`]: crate::error::JlrsError::Exception
    pub fn error_color(&mut self, enable: bool) -> JlrsResult<()> {
        self.scope(|frame| unsafe {
            let enable = if enable {
                Value::true_v(&frame)
            } else {
                Value::false_v(&frame)
            };

            Module::main(&frame)
                .submodule(&frame, "Jlrs")?
                .as_managed()
                .global(&frame, "color")?
                .as_value()
                .set_field_unchecked("x", enable)
        })?;

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

    /// This method is a main entrypoint to interact with Julia. It takes a closure with one
    /// argument, a `GcFrame`, and can return arbitrary results.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::test::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    /// # let mut frame = StackFrame::new();
    /// # let mut julia = julia.instance(&mut frame);
    /// julia
    ///     .scope(|mut frame| {
    ///         let _i = Value::new(&mut frame, 1u64);
    ///         Ok(())
    ///     })
    ///     .unwrap();
    /// # });
    /// # }
    /// ```
    pub fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(GcFrame<'base>) -> JlrsResult<T>,
    {
        unsafe {
            let (owner, frame) = GcFrame::base(&self.stack);

            let ret = func(frame);
            std::mem::drop(owner);
            ret
        }
    }
}
