// TODO: actually  use CCall

use jlrs::prelude::*;

// This crate is called `ccall`, so the library is called `libccall`. The functions are
// annotated with `no_mangle` to prevent name mangling and `extern "C"` to make them callable
// with the C ABI.

// Add two 32-bit signed integers, it can be called from Julia with:
// `ccall((:add, "libccall"), Int32, (Int32, Int32), a, b)` where `a` and `b` are `Int32`s.
// Note that you can write this function and use it from Julia *without* jlrs.
#[no_mangle]
pub unsafe extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Increment every element in an array of `f64`s, it can be called from Julia with:
// `ccall((:incr_array, "libccall"), Cvoid, (Array{Float64},), arr)`  where `arr` is an
// `Array{Float64}`.
#[no_mangle]
pub unsafe extern "C" fn incr_array(mut arr: TypedArray<f64>) {
    let Ok(mut arr) = arr.track_mut() else {
        return;
    };

    let Ok(mut arr) = arr.bits_data_mut() else {
        return // unreachable
    };

    for x in arr.as_mut_slice() {
        *x += 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        pub static JULIA: RefCell<PendingJulia> = {
            RefCell::new(unsafe { RuntimeBuilder::new().start().unwrap() })
        };
    }

    #[test]
    fn call_add() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let mut frame = StackFrame::new();
            let mut jlrs = jlrs.instance(&mut frame);

            jlrs.scope(|mut frame| unsafe {
                let add_ptr = Value::new(&mut frame, add as *mut std::ffi::c_void);

                let func = Value::eval_string(
                    &mut frame,
                    "addfunc(add_ptr::Ptr{Cvoid})::Int = ccall(add_ptr, Int32, (Int32, Int32), 1, 2)"
                ).into_jlrs_result()?;

                let output = func.call1(&mut frame, add_ptr)
                    .into_jlrs_result()?
                    .unbox::<isize>()?;

                assert_eq!(output, 3);

                Ok(())
            }).unwrap();
        })
    }

    #[test]
    fn call_incr_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let mut frame = StackFrame::new();
            let mut jlrs = jlrs.instance(&mut frame);

            jlrs.scope(|mut frame| unsafe {
                // Cast the function to a void pointer
                let incr_array_ptr = Value::new(&mut frame, incr_array as *mut std::ffi::c_void);

                // Value::eval_string can be used to create new functions.
                let func = Value::eval_string(
                    &mut frame,
                    "incrarray(incr_array_ptr::Ptr{Cvoid}, arr::Array{Float64, 1}) = ccall(incr_array_ptr, Cvoid, (Array{Float64, 1},), arr)"
                ).into_jlrs_result()?;

                let data  = vec![1.0f64, 2.0, 3.0];
                let array = TypedArray::from_vec_unchecked(frame.as_extended_target(), data, 3)?;

                // Call the function and unbox the result.
                let output = func.call2(&frame, incr_array_ptr, array.as_value());
                assert!(output.is_ok());

                {
                    let data = array.inline_data()?;
                    assert_eq!(data[0], 2.0);
                    assert_eq!(data[1], 3.0);
                    assert_eq!(data[2], 4.0);
                }

                Ok(())
            }).unwrap();
        })
    }
}
