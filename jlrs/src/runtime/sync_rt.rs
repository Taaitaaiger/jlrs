//! Use Julia without support for multitasking.
//!
//! This module is only available if the `sync-rt` feature is enabled.

use crate::{
    call::Call,
    convert::into_jlrs_result::IntoJlrsResult,
    error::{IOError, JlrsResult, RuntimeError},
    memory::{
        context::{Stack},
        frame::GcFrame,
        global::{Global, BetterGlobal},
        context::ContextFrame, ledger::Ledger,
    },
    runtime::{builder::RuntimeBuilder, init_jlrs, INIT},
    wrappers::ptr::{module::Module, string::JuliaString, value::Value, Wrapper},
};
use jl_sys::{jl_atexit_hook, jl_init, jl_init_with_image, jl_is_initialized};
use std::{path::Path, ptr::{NonNull}, sync::atomic::Ordering, cell::RefCell};

/*
/// A Julia instance.
///
/// You must create this instance with [`RuntimeBuilder::start`] before you can start using
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
            Err(RuntimeError::AlreadyInitialized)?;
        }

        if let Some((julia_bindir, image_path)) = builder.image {
            let julia_bindir_str = julia_bindir.to_string_lossy().to_string();
            let image_path_str = image_path.to_string_lossy().to_string();

            if !julia_bindir.exists() {
                Err(IOError::NotFound {
                    path: julia_bindir_str,
                })?;
                unreachable!()
            }

            if !image_path.exists() {
                Err(IOError::NotFound {
                    path: image_path_str,
                })?;
                unreachable!()
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

        jl.scope(|_, mut frame| {
            init_jlrs(&mut frame);
            Ok(())
        })
        .ok();

        Ok(jl)
    }

    /// Enable or disable colored error messages originating from Julia. If this is enabled the
    /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
    /// disabled by default.
    ///
    /// [`JlrsError::Exception`]: crate::error::JlrsError::Exception
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
                .set_field_unchecked("x", enable)
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
            return self.scope(|global, mut frame| {
                let path_jl_str = JuliaString::new(&mut frame, path.as_ref().to_string_lossy())?;
                Module::main(global)
                    .function_ref("include")?
                    .wrapper_unchecked()
                    .call1(&mut frame, path_jl_str.as_value())?
                    .into_jlrs_result()
                    .map(|_| ())
            });
        }

        Err(IOError::NotFound {
            path: path.as_ref().to_string_lossy().into(),
        })?
    }

    /// This method is a main entrypoint to interact with Julia. It takes a closure with two
    /// arguments, a `Global` and a `GcFrame`, and can return arbitrary results.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::test::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope(|_global, mut frame| {
    ///       let _i = Value::new(&mut frame, 1u64)?;
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    pub fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let global = Global::new();
            let (frame, owner) = GcFrame::new(self.page.as_ref(), Sync);

            let ret = func(global, frame);
            std::mem::drop(owner);
            ret
        }
    }

    /// This method is a main entrypoint to interact with Julia. It takes a closure with two
    /// arguments, a `Global` and a `GcFrame`, and can return arbitrary results. The frame will
    /// have capacity for at least `capacity` roots.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::test::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope(|_global, mut frame| {
    ///       let _i = Value::new(&mut frame, 1u64)?;
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    pub fn scope_with_capacity<T, F>(&mut self, capacity: usize, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let global = Global::new();
            if capacity + 2 > self.page.size() {
                self.page = StackPage::new(capacity + 2);
            }
            let (frame, owner) = GcFrame::new(self.page.as_ref(), Sync);

            let ret = func(global, frame);
            std::mem::drop(owner);
            ret
        }
    }

    #[cfg(test)]
    pub(crate) fn get_page(&self) -> &StackPage {
        &self.page
    }
}

impl Drop for Julia {
    fn drop(&mut self) {
        unsafe {
            jl_atexit_hook(0);
        }
    }
}
 */

pub struct Julia<'context> {
    _base_frame: &'context ContextFrame,
    ledger: RefCell<Ledger>,
    context: &'context Stack,
}

