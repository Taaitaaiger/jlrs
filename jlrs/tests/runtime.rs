mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn error_include_nonexistent() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            assert!(jlrs.include("nonexistent/path/").is_err());
        });
    }

    #[test]
    fn cannot_init_again() {
        JULIA.with(|_j| unsafe {
            assert!(Julia::init().is_err());
        });
    }

    #[test]
    fn include_error() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            assert!(jlrs.include("Cargo.toml").is_err());
        });
    }
}
