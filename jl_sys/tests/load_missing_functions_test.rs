use jl_sys::{jl_atexit_hook, jl_init, jl_is_initialized, jlrs_init_missing_functions};

#[test]
fn load_missing_functions() {
    unsafe {
        jl_init();

        assert!(jl_is_initialized() != 0);

        jlrs_init_missing_functions();

        jl_atexit_hook(0);
    }
}
