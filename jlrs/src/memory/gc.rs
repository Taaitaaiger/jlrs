//! Manage the garbage collector.

#[julia_version(since = "1.10")]
use jl_sys::jl_gc_set_max_memory;
use jl_sys::{
    jl_gc_collect, jl_gc_collection_t, jl_gc_enable, jl_gc_is_enabled, jl_gc_mark_queue_obj,
    jl_gc_mark_queue_objarray, jl_gc_safepoint, jl_gc_wb,
};
use jlrs_macros::julia_version;

use super::{target::Target, PTls};
#[cfg(feature = "sync-rt")]
use crate::runtime::sync_rt::Julia;
#[julia_version(since = "1.7")]
use crate::{call::Call, data::managed::module::Module};
use crate::{
    data::managed::{
        private::ManagedPriv,
        value::{Value, ValueRef},
    },
    private::Private,
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
/// [`Julia`] and all implementations of [`Target`]
pub trait Gc: private::GcPriv {
    /// Enable or disable the GC.
    fn enable_gc(&self, on: bool) -> bool {
        // Safety: this function is called with a valid argument and can only be called while
        // Julia is active.
        unsafe { jl_gc_enable(on as i32) != 0 }
    }

    #[julia_version(since = "1.7")]
    /// Enable or disable GC logging.
    ///
    /// This method is not available when the `lts` feature is enabled.
    fn enable_gc_logging(&self, on: bool) {
        // Safety: Julia is active, this method is called from a thread known to Julia, and no
        // Julia data is returned by this method.

        use super::target::unrooted::Unrooted;

        let global = unsafe { Unrooted::new() };

        // Safety: everything is globally rooted.
        let func = unsafe {
            Module::base(&global)
                .submodule(&global, "GC")
                .expect("No GC module in Base")
                .as_managed()
                .function(&global, "enable_logging")
                .expect("No enable_logging function in GC")
                .as_managed()
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

    #[julia_version(since = "1.10")]
    /// Set GC memory trigger in bytes for greedy memory collecting
    fn gc_set_max_memory(max_mem: u64) {
        unsafe { jl_gc_set_max_memory(max_mem) }
    }
}

/// Mark `obj`, returns `true` if `obj` points to young data.
///
/// This method can be used to implement custom mark functions. If a foreign type contains
/// references to Julia data, a custom `mark` function must be implemented that calls this
/// function on each of those references.
///
/// Safety
///
/// This method must only be called from `ForeignType::mark`.
pub unsafe fn mark_queue_obj(ptls: PTls, obj: ValueRef) -> bool {
    jl_gc_mark_queue_obj(ptls, obj.ptr().as_ptr()) != 0
}

/// Mark `objs`.
///
/// This method can be used to implement custom mark functions. If a foreign type contains
/// references to Julia data, a custom `mark` function must be implemented. This method can be
/// used on arrays of references to Julia data instead of calling [`mark_queue_obj`] for each
/// reference in that array.
///
/// Safety
///
/// This method must only be called from `ForeignType::mark`.
pub unsafe fn mark_queue_objarray(ptls: PTls, parent: ValueRef, objs: &[Option<ValueRef>]) {
    jl_gc_mark_queue_objarray(ptls, parent.ptr().as_ptr(), objs.as_ptr() as _, objs.len())
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
/// managed by the GC.
pub unsafe fn write_barrier<T>(data: &mut T, child: Value) {
    jl_gc_wb(data as *mut _ as *mut _, child.unwrap(Private))
}

/*
void jl_gc_queue_multiroot(const jl_value_t *parent, const jl_value_t *ptr) JL_NOTSAFEPOINT
{
    // first check if this is really necessary
    // TODO: should we store this info in one of the extra gc bits?
    jl_datatype_t *dt = (jl_datatype_t*)jl_typeof(ptr);
    const jl_datatype_layout_t *ly = dt->layout;
    uint32_t npointers = ly->npointers;
    //if (npointers == 0) // this was checked by the caller
    //    return;
    jl_value_t *ptrf = ((jl_value_t**)ptr)[ly->first_ptr];
    if (ptrf && (jl_astaggedvalue(ptrf)->bits.gc & 1) == 0) {
        // this pointer was young, move the barrier back now
        jl_gc_wb_back(parent);
        return;
    }
    const uint8_t *ptrs8 = (const uint8_t *)jl_dt_layout_ptrs(ly);
    const uint16_t *ptrs16 = (const uint16_t *)jl_dt_layout_ptrs(ly);
    const uint32_t *ptrs32 = (const uint32_t*)jl_dt_layout_ptrs(ly);
    for (size_t i = 1; i < npointers; i++) {
        uint32_t fld;
        if (ly->fielddesc_type == 0) {
            fld = ptrs8[i];
        }
        else if (ly->fielddesc_type == 1) {
            fld = ptrs16[i];
        }
        else {
            assert(ly->fielddesc_type == 2);
            fld = ptrs32[i];
        }
        jl_value_t *ptrf = ((jl_value_t**)ptr)[fld];
        if (ptrf && (jl_astaggedvalue(ptrf)->bits.gc & 1) == 0) {
            // this pointer was young, move the barrier back now
            jl_gc_wb_back(parent);
            return;
        }
    }
}
 */

#[cfg(feature = "sync-rt")]
impl Gc for Julia<'_> {}
impl<'frame, T: Target<'frame>> Gc for T {}

mod private {
    use crate::memory::target::Target;
    #[cfg(feature = "sync-rt")]
    use crate::runtime::sync_rt::Julia;
    pub trait GcPriv {}
    impl<'frame, T: Target<'frame>> GcPriv for T {}
    #[cfg(feature = "sync-rt")]
    impl GcPriv for Julia<'_> {}
}
