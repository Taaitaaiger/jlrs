mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn ptr_union_fields_access_something() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|frame| unsafe {
                    let field = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "has_module")
                        .unwrap()
                        .as_value()
                        .field_accessor()
                        .field("a")
                        .unwrap()
                        .access::<WeakValue>()
                        .unwrap();

                    assert!(field.as_value().is::<Module>());
                })
            })
        })
    }

    fn ptr_union_fields_nothing_is_not_null() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|frame| unsafe {
                    let _field = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "has_nothing")
                        .unwrap()
                        .as_value()
                        .field_accessor()
                        .field("a")
                        .unwrap()
                        .access::<Nothing>()
                        .unwrap();
                })
            })
        })
    }

    #[test]
    fn union_layout_tests() {
        ptr_union_fields_access_something();
        ptr_union_fields_nothing_is_not_null();
    }
}