impl<'context> Julia<'context> {
    pub(crate) unsafe fn init(
        builder: RuntimeBuilder,
        base_frame: &'context ContextFrame,
    ) -> JlrsResult<Self> {
        if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
            Err(RuntimeError::AlreadyInitialized)?;
        }

        if let Some((julia_bindir, image_path)) = builder.image {
            let julia_bindir_str = julia_bindir.to_string_lossy().to_string();
            let image_path_str = image_path.to_string_lossy().to_string();

            if !julia_bindir.exists() {
                Err(IOError::NotFound {
                    path: julia_bindir_str,
                })?;
                unreachable!()
            }

            if !image_path.exists() {
                Err(IOError::NotFound {
                    path: image_path_str,
                })?;
                unreachable!()
            }

            let bindir = std::ffi::CString::new(julia_bindir_str).unwrap();
            let im_rel_path = std::ffi::CString::new(image_path_str).unwrap();

            jl_init_with_image(bindir.as_ptr(), im_rel_path.as_ptr());
        } else {
            jl_init();
        }

        assert!(jl_is_initialized() != 0);

        cfg_if::cfg_if! {
            if #[cfg(feature = "lts")] {
                let rtls = NonNull::new_unchecked(jl_sys::jl_get_ptls_states()).as_mut();
                rtls.pgcstack = base_frame as *const _ as *mut _;
            } else {
                use jl_sys::{jl_get_current_task, jl_task_t};
                let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();
                task.gcstack = base_frame as *const _ as *mut _;
            }
        }

        let ctx_ty = Stack::init();
        let ctx = Stack::new(ctx_ty);
        let context = base_frame.set(ctx);

        let mut jl = Julia {
            _base_frame: base_frame,
            ledger: RefCell::new(Ledger::default()),
            context,
        };

        jl.scope(|_, mut frame| {
            init_jlrs(&mut frame);
            Ok(())
        })
        .ok();

        Ok(jl)
    }

    /// Enable or disable colored error messages originating from Julia. If this is enabled the
    /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
    /// disabled by default.
    ///
    /// [`JlrsError::Exception`]: crate::error::JlrsError::Exception
    pub fn error_color(&mut self, enable: bool) -> JlrsResult<()> {
        self.scope(|global, _frame| unsafe {
            let enable = if enable {
                Value::true_v(*global)
            } else {
                Value::false_v(*global)
            };

            Module::main(*global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .global_ref("color")?
                .value_unchecked()
                .set_field_unchecked("x", enable)
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
    /// # let context_frame = ContextFrame::new();
    /// # let mut julia = unsafe { RuntimeBuilder::new().start(&context_frame).unwrap() };
    /// unsafe { julia.include("Path/To/MyJuliaCode.jl").unwrap(); }
    /// # }
    /// ```
    pub unsafe fn include<P: AsRef<Path>>(&mut self, path: P) -> JlrsResult<()> {
        if path.as_ref().exists() {
            return self.scope(|global, mut frame| {
                let path_jl_str = JuliaString::new(&mut frame, path.as_ref().to_string_lossy())?;
                Module::main(*global)
                    .function_ref("include")?
                    .wrapper_unchecked()
                    .call1(&mut frame, path_jl_str.as_value())
                    .into_jlrs_result()
                    .map(|_| ())
            });
        }

        Err(IOError::NotFound {
            path: path.as_ref().to_string_lossy().into(),
        })?
    }

    /// This method is a main entrypoint to interact with Julia. It takes a closure with two
    /// arguments, a `Global` and a `GcFrame`, and can return arbitrary results.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::test::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope(|_global, mut frame| {
    ///       let _i = Value::new(&mut frame, 1u64);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    pub fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(BetterGlobal<'base>, GcFrame<'base>) -> JlrsResult<T>,
    {
        unsafe {
            let global = BetterGlobal::new(&self.ledger);
            let (frame, owner) = GcFrame::base(self.context, &self.ledger);

            let ret = func(global, frame);
            std::mem::drop(owner);
            ret
        }
    }
}

impl Drop for Julia<'_> {
    fn drop(&mut self) {
        unsafe {
            jl_atexit_hook(0);
        }
    }
}
