mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{
        data::managed::{type_var::TypeVar, union_all::UnionAll},
        prelude::*,
    };

    use super::util::JULIA;

    fn create_new_unionall() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let atype = UnionAll::array_type(&frame);
                    let body = atype.body();
                    let number_type = DataType::number_type(&frame).as_value();
                    let tvar = TypeVar::new(&mut frame, "V", None, Some(number_type))
                        .into_jlrs_result()?;

                    let ua = UnionAll::new(&mut frame, tvar, body)
                        .into_jlrs_result()?
                        .cast::<UnionAll>()?;
                    let v = ua.var();

                    let equals = Module::base(&frame)
                        .function(&frame, "!=")?
                        .as_managed()
                        .call2(&mut frame, v.as_value(), atype.var().as_value())
                        .unwrap()
                        .unbox::<bool>()?
                        .as_bool();
                    assert!(equals);
                    Ok(())
                })
                .unwrap();
        })
    }

    fn instantiate_unionall() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let v = Value::new(&mut frame, 3i8);
                    let args = [DataType::int8_type(&frame).as_value()];
                    let out = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .global(&frame, "ParameterStruct")?
                        .as_managed()
                        .apply_type(&mut frame, args)
                        .into_jlrs_result()?
                        .cast::<DataType>()?
                        .instantiate(&mut frame, [v])?
                        .into_jlrs_result()?
                        .get_field(&mut frame, "a")?
                        .unbox::<i8>()?;

                    assert_eq!(out, 3);
                    Ok(())
                })
                .unwrap();
        })
    }

    fn apply_value_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let ty1 = Value::new(&mut frame, 1isize);
                    let ty2 = Value::new(&mut frame, 2isize);

                    let vts = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .global(&frame, "ValueTypeStruct")?
                        .as_managed();

                    let v1 = vts
                        .apply_type(&mut frame, &mut [ty1])
                        .into_jlrs_result()?
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [])?
                        .into_jlrs_result()?;

                    let v2 = vts
                        .apply_type(&mut frame, &mut [ty2])
                        .into_jlrs_result()?
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [])?
                        .into_jlrs_result()?;

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "valuedispatch")?
                        .as_managed();

                    let o1 = func.call1(&mut frame, v1).unwrap().unbox::<isize>()?;
                    let o2 = func.call1(&mut frame, v2).unwrap().unbox::<f64>()?;

                    assert_eq!(o1, 3isize);
                    assert_eq!(o2, 3.0f64);
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn union_all_tests() {
        create_new_unionall();
        instantiate_unionall();
        apply_value_type();
    }
}
