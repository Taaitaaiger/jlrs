//! Manage the garbage collector.

use super::frame::Frame;
#[cfg(feature = "sync-rt")]
use crate::runtime::sync_rt::Julia;
use jl_sys::{jl_gc_collect, jl_gc_collection_t, jl_gc_enable, jl_gc_is_enabled, jl_gc_safepoint};

#[cfg(not(feature = "lts"))]
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

/// This trait is used to enable and disable the garbage collector and to force a collection.
pub trait Gc: private::Gc {
    /// Enable or disable the GC.
    fn enable_gc(&mut self, on: bool) -> bool {
        unsafe { jl_gc_enable(on as i32) == 1 }
    }

    /// Enable or disable GC logging.
    ///
    /// This method is not available when the `lts` feature is enabled.
    #[cfg(not(feature = "lts"))]
    fn enable_logging(&mut self, on: bool) {
        unsafe {
            let global = Global::new();
            let func = Module::base(global)
                .submodule_ref("GC")
                .expect("No GC module in Base")
                .wrapper_unchecked()
                .function_ref("enable_logging")
                .expect("No enable_logging function in GC")
                .wrapper_unchecked();

            let arg = if on {
                Value::true_v(global)
            } else {
                Value::false_v(global)
            };

            func.call1_unrooted(global, arg)
                .expect("GC.enable_logging threw an exception");
        }
    }

    /// Returns `true` if the GC is enabled.
    fn gc_is_enabled(&mut self) -> bool {
        unsafe { jl_gc_is_enabled() == 1 }
    }

    /// Force a collection.
    fn gc_collect(&mut self, mode: GcCollection) {
        unsafe { jl_gc_collect(mode as jl_gc_collection_t) }
    }

    /// Insert a safepoint, a point where the garbage collector may run.
    fn gc_safepoint(&mut self) {
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
    pub trait Gc {}
    impl<'a, F: Frame<'a>> Gc for F {}
    #[cfg(feature = "sync-rt")]
    impl Gc for Julia {}
}
