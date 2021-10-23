mod util;
use jlrs::{
    prelude::*,
    wrappers::ptr::{type_var::TypeVar, union_all::UnionAll},
};
use util::JULIA;

#[test]
fn create_new_unionall() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(3, |global, frame| unsafe {
            let atype = UnionAll::array_type(global);
            let body = atype.body().wrapper_unchecked();
            let tvar = TypeVar::new(
                &mut *frame,
                "V",
                None,
                Some(DataType::number_type(global).as_value()),
            )?
            .into_jlrs_result()?
            .cast()?;
            let ua = UnionAll::new(&mut *frame, tvar, body)?
                .into_jlrs_result()?
                .cast::<UnionAll>()?;
            let v = ua.var().wrapper().unwrap();

            let equals = Module::base(global)
                .function_ref("!=")?
                .wrapper_unchecked()
                .call2(&mut *frame, v.as_value(), atype.var().value_unchecked())?
                .unwrap()
                .unbox::<bool>()?
                .as_bool();
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
        jlrs.scope_with_slots(4, |global, frame| unsafe {
            let v = Value::new(&mut *frame, 3i8)?;
            let out = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .global_ref("ParameterStruct")?
                .wrapper_unchecked()
                .apply_type(&mut *frame, &mut [DataType::int8_type(global).as_value()])?
                .into_jlrs_result()?
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [v])?
                .into_jlrs_result()?
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
        jlrs.scope_with_slots(8, |global, frame| unsafe {
            let ty1 = Value::new(&mut *frame, 1isize)?;
            let ty2 = Value::new(&mut *frame, 2isize)?;

            let vts = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .global_ref("ValueTypeStruct")?
                .wrapper_unchecked();

            let v1 = vts
                .apply_type(&mut *frame, &mut [ty1])?
                .into_jlrs_result()?
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [])?
                .into_jlrs_result()?;

            let v2 = vts
                .apply_type(&mut *frame, &mut [ty2])?
                .into_jlrs_result()?
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [])?
                .into_jlrs_result()?;

            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("valuedispatch")?
                .wrapper_unchecked();

            let o1 = func.call1(&mut *frame, v1)?.unwrap().unbox::<isize>()?;
            let o2 = func.call1(&mut *frame, v2)?.unwrap().unbox::<f64>()?;

            assert_eq!(o1, 3isize);
            assert_eq!(o2, 3.0f64);
            Ok(())
        })
        .unwrap();
    })
}
