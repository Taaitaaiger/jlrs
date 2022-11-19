mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn error_include_nonexistent() {
        JULIA.with(|j| unsafe {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            assert!(jlrs
                .instance(&mut frame)
                .include("nonexistent/path/")
                .is_err());
        });
    }

    fn cannot_init_again() {
        JULIA.with(|_j| unsafe { assert!(RuntimeBuilder::new().start().is_err()) });
    }

    fn include_error() {
        JULIA.with(|j| unsafe {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            assert!(jlrs.instance(&mut frame).include("Cargo.toml").is_err());
        });
    }

    #[test]
    fn runtime_test() {
        error_include_nonexistent();
        cannot_init_again();
        include_error();
    }
}
