use std::{
    os::raw::c_void,
    sync::atomic::{AtomicBool, Ordering},
};

use jl_sys::{
    bindings::{
        jl_alloc_array_1d, jl_array_any_type, jl_gc_add_ptr_finalizer, jl_gc_collect, jl_init,
        jl_is_initialized, jlrs_get_ptls_states,
    },
    gc_frame::{sized_local_scope, unsized_local_scope},
    types::jl_gc_collection_t,
};

static FINALIZED: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn finalizer(_: *mut c_void) {
    FINALIZED.store(true, Ordering::Relaxed)
}

fn test_sized_gc_frame() {
    FINALIZED.store(false, Ordering::Relaxed);

    unsafe {
        sized_local_scope::<_, _, 1>(|mut frame| {
            let v = jl_alloc_array_1d(jl_array_any_type, 1);
            frame.root_value(0, v.cast());

            let ptls = jlrs_get_ptls_states();
            jl_gc_add_ptr_finalizer(ptls, v.cast(), finalizer as *mut c_void);

            jl_gc_collect(jl_gc_collection_t::Full);
            jl_gc_collect(jl_gc_collection_t::Full);
            jl_gc_collect(jl_gc_collection_t::Full);
            assert!(!FINALIZED.load(Ordering::Relaxed));
        });

        jl_gc_collect(jl_gc_collection_t::Full);
        jl_gc_collect(jl_gc_collection_t::Full);
        jl_gc_collect(jl_gc_collection_t::Full);
        assert!(FINALIZED.load(Ordering::Relaxed));
    }
}

fn test_unsized_gc_frame() {
    FINALIZED.store(false, Ordering::Relaxed);

    unsafe {
        unsized_local_scope(1, |mut frame| {
            let v = jl_alloc_array_1d(jl_array_any_type, 1);
            frame.root_value(0, v.cast());

            let ptls = jlrs_get_ptls_states();
            jl_gc_add_ptr_finalizer(ptls, v.cast(), finalizer as *mut c_void);

            jl_gc_collect(jl_gc_collection_t::Full);
            jl_gc_collect(jl_gc_collection_t::Full);
            jl_gc_collect(jl_gc_collection_t::Full);
            assert!(!FINALIZED.load(Ordering::Relaxed));
        });

        jl_gc_collect(jl_gc_collection_t::Full);
        jl_gc_collect(jl_gc_collection_t::Full);
        jl_gc_collect(jl_gc_collection_t::Full);
        assert!(FINALIZED.load(Ordering::Relaxed));
    }
}

#[test]
fn gc_frame_tests() {
    unsafe {
        // Test that data is not freed until frame has been popped from GC stack.

        jl_init();
        assert!(jl_is_initialized() != 0);

        test_sized_gc_frame();
        test_unsized_gc_frame();
    }
}
