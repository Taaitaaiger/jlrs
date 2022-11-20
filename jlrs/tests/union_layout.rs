mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn ptr_union_fields_access_something() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|frame| unsafe {
                    let field = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .wrapper()
                        .global(&frame, "has_module")?
                        .value()
                        .field_accessor()
                        .field("a")?
                        .access::<ValueRef>()?;

                    assert!(field.value().is::<Module>());

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
                .scope(|frame| unsafe {
                    let _field = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .wrapper()
                        .global(&frame, "has_nothing")?
                        .value()
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
