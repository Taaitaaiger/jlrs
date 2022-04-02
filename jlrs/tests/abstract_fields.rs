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
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("WithAbstract")?
                        .value_unchecked()
                };

                let arg1 = Value::new(&mut *frame, 3u32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1])?
                    .into_jlrs_result()?;

                let a = instance.get_raw_field::<Value, _>("a")?;
                assert_eq!(a.unbox::<u32>()?, 3);

                Ok(())
            })
            .unwrap();
        })
    }
}
