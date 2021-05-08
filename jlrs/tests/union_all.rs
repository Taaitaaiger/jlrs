use jlrs::util::JULIA;
use jlrs::{
    prelude::*,
    value::{type_var::TypeVar, union_all::UnionAll},
};

#[test]
fn create_new_unionall() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(3, |global, frame| {
            let atype = UnionAll::array_type(global);
            let body = unsafe { atype.body().assume_reachable_unchecked() };
            let tvar = TypeVar::new(
                frame,
                "V",
                None,
                Some(DataType::number_type(global).as_value()),
            )?;
            let ua = Value::new_unionall(&mut *frame, tvar, body)?.cast::<UnionAll>()?;
            let v = unsafe { ua.var().assume_reachable().unwrap() };

            let equals = Module::base(global)
                .function("!=")?
                .call2(&mut *frame, v.as_value(), unsafe {
                    atype.var().assume_reachable_value_unchecked()
                })?
                .unwrap()
                .unbox::<bool>()?;
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
        jlrs.scope_with_slots(4, |global, frame| {
            let v = Value::new(&mut *frame, 3i8)?;
            let out = Module::main(global)
                .submodule("JlrsTests")?
                .global("ParameterStruct")?
                .apply_type(&mut *frame, &mut [DataType::int8_type(global).as_value()])?
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [v])?
                .get_field(&mut *frame, "a")?
                .unbox::<i8>()?;

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
        jlrs.scope_with_slots(8, |global, frame| {
            let ty1 = Value::new(&mut *frame, 1isize)?;
            let ty2 = Value::new(&mut *frame, 2isize)?;

            let vts = Module::main(global)
                .submodule("JlrsTests")?
                .global("ValueTypeStruct")?;

            let v1 = vts
                .apply_type(&mut *frame, &mut [ty1])?
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [])?;

            let v2 = vts
                .apply_type(&mut *frame, &mut [ty2])?
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [])?;

            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("valuedispatch")?;

            let o1 = func.call1(&mut *frame, v1)?.unwrap().unbox::<isize>()?;
            let o2 = func.call1(&mut *frame, v2)?.unwrap().unbox::<f64>()?;

            assert_eq!(o1, 3isize);
            assert_eq!(o2, 3.0f64);
            Ok(())
        })
        .unwrap();
    })
}
