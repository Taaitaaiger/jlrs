//! Control the garbage collector.

use super::frame::Frame;
#[cfg(feature = "sync-rt")]
use crate::julia::Julia;
use jl_sys::{jl_gc_collect, jl_gc_collection_t, jl_gc_enable, jl_gc_is_enabled, jl_gc_safepoint};

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
    unsafe fn enable_gc(&mut self, on: bool) -> bool {
        jl_gc_enable(on as i32) == 1
    }

    /// Returns `true` if the GC is enabled.
    fn gc_is_enabled(&mut self) -> bool {
        unsafe { jl_gc_is_enabled() == 1 }
    }

    /// Force a collection.
    unsafe fn gc_collect(&mut self, mode: GcCollection) {
        jl_gc_collect(mode as jl_gc_collection_t)
    }

    /// Insert a safepoint, a point where the garbage collector may run.
    unsafe fn gc_safepoint(&mut self) {
        jl_gc_safepoint();
    }
}

#[cfg(feature = "sync-rt")]
impl Gc for Julia {}
impl<'frame, T: Frame<'frame>> Gc for T {}

mod private {
    use super::Frame;
    #[cfg(feature = "sync-rt")]
    use crate::julia::Julia;
    pub trait Gc {}
    impl<'a, F: Frame<'a>> Gc for F {}
    #[cfg(feature = "sync-rt")]
    impl Gc for Julia {}
}
