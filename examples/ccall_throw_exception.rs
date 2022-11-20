//! This example shows how to throw a Julia exception from a `ccall`ed function.

use jlrs::prelude::*;

// This function returns `nothing` if a < b, throws an `AssertionError` otherwise.
#[no_mangle]
pub unsafe extern "C" fn assert_less_than(a: i32, b: i32) {
    let mut stack_frame = StackFrame::new();
    let ccall = CCall::new(&mut stack_frame);

    if a >= b {
        // Safe because there are no pending drops.
        ccall.throw_exception(|frame| {
            let msg = JuliaString::new(frame.as_mut(), "a is larger than b").as_value();

            Module::core(&frame)
                .global(&frame, "AssertionError")
                .expect("AssertionError does not exist in Core")
                .value()
                .cast::<DataType>()
                .expect("AssertionError is not a DataType")
                .instantiate_unchecked(frame.as_mut(), [msg])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_assert_less_than() {
        let mut jlrs = unsafe { RuntimeBuilder::new().start().unwrap() };
        let mut frame = StackFrame::new();
        let mut jlrs = jlrs.instance(&mut frame);

        jlrs.scope(|mut frame| unsafe {
            let assert_less_than_ptr =
                Value::new(&mut frame, assert_less_than as *mut std::ffi::c_void);

            let func = Value::eval_string(
                &mut frame,
                "throwing_func(fnptr::Ptr{Cvoid}) = ccall(fnptr, Cvoid, (Int32, Int32), 2, 1)",
            )
            .into_jlrs_result()?;

            let output = func.call1(&mut frame, assert_less_than_ptr);
            assert!(output.is_err());
            Ok(())
        })
        .unwrap();
    }
}
