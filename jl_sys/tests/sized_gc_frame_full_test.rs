use jl_sys::bindings::{jl_init, jl_is_initialized};

#[test]
fn gc_frame_tests() {
    unsafe {
        jl_init();
        assert!(jl_is_initialized() != 0);
    }
}
