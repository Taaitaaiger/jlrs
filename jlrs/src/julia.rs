//! The sync runtime.
//!
//! This module is only available if the `sync-rt` feature is enabled.

use crate::{
    error::{JlrsError, JlrsResult, CANNOT_DISPLAY_VALUE},
    info::Info,
    init_jlrs,
    memory::{frame::GcFrame, global::Global, mode::Sync, stack_page::StackPage},
    wrappers::ptr::{call::Call, module::Module, string::JuliaString, value::Value, Wrapper},
    INIT,
};
use jl_sys::{jl_atexit_hook, jl_init, jl_init_with_image, jl_is_initialized};
use std::{
    ffi::CString,
    io::{Error as IOError, ErrorKind},
    path::Path,
    sync::atomic::Ordering,
};

/// A Julia instance. You must create it with [`Julia::init`] or [`Julia::init_with_image`]
/// before you can do anything related to Julia. While this struct exists Julia is active,
/// dropping it causes the shutdown code to be called but this doesn't leave Julia in a state from
/// which it can be reinitialized.
pub struct Julia {
    page: StackPage,
}

impl Julia {
    /// Initialize Julia, this method can only be called once. If it's called a second time it
    /// will return an error. If this struct is dropped, you will need to restart your program to
    /// be able to call Julia code again.
    ///
    /// This method is unsafe because it can race with another crate initializing Julia.
    pub unsafe fn init() -> JlrsResult<Self> {
        if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
            return Err(JlrsError::AlreadyInitialized.into());
        }

        jl_init();
        let mut jl = Julia {
            page: StackPage::default(),
        };

        jl.scope(|_, frame| {
            init_jlrs(&mut *frame);

            #[cfg(feature = "pyplot")]
            crate::extensions::pyplot::init_jlrs_py_plot(&mut *frame);

            Ok(())
        })
        .expect("Could not load Jlrs module");

        Ok(jl)
    }

    /// This method is similar to [`Julia::init`] except that it loads a custom system image. A
    /// custom image can be generated with the [`PackageCompiler`] package for Julia. The main
    /// advantage of using a custom image over the default one is that it allows you to avoid much
    /// of the compilation overhead often associated with Julia.
    ///
    /// Two arguments are required to call this method compared to [`Julia::init`];
    /// `julia_bindir` and `image_relative_path`. The first must be the absolute path to a
    /// directory that contains a compatible Julia binary (eg `${JULIA_DIR}/bin`), the second must
    /// be either an absolute or a relative path to a system image.
    ///
    /// This method will return an error if either of the two paths doesn't  exist or if Julia
    /// has already been initialized. It is unsafe because it can race with another crate
    /// initializing Julia.
    ///
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl/dev/
    pub unsafe fn init_with_image<P: AsRef<Path>, Q: AsRef<Path>>(
        julia_bindir: P,
        image_path: Q,
    ) -> JlrsResult<Self> {
        if INIT.swap(true, Ordering::SeqCst) {
            Err(JlrsError::AlreadyInitialized)?;
        }

        let julia_bindir_str = julia_bindir.as_ref().to_string_lossy().to_string();
        let image_path_str = image_path.as_ref().to_string_lossy().to_string();

        if !julia_bindir.as_ref().exists() {
            let io_err = IOError::new(ErrorKind::NotFound, julia_bindir_str);
            return Err(JlrsError::other(io_err))?;
        }

        if !image_path.as_ref().exists() {
            let io_err = IOError::new(ErrorKind::NotFound, image_path_str);
            return Err(JlrsError::other(io_err))?;
        }

        let bindir = CString::new(julia_bindir_str).unwrap();
        let im_rel_path = CString::new(image_path_str).unwrap();

        jl_init_with_image(bindir.as_ptr(), im_rel_path.as_ptr());

        let mut jl = Julia {
            page: StackPage::default(),
        };

        jl.scope(|_, frame| {
            init_jlrs(&mut *frame);

            #[cfg(feature = "pyplot")]
            crate::extensions::pyplot::init_jlrs_py_plot(&mut *frame);

            Ok(())
        })?;

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
    /// Example:
    ///
    /// ```no_run
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = unsafe { Julia::init().unwrap() };
    /// julia.include("Path/To/MyJuliaCode.jl").unwrap();
    /// # }
    /// ```
    pub fn include<P: AsRef<Path>>(&mut self, path: P) -> JlrsResult<()> {
        if path.as_ref().exists() {
            return self.scope(|global, frame| unsafe {
                let path_jl_str = JuliaString::new(&mut *frame, path.as_ref().to_string_lossy())?;
                let include_func = Module::main(global)
                    .function_ref("include")?
                    .wrapper_unchecked();

                let res = include_func.call1(frame, path_jl_str)?;

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
            func(global, &mut frame)
        }
    }

    /// This method is a main entrypoint to interact with Julia. It takes a closure with two
    /// arguments, a `Global` and a mutable reference to a `GcFrame`, and can return arbitrary
    /// results. The frame will have capacity for at least `slots` roots.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope_with_slots(1, |_global, frame| {
    ///       let _i = Value::new(&mut *frame, 1u64)?;
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    pub fn scope_with_slots<T, F>(&mut self, slots: usize, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, &mut GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let global = Global::new();
            if slots + 2 > self.page.size() {
                self.page = StackPage::new(slots + 2);
            }
            let mut frame = GcFrame::new(self.page.as_mut(), Sync);
            func(global, &mut frame)
        }
    }

    /// Provides access to global information.
    pub fn info(&self) -> Info {
        Info::new()
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
