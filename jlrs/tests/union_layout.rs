mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn ptr_union_fields_access_something() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let field = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .wrapper_unchecked()
                        .global(&frame, "has_module")?
                        .value_unchecked()
                        .field_accessor(&mut frame)
                        .field("a")?
                        .access::<ValueRef>()?;

                    assert!(!field.is_undefined());
                    assert!(field.value_unchecked().is::<Module>());

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn ptr_union_fields_nothing_is_not_null() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let _field = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .wrapper_unchecked()
                        .global(&frame, "has_nothing")?
                        .value_unchecked()
                        .field_accessor(&mut frame)
                        .field("a")?
                        .access::<Nothing>()?;

                    Ok(())
                })
                .unwrap();
        })
    }
}
