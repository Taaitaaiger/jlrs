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
pub unsafe extern "C" fn incr_array(arr: TypedArray<f64>) {
    // We want to mutably borrow the array data but don't need to protect any new values, so we
    // can use `CCall::null_frame` to avoid allocations.
    CCall::new()
        .null_scope(|frame| {
            for x in arr.bits_data_mut(frame)?.as_mut_slice() {
                *x += 1.0;
            }

            Ok(())
        })
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        pub static JULIA: RefCell<Julia> = {
            RefCell::new(unsafe { RuntimeBuilder::new().start().unwrap() })
        };
    }

    #[test]
    fn call_add() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_global, frame| unsafe {
                let add_ptr = Value::new(&mut *frame, add as *mut std::ffi::c_void)?;

                let func = Value::eval_string(
                    &mut *frame,
                    "addfunc(add_ptr::Ptr{Cvoid})::Int = ccall(add_ptr, Int32, (Int32, Int32), 1, 2)"
                )?.into_jlrs_result()?;

                let output = func.call1(&mut *frame, add_ptr)?
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
            jlrs.scope(|global, frame| unsafe {
                // Cast the function to a void pointer
                let incr_array_ptr = Value::new(&mut *frame, incr_array as *mut std::ffi::c_void)?;

                // Value::eval_string can be used to create new functions.
                let func = Value::eval_string(
                    &mut *frame,
                    "incrarray(incr_array_ptr::Ptr{Cvoid}, arr::Array{Float64, 1}) = ccall(incr_array_ptr, Cvoid, (Array{Float64, 1},), arr)"
                )?.into_jlrs_result()?;

                let data  = vec![1.0f64, 2.0, 3.0];
                let array = TypedArray::from_vec_unchecked(&mut *frame, data, 3)?;

                // Call the function and unbox the result.
                let output = func.call2_unrooted(global, incr_array_ptr, array.as_value());
                assert!(output.is_ok());

                {
                    let data = array.inline_data(frame)?;
                    assert_eq!(data[0], 2.0);
                    assert_eq!(data[1], 3.0);
                    assert_eq!(data[2], 4.0);
                }

                Ok(())
            }).unwrap();
        })
    }
}
