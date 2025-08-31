use jl_sys::{
    bindings::{jl_alloc_array_1d, jl_array_any_type, jl_init, jl_is_initialized},
    gc_frame::unsized_local_scope,
};

fn test_unsized_sized_gc_frame_full() {
    unsafe {
        unsized_local_scope(0, |mut frame| {
            let v = jl_alloc_array_1d(jl_array_any_type, 1);
            frame.root_value(0, v.cast());
        });
    }
}

#[test]
#[should_panic]
fn unsized_gc_frame_full_test() {
    unsafe {
        jl_init();
        assert!(jl_is_initialized() != 0);
        test_unsized_sized_gc_frame_full();
    }
}
