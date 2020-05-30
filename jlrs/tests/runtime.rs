use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn error_include_nonexistent() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        assert!(jlrs.include("nonexistent/path/").is_err());
    });
}

#[test]
fn change_stack_size() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        assert_ne!(jlrs.stack_size(), 48);
        jlrs.set_stack_size(48);
        assert_eq!(jlrs.stack_size(), 48);
    });
}

#[test]
fn cannot_init_again() {
    JULIA.with(|_j| unsafe {
        assert!(Julia::init(42).is_err());
    });
}

#[test]
fn include_error() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        assert!(jlrs.include("Cargo.toml").is_err());
    });
}
