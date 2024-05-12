use jl_sys::bindings::{jl_is_initialized, jlrs_init};

#[test]
fn gc_frame_tests() {
    unsafe {
        jlrs_init();
        assert!(jl_is_initialized() != 0);
    }
}
