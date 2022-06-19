mod util;
#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn read_abstract_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("WithAbstract")?
                        .value_unchecked()
                };

                let arg1 = Value::new(&mut frame, 3u32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1])?
                    .into_jlrs_result()?;

                let field = instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<u32>()?;
                assert_eq!(field, 3);

                Ok(())
            })
            .unwrap();
        })
    }
}
