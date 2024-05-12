mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn ptr_union_fields_access_something() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| unsafe {
                    let field = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .global(&frame, "has_module")?
                        .as_value()
                        .field_accessor()
                        .field("a")?
                        .access::<ValueRef>()?;

                    assert!(field.as_value().is::<Module>());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn ptr_union_fields_nothing_is_not_null() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| unsafe {
                    let _field = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .global(&frame, "has_nothing")?
                        .as_value()
                        .field_accessor()
                        .field("a")?
                        .access::<Nothing>()?;

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn union_layout_tests() {
        ptr_union_fields_access_something();
        ptr_union_fields_nothing_is_not_null();
    }
}
