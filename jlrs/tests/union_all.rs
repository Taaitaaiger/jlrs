use jlrs::util::JULIA;
use jlrs::{
    prelude::*,
    value::{type_var::TypeVar, union_all::UnionAll},
};

#[test]
fn create_new_unionall() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(3, |global, frame| {
            let atype = UnionAll::array_type(global);
            let body = atype.body();
            let tvar = TypeVar::new(
                frame,
                "V",
                None,
                Some(DataType::number_type(global).as_value()),
            )?;
            let ua = Value::new_unionall(frame, tvar, body)?.cast::<UnionAll>()?;
            let v = ua.var();

            let equals = Module::base(global)
                .function("!=")?
                .call2(frame, v.as_value(), atype.var().as_value())?
                .unwrap()
                .cast::<bool>()?;
            assert!(equals);
            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn instantiate_unionall() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(4, |global, frame| {
            let v = Value::new(frame, 3i8)?;
            let out = Module::main(global)
                .submodule("JlrsTests")?
                .global("ParameterStruct")?
                .apply_type(frame, &mut [DataType::int8_type(global).as_value()])?
                .cast::<DataType>()?
                .instantiate(frame, &mut [v])?
                .get_field(frame, "a")?
                .cast::<i8>()?;

            assert_eq!(out, 3);
            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn apply_value_type() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(8, |global, frame| {
            let ty1 = Value::new(frame, 1isize)?;
            let ty2 = Value::new(frame, 2isize)?;

            let vts = Module::main(global)
                .submodule("JlrsTests")?
                .global("ValueTypeStruct")?;

            let v1 = vts
                .apply_type(frame, &mut [ty1])?
                .cast::<DataType>()?
                .instantiate(frame, &mut [])?;

            let v2 = vts
                .apply_type(frame, &mut [ty2])?
                .cast::<DataType>()?
                .instantiate(frame, &mut [])?;

            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("valuedispatch")?;

            let o1 = func.call1(frame, v1)?.unwrap().cast::<isize>()?;
            let o2 = func.call1(frame, v2)?.unwrap().cast::<f64>()?;

            assert_eq!(o1, 3isize);
            assert_eq!(o2, 3.0f64);
            Ok(())
        })
        .unwrap();
    })
}
