//! Manage the garbage collector.

use super::frame::Frame;
#[cfg(feature = "sync-rt")]
use crate::runtime::sync_rt::Julia;
use jl_sys::{jl_gc_collect, jl_gc_collection_t, jl_gc_enable, jl_gc_is_enabled, jl_gc_safepoint};

#[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
use crate::{
    call::Call,
    memory::global::Global,
    wrappers::ptr::{module::Module, value::Value},
};

/// The different collection modes.
#[derive(Debug, Copy, Clone)]
pub enum GcCollection {
    Auto = 0,
    Full = 1,
    Incremental = 2,
}

/// Manage the GC.
///
/// This trait provides several methods that can be used to enable or disable the GC, force a
/// collection, insert a safepoint, and to enable and disable GC logging. It's implemented for
/// all mutable reference to implementations of [`Frame`], and [`Julia`].
pub trait Gc: private::GcPriv {
    /// Enable or disable the GC.
    fn enable_gc(&mut self, on: bool) -> bool {
        // Safety: this function is called with a valid argument and can only be called while
        // Julia is active.
        unsafe { jl_gc_enable(on as i32) != 0 }
    }

    /// Enable or disable GC logging.
    ///
    /// This method is not available when the `lts` feature is enabled.
    #[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
    fn enable_gc_logging(&mut self, on: bool) {
        // Safety: Julia is active, this method is called from a thread known to Julia, and no
        // Julia data is returned by this method.
        let global = unsafe { Global::new() };

        // Safety: everything is globally rooted.
        let func = unsafe {
            Module::base(global)
                .submodule_ref("GC")
                .expect("No GC module in Base")
                .wrapper_unchecked()
                .function_ref("enable_logging")
                .expect("No enable_logging function in GC")
                .wrapper_unchecked()
        };

        let arg = if on {
            Value::true_v(global)
        } else {
            Value::false_v(global)
        };

        // Safety: GC.enable_logging is safe to call.
        unsafe { func.call1_unrooted(global, arg) }.expect("GC.enable_logging threw an exception");
    }

    /// Returns `true` if the GC is enabled.
    fn gc_is_enabled(&mut self) -> bool {
        // Safety: this function can only be called while Julia is active from a thread known to
        // Julia.
        unsafe { jl_gc_is_enabled() != 0 }
    }

    /// Force a collection.
    fn gc_collect(&mut self, mode: GcCollection) {
        // Safety: this function can only be called while Julia is active from a thread known to
        // Julia.
        unsafe { jl_gc_collect(mode as jl_gc_collection_t) }
    }

    /// Insert a safepoint, a point where the garbage collector may run.
    fn gc_safepoint(&mut self) {
        // Safety: this function can only be called while Julia is active from a thread known to
        // Julia.
        unsafe {
            jl_gc_safepoint();
        }
    }
}

#[cfg(feature = "sync-rt")]
impl Gc for Julia {}
impl<'frame, T: Frame<'frame>> Gc for T {}

mod private {
    use super::Frame;
    #[cfg(feature = "sync-rt")]
    use crate::runtime::sync_rt::Julia;
    pub trait GcPriv {}
    impl<'frame, F: Frame<'frame>> GcPriv for F {}
    #[cfg(feature = "sync-rt")]
    impl GcPriv for Julia {}
}
