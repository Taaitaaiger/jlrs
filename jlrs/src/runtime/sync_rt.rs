//! The sync runtime.
//!
//! This module is only available if the `sync-rt` feature is enabled.

use crate::{
    call::Call,
    error::{JlrsError, JlrsResult, CANNOT_DISPLAY_VALUE},
    memory::{frame::GcFrame, global::Global, mode::Sync, stack_page::StackPage},
    runtime::{builder::RuntimeBuilder, init_jlrs, INIT},
    wrappers::ptr::{module::Module, string::JuliaString, value::Value, Wrapper},
};
use jl_sys::{jl_atexit_hook, jl_init, jl_init_with_image, jl_is_initialized};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
    sync::atomic::Ordering,
};

/// A Julia instance. You must create it with [`RuntimeBuilder::start`] before you can start using
/// Julia from Rust. While this struct exists Julia is active, dropping it causes the shutdown
/// code to be called but this doesn't leave Julia in a state from which it can be reinitialized.
///
/// [`RuntimeBuilder::start`]: crate::runtime::builder::RuntimeBuilder::start
pub struct Julia {
    page: StackPage,
}

impl Julia {
    pub(crate) unsafe fn init(builder: RuntimeBuilder) -> JlrsResult<Self> {
        if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
            return Err(JlrsError::AlreadyInitialized.into());
        }

        if let Some((ref julia_bindir, ref image_path)) = builder.image {
            let julia_bindir_str = julia_bindir.to_string_lossy().to_string();
            let image_path_str = image_path.to_string_lossy().to_string();

            if !julia_bindir.exists() {
                let io_err = IOError::new(ErrorKind::NotFound, julia_bindir_str);
                return Err(JlrsError::other(io_err))?;
            }

            if !image_path.exists() {
                let io_err = IOError::new(ErrorKind::NotFound, image_path_str);
                return Err(JlrsError::other(io_err))?;
            }

            let bindir = std::ffi::CString::new(julia_bindir_str).unwrap();
            let im_rel_path = std::ffi::CString::new(image_path_str).unwrap();

            jl_init_with_image(bindir.as_ptr(), im_rel_path.as_ptr());
        } else {
            jl_init();
        }

        assert!(jl_is_initialized() != 0);

        let mut jl = Julia {
            page: StackPage::default(),
        };

        jl.scope(|_, frame| {
            init_jlrs(&mut *frame);
            Ok(())
        })
        .expect("Could not load Jlrs module");

        Ok(jl)
    }

    /// Enable or disable colored error messages originating from Julia. If this is enabled the
    /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
    /// disabled by default.
    pub fn error_color(&mut self, enable: bool) -> JlrsResult<()> {
        self.scope(|global, _frame| unsafe {
            let enable = if enable {
                Value::true_v(global)
            } else {
                Value::false_v(global)
            };
            Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .global_ref("color")?
                .value_unchecked()
                .set_field_unchecked("x", enable)?;
            Ok(())
        })?;

        Ok(())
    }

    /// Calls `include` in the `Main` module in Julia, which executes the file's contents in that
    /// module. This has the same effect as calling `include` in the Julia REPL.
    ///
    /// This is unsafe because the contents of the file are evaluated.
    ///
    /// Example:
    ///
    /// ```no_run
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = unsafe { RuntimeBuilder::new().start().unwrap() };
    /// unsafe { julia.include("Path/To/MyJuliaCode.jl").unwrap(); }
    /// # }
    /// ```
    pub unsafe fn include<P: AsRef<Path>>(&mut self, path: P) -> JlrsResult<()> {
        if path.as_ref().exists() {
            return self.scope(|global, frame| {
                let path_jl_str = JuliaString::new(&mut *frame, path.as_ref().to_string_lossy())?;
                let include_func = Module::main(global)
                    .function_ref("include")?
                    .wrapper_unchecked();

                let res = include_func.call1(frame, path_jl_str.as_value())?;

                return match res {
                    Ok(_) => Ok(()),
                    Err(e) => Err(JlrsError::IncludeError {
                        path: path.as_ref().to_string_lossy().into(),
                        msg: e.display_string_or(CANNOT_DISPLAY_VALUE),
                    })?,
                };
            });
        }

        Err(JlrsError::IncludeNotFound {
            path: path.as_ref().to_string_lossy().into(),
        })?
    }

    /// This method is a main entrypoint to interact with Julia. It takes a closure with two
    /// arguments, a `Global` and a mutable reference to a `GcFrame`, and can return arbitrary
    /// results.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope(|_global, frame| {
    ///       let _i = Value::new(&mut *frame, 1u64)?;
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    pub fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, &mut GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let global = Global::new();
            let mut frame = GcFrame::new(self.page.as_mut(), Sync);

            let ret = func(global, &mut frame);
            std::mem::drop(frame);
            ret
        }
    }

    /// This method is a main entrypoint to interact with Julia. It takes a closure with two
    /// arguments, a `Global` and a mutable reference to a `GcFrame`, and can return arbitrary
    /// results. The frame will have capacity for at least `capacity` roots.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope_with_capacity(1, |_global, frame| {
    ///       let _i = Value::new(&mut *frame, 1u64)?;
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    pub fn scope_with_capacity<T, F>(&mut self, capacity: usize, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, &mut GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let global = Global::new();
            if capacity + 2 > self.page.size() {
                self.page = StackPage::new(capacity + 2);
            }
            let mut frame = GcFrame::new(self.page.as_mut(), Sync);

            let ret = func(global, &mut frame);
            std::mem::drop(frame);
            ret
        }
    }

    #[cfg(test)]
    pub(crate) fn get_page(&mut self) -> &mut StackPage {
        &mut self.page
    }
}

impl Drop for Julia {
    fn drop(&mut self) {
        unsafe {
            jl_atexit_hook(0);
        }
    }
}
