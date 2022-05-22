//! Plot data with Plots.jl and PyPlot.jl
//!
//! In order to use this module, both PyPlot.jl and PyCall.jl must have been installed, as well as
//! GTK3. GTK3 is currently the only supported GUI. Note that when multiple figures are open, only
//! the most recently opened one is updated automatically.

use crate::{
    call::{Call, CallExt},
    convert::into_jlrs_result::IntoJlrsResult,
    error::JlrsResult,
    memory::{frame::Frame, global::Global, scope::PartialScope},
    wrappers::ptr::{
        function::Function,
        module::Module,
        value::{Value, MAX_SIZE},
        Wrapper,
    },
};

#[cfg(feature = "async-rt")]
use crate::{call::CallAsync, memory::frame::AsyncGcFrame};

use smallvec::SmallVec;

init_fn!(init_jlrs_py_plot, JLRS_PY_PLOT_JL, "JlrsPyPlot.jl");

/// A handle to a plotting window.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PyPlot<'scope>(Value<'scope, 'static>);

impl<'scope> PyPlot<'scope> {
    /// Create a new plotting window by calling `plotfn(args...)`. The window stays open until it
    /// has been closed, even if all handles have been dropped. `plot_fn` must be a plotting
    /// function from the Plots.jl package, such as `plot` or `hexbin`. The resources associated
    /// with the window are only cleaned up if one of the `PyPlot::wait` methods is called.
    pub unsafe fn new<'value, V, F>(
        frame: &mut F,
        plot_fn: Function<'_, 'static>,
        args: V,
    ) -> JlrsResult<Self>
    where
        V: AsRef<[Value<'value, 'static>]>,
        F: Frame<'scope>,
    {
        let global = frame.global();
        let args = args.as_ref();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + args.len());
        vals.push(plot_fn.as_value());

        for arg in args.iter().copied() {
            vals.push(arg);
        }

        let plt = Module::main(global)
            .submodule_ref("JlrsPyPlot")
            .unwrap()
            .wrapper_unchecked()
            .function_ref("jlrsplot")
            .unwrap()
            .wrapper_unchecked()
            .call(frame, vals)?
            .into_jlrs_result()?;

        Ok(PyPlot(plt))
    }

    /// Create a new plotting window by calling `plotfn(args...; keywords)`. The window stays open
    /// until it has been closed, even if all handles have been dropped. `plot_fn` must be a
    /// plotting function from the Plots.jl package, such as `plot` or `hexbin`. The resources
    /// associated  with the window are only cleaned up if one of the `PyPlot::wait` methods is
    /// called.
    pub unsafe fn new_with_keywords<'value, V, F>(
        frame: &mut F,
        plot_fn: Function<'_, 'static>,
        args: V,
        keywords: Value<'_, 'static>,
    ) -> JlrsResult<Self>
    where
        V: AsRef<[Value<'value, 'static>]>,
        F: Frame<'scope>,
    {
        let global = frame.global();
        let args = args.as_ref();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + args.len());
        vals.push(plot_fn.as_value());

        for arg in args.iter().copied() {
            vals.push(arg);
        }

        let plt = Module::main(global)
            .submodule_ref("JlrsPyPlot")
            .unwrap()
            .wrapper_unchecked()
            .function_ref("jlrsplot")
            .unwrap()
            .wrapper_unchecked()
            .with_keywords(keywords)?
            .call(frame, vals)?
            .into_jlrs_result()?;

        Ok(PyPlot(plt))
    }

    /// Update an existing plotting window by calling
    /// `plotfn(<plot associated with self>, args...)`. If the window has already been closed an
    /// error is returned. Note that if multiple plotting windows are currently open, only the
    /// most recently created one is redrawn automatically.
    pub unsafe fn update<'value, 'frame, V, F>(
        self,
        frame: &mut F,
        plot_fn: Function<'_, 'static>,
        args: V,
    ) -> JlrsResult<isize>
    where
        V: AsRef<[Value<'value, 'static>]>,
        F: Frame<'frame>,
    {
        let global = frame.global();

        let args = args.as_ref();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(2 + args.len());
        vals.push(self.0);
        vals.push(plot_fn.as_value());

        for arg in args.iter().copied() {
            vals.push(arg);
        }

        Module::main(global)
            .submodule_ref("JlrsPyPlot")
            .unwrap()
            .wrapper_unchecked()
            .function_ref("updateplot!")
            .unwrap()
            .wrapper_unchecked()
            .call(frame, vals)?
            .into_jlrs_result()?
            .unbox::<isize>()
    }

    /// Update an existing plotting window by calling
    /// `plotfn(<plot associated with self>, args...; kwargs...)`. If the window has already been
    /// closed an error is returned. Note that if multiple plotting windows are currently open,
    /// only the most recently created one is redrawn automatically.
    pub unsafe fn update_with_keywords<'value, 'frame, V, F>(
        self,
        frame: &mut F,
        plot_fn: Function<'_, 'static>,
        args: V,
        keywords: Value<'_, 'static>,
    ) -> JlrsResult<isize>
    where
        V: AsRef<[Value<'value, 'static>]>,
        F: Frame<'frame>,
    {
        let global = frame.global();

        let args = args.as_ref();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(2 + args.len());
        vals.push(self.0);
        vals.push(plot_fn.as_value());

        for arg in args.iter().copied() {
            vals.push(arg);
        }

        Module::main(global)
            .submodule_ref("JlrsPyPlot")
            .unwrap()
            .wrapper_unchecked()
            .function_ref("updateplot!")
            .unwrap()
            .wrapper_unchecked()
            .with_keywords(keywords)?
            .call(frame, vals)?
            .into_jlrs_result()?
            .unbox::<isize>()
    }

    /// Wait until the window associated with `self` has been closed.
    pub fn wait<'frame, F: Frame<'frame>>(self, frame: &mut F) -> JlrsResult<()> {
        unsafe {
            let global = frame.global();

            Module::base(global)
                .function_ref("wait")?
                .wrapper_unchecked()
                .call1(frame, self.0)?
                .into_jlrs_result()?;

            Ok(())
        }
    }

    /// Whenever a plot is updated with a non-mutating plotting function a new version is
    /// created. Because all versions are protected from garbage collection until [`PyPlot::wait`]
    /// has returned, it's possible to change the pending version which will be used as the base
    /// plot when [`PyPlot::update`] is called.
    pub fn set_pending_version<'frame, F: Frame<'frame>>(
        self,
        frame: &mut F,
        version: isize,
    ) -> JlrsResult<()> {
        frame.scope(|frame| unsafe {
            let global = frame.global();
            let version = Value::new(&mut *frame, version)?;

            Module::main(global)
                .submodule_ref("JlrsPyPlot")
                .unwrap()
                .wrapper_unchecked()
                .function_ref("setversion")
                .unwrap()
                .wrapper_unchecked()
                .call1(&mut *frame, version)?
                .into_jlrs_result()?;

            Ok(())
        })
    }

    /// Wait until the window associated with `self` has been closed in a new task scheduled
    /// on the main thread.
    #[cfg(feature = "async-rt")]
    pub async fn wait_async_main<'frame>(self, frame: &mut AsyncGcFrame<'frame>) -> JlrsResult<()> {
        unsafe {
            let global = frame.global();

            Module::base(global)
                .function_ref("wait")?
                .wrapper_unchecked()
                .call_async_main(frame, &mut [self.0])
                .await?
                .into_jlrs_result()?;

            Ok(())
        }
    }

    /// Wait until the window associated with `self` has been closed in a new task scheduled
    /// on another thread.
    #[cfg(feature = "async-rt")]
    pub async fn wait_async_local<'frame>(
        self,
        frame: &mut AsyncGcFrame<'frame>,
    ) -> JlrsResult<()> {
        unsafe {
            let global = frame.global();

            Module::base(global)
                .function_ref("wait")?
                .wrapper_unchecked()
                .call_async_local(frame, &mut [self.0])
                .await?
                .into_jlrs_result()?;

            Ok(())
        }
    }

    /// Wait until the window associated with `self` has been closed in a new task scheduled
    /// on another thread.
    #[cfg(feature = "async-rt")]
    pub async fn wait_async<'frame>(self, frame: &mut AsyncGcFrame<'frame>) -> JlrsResult<()> {
        unsafe {
            let global = frame.global();

            Module::base(global)
                .function_ref("wait")?
                .wrapper_unchecked()
                .call_async(frame, &mut [self.0])
                .await?
                .into_jlrs_result()?;

            Ok(())
        }
    }
}

/// This trait is, and can only be, implemented by [`Module`]. It adds the method `Module::plots`
/// that provides access to the contents of the `Plots` package.
pub trait AccessPlotsModule: private::AccessPlotsModule {
    /// Returns the `Plots` module.
    fn plots<'global>(global: Global<'global>) -> Module<'global> {
        unsafe {
            Module::main(global)
                .submodule_ref("JlrsPyPlot")
                .unwrap()
                .wrapper_unchecked()
                .submodule_ref("Plots")
                .unwrap()
                .wrapper_unchecked()
        }
    }
}

impl<'scope> AccessPlotsModule for Module<'scope> {}

mod private {
    use crate::wrappers::ptr::module::Module;

    pub trait AccessPlotsModule {}

    impl<'scope> AccessPlotsModule for Module<'scope> {}
}
