use jlrs::prelude::*;
use jlrs::{error::JuliaResult, util::JULIA};

fn eval_string(string: &str, with_result: impl for<'f> FnOnce(JuliaResult<'f, 'static>)) {
    JULIA.with(|j| unsafe {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(1, |_global, frame| {
            with_result(Value::eval_string(&mut *frame, string)?);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn basic_math() {
    eval_string("Int32(2) + Int32(3)", |result| {
        assert_eq!(result.unwrap().unbox::<i32>().unwrap(), 5i32);
    });
}

#[test]
fn runtime_error() {
    eval_string("[1, 2, 3][4]", |result| {
        assert_eq!(result.unwrap_err().datatype_name().unwrap(), "BoundsError");
    });
}

#[test]
fn syntax_error() {
    eval_string("asdf fdsa asdf fdsa", |result| {
        assert_eq!(
            result.unwrap_err().datatype_name().unwrap(),
            "ErrorException"
        );
    });
}

#[test]
fn define_then_use() {
    eval_string("increase(x) = x + Int32(1)", |result| {
        assert!(result.is_ok());
    });
    eval_string("increase(Int32(12))", |result| {
        assert_eq!(result.unwrap().unbox::<i32>().unwrap(), 13i32);
    });
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(4, |global, frame| unsafe {
            let func = Module::main(global)
                .function_ref("increase")?
                .wrapper_unchecked();
            let twelve = Value::new(&mut *frame, 12i32).unwrap();
            let result = func.call1(&mut *frame, twelve)?;
            assert_eq!(result.unwrap().unbox::<i32>().unwrap(), 13i32);
            Ok(())
        })
        .unwrap();
    });
}
