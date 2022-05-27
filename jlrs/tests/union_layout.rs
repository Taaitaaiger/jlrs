mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;
    use jlrs::wrappers::ptr::ValueRef;

    #[test]
    fn ptr_union_fields_access_something() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_capacity(4, |global, frame| unsafe {
                let field = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .global_ref("has_module")?
                    .value_unchecked()
                    .field_accessor(frame)
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
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_capacity(4, |global, frame| unsafe {
                let _field = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .global_ref("has_nothing")?
                    .value_unchecked()
                    .field_accessor(frame)
                    .field("a")?
                    .access::<Nothing>()?;

                Ok(())
            })
            .unwrap();
        })
    }
}