mod util;
#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    #[test]
    fn read_abstract_field() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "WithAbstract")?
                            .value()
                    };

                    let arg1 = Value::new(&mut frame, 3u32);
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
