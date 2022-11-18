mod util;
#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use super::util::JULIA;
    use jlrs::error::JuliaResult;
    use jlrs::prelude::*;

    fn eval_string(string: &str, with_result: impl for<'f> FnOnce(JuliaResult<'f, 'static>)) {
        JULIA.with(|j| unsafe {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    with_result(Value::eval_string(&mut frame, string));
                    Ok(())
                })
                .unwrap();
        });
    }

    fn basic_math() {
        eval_string("Int32(2) + Int32(3)", |result| {
            assert_eq!(result.unwrap().unbox::<i32>().unwrap(), 5i32);
        });
    }

    fn runtime_error() {
        eval_string("[1, 2, 3][4]", |result| {
            assert_eq!(result.unwrap_err().datatype_name().unwrap(), "BoundsError");
        });
    }

    fn syntax_error() {
        eval_string("asdf fdsa asdf fdsa", |result| {
            assert_eq!(
                result.unwrap_err().datatype_name().unwrap(),
                "ErrorException"
            );
        });
    }

    fn define_then_use() {
        eval_string("increase(x) = x + Int32(1)", |result| {
            assert!(result.is_ok());
        });
        eval_string("increase(Int32(12))", |result| {
            assert_eq!(result.unwrap().unbox::<i32>().unwrap(), 13i32);
        });
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let func = Module::main(&frame).function(&frame, "increase")?.wrapper();
                    let twelve = Value::new(&mut frame, 12i32);
                    let result = func.call1(&mut frame, twelve);
                    assert_eq!(result.unwrap().unbox::<i32>().unwrap(), 13i32);
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn eval_string_tests() {
        basic_math();
        runtime_error();
        syntax_error();
        define_then_use();
    }
}
