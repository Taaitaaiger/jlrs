mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{
        data::managed::{type_var::TypeVar, union_all::UnionAll},
        prelude::*,
    };

    use super::util::JULIA;

    fn create_new_unionall() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let atype = UnionAll::array_type(&frame);
                    let body = atype.body();
                    let number_type = DataType::number_type(&frame).as_value();
                    let tvar = TypeVar::new(&mut frame, "V", None, Some(number_type)).unwrap();

                    let ua = UnionAll::new(&mut frame, tvar, body)
                        .unwrap()
                        .cast::<UnionAll>()
                        .unwrap();
                    let v = ua.var();

                    let equals = Module::base(&frame)
                        .global(&frame, "!=")
                        .unwrap()
                        .as_managed()
                        .call(&mut frame, [v.as_value(), atype.var().as_value()])
                        .unwrap()
                        .unbox::<bool>()
                        .unwrap()
                        .as_bool();
                    assert!(equals);
                })
            })
        })
    }

    fn instantiate_unionall() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let v = Value::new(&mut frame, 3i8);
                    let args = [DataType::int8_type(&frame).as_value()];
                    let out = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "ParameterStruct")
                        .unwrap()
                        .as_managed()
                        .apply_type(&mut frame, args)
                        .unwrap()
                        .call(&mut frame, [v])
                        .unwrap()
                        .get_field(&mut frame, "a")
                        .unwrap()
                        .unbox::<i8>()
                        .unwrap();

                    assert_eq!(out, 3);
                })
            })
        })
    }

    fn apply_value_type() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let ty1 = Value::new(&mut frame, 1isize);
                    let ty2 = Value::new(&mut frame, 2isize);

                    let vts = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "ValueTypeStruct")
                        .unwrap()
                        .as_managed();

                    let v1 = vts
                        .apply_type(&mut frame, &mut [ty1])
                        .unwrap()
                        .call(&mut frame, &mut [])
                        .unwrap();

                    let v2 = vts
                        .apply_type(&mut frame, &mut [ty2])
                        .unwrap()
                        .call(&mut frame, &mut [])
                        .unwrap();

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "valuedispatch")
                        .unwrap()
                        .as_managed();

                    let o1 = func
                        .call(&mut frame, [v1])
                        .unwrap()
                        .unbox::<isize>()
                        .unwrap();
                    let o2 = func.call(&mut frame, [v2]).unwrap().unbox::<f64>().unwrap();

                    assert_eq!(o1, 3isize);
                    assert_eq!(o2, 3.0f64);
                })
            })
        })
    }

    #[test]
    fn union_all_tests() {
        create_new_unionall();
        instantiate_unionall();
        apply_value_type();
    }
}
