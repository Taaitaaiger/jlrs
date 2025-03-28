mod util;

#[cfg(feature = "local-rt")]
mod tests {
    /*
    struct HasConstructors
        a::DataType
        b::Union{Int16, Int32}
        HasConstructors(i::Int16) = new(Int16, i)
        HasConstructors(i::Int32) = new(Int32, i)
        HasConstructors(v::Bool) = new(Bool, v ? one(Int32) : zero(Int32))
    end

    HasConstructors() = HasConstructors(false)
    */

    use jlrs::prelude::*;

    use crate::util::JULIA;

    fn call_outer_constructor() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        unsafe {
                            let ty = Module::main(&frame)
                                .submodule(&frame, "JlrsTests")?
                                .as_managed()
                                .global(&frame, "HasConstructors")?
                                .as_value();

                            assert!(ty.is::<DataType>());

                            let res = ty.call0(&mut frame);
                            assert!(res.is_ok());
                            let value = res.unwrap();
                            let is_bool = value
                                .field_accessor()
                                .field("a")?
                                .access::<WeakDataType>()?
                                .as_managed()
                                .is::<Bool>();

                            assert!(is_bool);

                            let field_b = value.field_accessor().field("b")?.access::<i32>()?;

                            assert_eq!(field_b, 0);
                        };

                        Ok(())
                    })
                    .unwrap();

                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        unsafe {
                            let ty = Module::main(&frame)
                                .submodule(&frame, "JlrsTests")?
                                .as_managed()
                                .global(&frame, "HasConstructors")?
                                .as_value();

                            assert!(ty.is::<DataType>());

                            let res = ty.call0(&mut frame);
                            assert!(res.is_ok());
                            let value = res.unwrap();
                            let is_bool = value
                                .field_accessor()
                                .field("a")?
                                .access::<WeakDataType>()?
                                .as_managed()
                                .is::<Bool>();

                            assert!(is_bool);

                            let field_b = value.field_accessor().field("b")?.access::<i32>()?;

                            assert_eq!(field_b, 0);
                        };

                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn call_inner_constructor() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        unsafe {
                            let ty = Module::main(&frame)
                                .submodule(&frame, "JlrsTests")?
                                .as_managed()
                                .global(&frame, "HasConstructors")?
                                .as_value();

                            let arg = Value::new(&mut frame, 1i16);

                            let res = ty.call1(&mut frame, arg);
                            assert!(res.is_ok());
                            let value = res.unwrap();
                            let is_i16 = value
                                .field_accessor()
                                .field("a")?
                                .access::<WeakDataType>()?
                                .as_managed()
                                .is::<i16>();

                            assert!(is_i16);

                            let field_b = value.field_accessor().field("b")?.access::<i16>()?;

                            assert_eq!(field_b, 1);
                        };

                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    #[test]
    fn constructor_tests() {
        call_outer_constructor();
        call_inner_constructor();
    }
}
