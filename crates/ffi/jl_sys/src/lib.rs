pub mod bindings;
pub mod types;

pub use bindings::*;
pub use types::*;

#[cfg(not(all(target_os = "windows", target_env = "gnu")))]
#[cfg(test)]
mod tests {
    use crate::{
        jl_atexit_hook, jl_eval_string, jl_exception_occurred, jl_init, jl_is_initialized,
    };

    #[test]
    fn sanity_test() {
        unsafe {
            assert!(jl_is_initialized() == 0);
            jl_init();
            assert!(jl_is_initialized() != 0);

            let s = c"sqrt(2.0)";
            let res = jl_eval_string(s.as_ptr());
            assert!(!res.is_null());
            assert!(jl_exception_occurred().is_null());

            jl_atexit_hook(0);
        }
    }
}
