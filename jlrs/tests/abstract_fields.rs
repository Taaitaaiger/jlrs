mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{memory::gc::Gc, prelude::*};

    use super::util::JULIA;

    #[test]
    fn read_abstract_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                        let ty = unsafe {
                            Module::main(&frame)
                                .submodule(&frame, "JlrsTests")?
                                .as_managed()
                                .global(&frame, "WithAbstract")?
                                .as_value()
                        };

                        let arg1 = Value::new(&mut frame, 3u32);
                        let instance = ty
                            .cast::<DataType>()?
                            .instantiate(&mut frame, &mut [arg1])?
                            .into_jlrs_result()?;

                        let field = instance.field_accessor().field("a")?.access::<u32>()?;
                        assert_eq!(field, 3);

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }
}
