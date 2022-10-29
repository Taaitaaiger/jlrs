mod util;

#[cfg(feature = "sync-rt")]
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

    use crate::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn call_outer_constructor() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    unsafe {
                        let ty = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper_unchecked()
                            .global(&frame, "HasConstructors")?
                            .value_unchecked();

                        assert!(ty.is::<DataType>());

                        let res = ty.call0(&mut frame);
                        assert!(res.is_ok());
                        let value = res.unwrap();
                        let is_bool = value
                            .field_accessor(&frame)
                            .field("a")?
                            .access::<DataTypeRef>()?
                            .wrapper_unchecked()
                            .is::<Bool>();

                        assert!(is_bool);

                        let field_b = value.field_accessor(&frame).field("b")?.access::<i32>()?;

                        assert_eq!(field_b, 0);
                    };

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn call_inner_constructor() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    unsafe {
                        let ty = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper_unchecked()
                            .global(&frame, "HasConstructors")?
                            .value_unchecked();

                        let arg = Value::new(&mut frame, 1i16);

                        let res = ty.call1(&mut frame, arg);
                        assert!(res.is_ok());
                        let value = res.unwrap();
                        let is_i16 = value
                            .field_accessor(&frame)
                            .field("a")?
                            .access::<DataTypeRef>()?
                            .wrapper_unchecked()
                            .is::<i16>();

                        assert!(is_i16);

                        let field_b = value.field_accessor(&frame).field("b")?.access::<i16>()?;

                        assert_eq!(field_b, 1);
                    };

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn call_instantiate() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    unsafe {
                        let ty = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper_unchecked()
                            .global(&frame, "HasConstructors")?
                            .value_unchecked();

                        let arg = Value::new(&mut frame, 1i16);
                        let args = [DataType::int64_type(&frame).as_value(), arg];

                        let value = ty
                            .cast::<DataType>()?
                            .instantiate_unchecked(&mut frame, args);

                        let is_i64 = value
                            .field_accessor(&frame)
                            .field("a")?
                            .access::<DataTypeRef>()?
                            .wrapper_unchecked()
                            .is::<i64>();

                        assert!(is_i64);

                        let field_b = value.field_accessor(&frame).field("b")?.access::<i16>()?;

                        assert_eq!(field_b, 1);
                    };

                    Ok(())
                })
                .unwrap();
        });
    }
}
