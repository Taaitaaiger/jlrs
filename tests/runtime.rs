use jlrs::prelude::*;

#[test]
fn error_include_nonexistent() {
    let mut jlrs = unsafe { Julia::testing_instance() };
    assert!(jlrs.include("nonexistent/path/").is_err());
}

#[test]
fn change_stack_size() {
    let mut jlrs = unsafe { Julia::testing_instance() };
    assert_ne!(jlrs.stack_size(), 8);
    jlrs.set_stack_size(8);
    assert_eq!(jlrs.stack_size(), 8);
}
