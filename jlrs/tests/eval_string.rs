use jlrs::prelude::*;
use jlrs::util::JULIA;
use jlrs::value::CallResult;

fn eval_string(string: &str, with_result: impl for<'f> FnOnce(CallResult<'f, 'static>)) {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(1, |global, frame| {
            with_result(jlrs::eval_string(global, frame, string)?);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn basic_math() {
    eval_string("Int32(2) + Int32(3)", |result| {
        assert_eq!(result.unwrap().cast::<i32>().unwrap(), 5i32);
    });
}

#[test]
fn runtime_error() {
    eval_string("[1, 2, 3][4]", |result| {
        assert_eq!(result.unwrap_err().type_name(), "BoundsError");
    });
}

#[test]
fn syntax_error() {
    eval_string("asdf fdsa asdf fdsa", |result| {
        assert_eq!(result.unwrap_err().type_name(), "ErrorException");
    });
}

#[test]
fn define_then_use() {
    eval_string("increase(x) = x + Int32(1)", |result| {
        assert!(result.is_ok());
    });
    eval_string("increase(Int32(12))", |result| {
        assert_eq!(result.unwrap().cast::<i32>().unwrap(), 13i32);
    });
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(4, |global, frame| {
            let func = Module::main(global).function("increase")?;
            let twelve = Value::new(frame, 12i32).unwrap();
            let result = func.call1(frame, twelve)?;
            assert_eq!(result.unwrap().cast::<i32>().unwrap(), 13i32);
            Ok(())
        })
        .unwrap();
    });
}
