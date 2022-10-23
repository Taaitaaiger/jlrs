//! Manage the garbage collector.

use std::ffi::c_void;

#[cfg(feature = "sync-rt")]
use crate::runtime::sync_rt::Julia;
use crate::wrappers::ptr::value::Value;
#[cfg(not(feature = "lts"))]
use crate::{call::Call, wrappers::ptr::module::Module};
use crate::{private::Private, wrappers::ptr::private::WrapperPriv};

use jl_sys::{
    jl_gc_collect, jl_gc_collection_t, jl_gc_enable, jl_gc_is_enabled, jl_gc_mark_queue_obj,
    jl_gc_mark_queue_objarray, jl_gc_safepoint, jl_gc_wb,
};

use super::{target::Target, PTls};

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
/// [`Julia`] and all implementations of [`Target`]
pub trait Gc: private::GcPriv {
    /// Enable or disable the GC.
    fn enable_gc(&self, on: bool) -> bool {
        // Safety: this function is called with a valid argument and can only be called while
        // Julia is active.
        unsafe { jl_gc_enable(on as i32) != 0 }
    }

    /// Enable or disable GC logging.
    ///
    /// This method is not available when the `lts` feature is enabled.
    #[cfg(not(feature = "lts"))]
    fn enable_gc_logging(&self, on: bool) {
        // Safety: Julia is active, this method is called from a thread known to Julia, and no
        // Julia data is returned by this method.

        use super::target::global::Global;

        let global = unsafe { Global::new() };

        // Safety: everything is globally rooted.
        let func = unsafe {
            Module::base(&global)
                .submodule(&global, "GC")
                .expect("No GC module in Base")
                .wrapper_unchecked()
                .function(&global, "enable_logging")
                .expect("No enable_logging function in GC")
                .wrapper_unchecked()
        };

        let arg = if on {
            Value::true_v(&global)
        } else {
            Value::false_v(&global)
        };

        // Safety: GC.enable_logging is safe to call.
        unsafe { func.call1(&global, arg) }.expect("GC.enable_logging threw an exception");
    }

    /// Returns `true` if the GC is enabled.
    fn gc_is_enabled(&self) -> bool {
        // Safety: this function can only be called while Julia is active from a thread known to
        // Julia.
        unsafe { jl_gc_is_enabled() != 0 }
    }

    /// Force a collection.
    fn gc_collect(&self, mode: GcCollection) {
        // Safety: this function can only be called while Julia is active from a thread known to
        // Julia.
        unsafe { jl_gc_collect(mode as jl_gc_collection_t) }
    }

    /// Insert a safepoint, a point where the garbage collector may run.
    fn gc_safepoint(&self) {
        // Safety: this function can only be called while Julia is active from a thread known to
        // Julia.
        unsafe {
            jl_gc_safepoint();
        }
    }
}

// TODO
pub unsafe fn mark_queue_obj(ptls: PTls, obj: *mut c_void) -> bool {
    jl_gc_mark_queue_obj(ptls, obj.cast()) != 0
}

pub unsafe fn mark_queue_objarray(ptls: PTls, parent: *mut c_void, objs: &[*mut c_void]) {
    jl_gc_mark_queue_objarray(
        ptls,
        parent.cast(),
        objs.as_ptr() as *mut c_void as _,
        objs.len(),
    )
}

/// Updates the write barrier.
///
/// When a pointer field of `data` has been set to `child`, this method must be called
/// immediately after changing the field. This must only be done when the child has been
/// mutated by directly changing the field and `data` is managed by Julia's GC.
///
/// This is necessary because the GC must remain aware of all old objects that contain
/// references to young objects.
///
/// Safety: must be called whenever a field of `self` is set to `child` if `self` is
/// maanged by the GC.
pub unsafe fn write_barrier<T>(data: &mut T, child: Value) {
    jl_gc_wb(data as *mut _ as *mut _, child.unwrap(Private))
}

#[cfg(feature = "sync-rt")]
impl Gc for Julia<'_> {}
impl<'frame, 'data, T: Target<'frame, 'data>> Gc for T {}

mod private {
    use crate::memory::target::Target;
    #[cfg(feature = "sync-rt")]
    use crate::runtime::sync_rt::Julia;
    pub trait GcPriv {}
    impl<'frame, 'data, T: Target<'frame, 'data>> GcPriv for T {}
    #[cfg(feature = "sync-rt")]
    impl GcPriv for Julia<'_> {}
}
