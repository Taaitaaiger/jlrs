//! Manage the garbage collector.

pub use jl_sys::GcCollection;
use jl_sys::{
    jl_gc_collect, jl_gc_collection_t, jl_gc_enable, jl_gc_is_enabled, jl_gc_mark_queue_obj,
    jl_gc_mark_queue_objarray, jl_gc_safepoint, jlrs_gc_safe_enter, jlrs_gc_safe_leave,
    jlrs_gc_unsafe_enter, jlrs_gc_unsafe_leave, jlrs_gc_wb, jlrs_ppgcstack,
};
use jlrs_macros::julia_version;

use super::{
    get_tls,
    target::{unrooted::Unrooted, Target},
    PTls,
};
#[cfg(feature = "local-rt")]
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

/// Manage the GC.
///
/// This trait provides several methods that can be used to enable or disable the GC, force a
/// collection, insert a safepoint, and to enable and disable GC logging. It's implemented for
/// [`Julia`] and all [`Target`]s.
pub trait Gc: private::GcPriv {
    /// Enable or disable the GC.
    #[inline]
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
    #[inline]
    fn gc_is_enabled(&self) -> bool {
        // Safety: this function can only be called while Julia is active from a thread known to
        // Julia.
        unsafe { jl_gc_is_enabled() != 0 }
    }

    /// Force a collection.
    #[inline]
    fn gc_collect(&self, mode: GcCollection) {
        // Safety: this function can only be called while Julia is active from a thread known to
        // Julia.
        unsafe { jl_gc_collect(mode as jl_gc_collection_t) }
    }

    /// Force `n` collections. This should only be used to investigate GC-related bugs.
    #[inline]
    fn gc_collect_n(&self, mode: GcCollection, n: usize) {
        // Safety: this function can only be called while Julia is active from a thread known to
        // Julia.
        for _i in 0..n {
            self.gc_collect(mode);
        }
    }

    /// Insert a safepoint, a point where the garbage collector may run.
    #[inline]
    fn gc_safepoint(&self) {
        // Safety: this function can only be called while Julia is active from a thread known to
        // Julia.
        unsafe {
            jl_gc_safepoint();
        }
    }

    /// Put the current task in a GC-safe state.
    ///
    /// In a GC-safe state a task must not be calling into Julia, it indicates that the GC is
    /// allowed to collect without waiting for the task to reach an explicit safepoint.
    ///
    /// Safety:
    ///
    /// While in a GC-safe state, you must not call into Julia in any way that. It should only be used
    /// in combination with blocking operations to allow the GC to collect while waiting for the
    /// blocking operation to complete.
    ///
    /// You must leave the GC-safe state by calling [`Gc::gc_safe_leave`] with the state returned
    /// by this function.
    #[inline]
    unsafe fn gc_safe_enter() -> i8 {
        let ptls = get_tls();
        jlrs_gc_safe_enter(ptls)
    }

    /// Leave a GC-safe region and return to the previous GC-state.
    ///
    /// Safety:
    ///
    /// Must be called with the state returned by a matching call to [`Gc::gc_safe_enter`].
    #[inline]
    unsafe fn gc_safe_leave(state: i8) {
        let ptls = get_tls();
        jlrs_gc_safe_leave(ptls, state)
    }

    /// Put the current task in a GC-unsafe state.
    ///
    /// In a GC-unsafe state a task must reach an explicit safepoint before the GC can collect.
    ///
    /// Safety:
    ///
    /// This function must only be called while the task is in a GC-safe state. After calling this
    /// function the task may call into Julia again.
    ///
    /// You must leave the GC-safe state by calling [`Gc::gc_unsafe_leave`] with the state
    /// returned by this function.
    #[inline]
    unsafe fn gc_unsafe_enter() -> i8 {
        let ptls = get_tls();
        jlrs_gc_unsafe_enter(ptls)
    }

    /// Leave a GC-unsafe region and return to the previous GC-state.
    ///
    /// Safety:
    ///
    /// Must be called with the state returned by a matching call to [`Gc::gc_unsafe_enter`].
    #[inline]
    unsafe fn gc_unsafe_leave(state: i8) {
        let ptls = get_tls();
        jlrs_gc_unsafe_leave(ptls, state)
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
#[inline]
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
#[inline]
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
#[inline]
pub unsafe fn write_barrier<T>(data: &mut T, child: Value) {
    jlrs_gc_wb(data as *mut _ as *mut _, child.unwrap(Private).cast())
}

/// Put the current task in a GC-safe state, call `f`, and return to the previous GC state.
///
/// This must only be used when long-running functions that don't call into Julia are called from
/// a thread that can call into Julia. It puts the current task into a GC-safe state, this can be
/// thought of as extended safepoint: a task that is in a GC-safe state allows the GC to collect
/// garbage as if it had reached a safepoint.
///
/// Safety:
///
/// - This function must be called from a thread that can call into Julia.
/// - `f` must not call into Julia in any way, except inside a function called with `gc_unsafe`.
#[inline]
pub unsafe fn gc_safe<F: FnOnce() -> T, T>(f: F) -> T {
    let pgc = jlrs_ppgcstack();
    if pgc.is_null() {
        return f();
    }

    let ptls = get_tls();
    let state = jlrs_gc_safe_enter(ptls);
    let res = f();
    jlrs_gc_safe_leave(ptls, state);

    res
}

#[inline]
#[cfg(feature = "async")]
pub(crate) unsafe fn gc_safe_with<F: FnOnce() -> T, T>(ptls: PTls, f: F) -> T {
    let state = jlrs_gc_safe_enter(ptls);
    let res = f();
    jlrs_gc_safe_leave(ptls, state);

    res
}

/// Put the current task in a GC-unsafe state, call `f`, and return to the previous GC state.
///
/// This should only be used in a function called with [`gc_safe`]. It puts the task back into a
/// GC=unsafe state. If the task is already in an GC-unsafe state calling this function has no
/// effect.
///
/// Safety:
///
/// - This function must be called from a thread that can call into Julia.
#[inline]
pub unsafe fn gc_unsafe<F: for<'scope> FnOnce(Unrooted<'scope>) -> T, T>(f: F) -> T {
    debug_assert!(!jlrs_ppgcstack().is_null());
    let ptls = get_tls();

    let unrooted = Unrooted::new();
    let state = jlrs_gc_unsafe_enter(ptls);
    let res = f(unrooted);
    jlrs_gc_unsafe_leave(ptls, state);

    res
}

#[cfg(feature = "async")]
pub(crate) unsafe fn gc_unsafe_with<F: for<'scope> FnOnce(Unrooted<'scope>) -> T, T>(
    ptls: PTls,
    f: F,
) -> T {
    let state = jlrs_gc_unsafe_enter(ptls);
    let unrooted = Unrooted::new();
    let res = f(unrooted);
    jlrs_gc_unsafe_leave(ptls, state);

    res
}

#[cfg(feature = "local-rt")]
impl Gc for Julia<'_> {}
impl<'frame, Tgt: Target<'frame>> Gc for Tgt {}

mod private {
    use crate::memory::target::Target;
    #[cfg(feature = "local-rt")]
    use crate::runtime::sync_rt::Julia;
    pub trait GcPriv {}
    impl<'frame, Tgt: Target<'frame>> GcPriv for Tgt {}
    #[cfg(feature = "local-rt")]
    impl GcPriv for Julia<'_> {}
}
